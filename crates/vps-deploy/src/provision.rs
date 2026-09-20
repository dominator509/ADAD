use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, TcpStream};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use adad_core::Error;

const TOR_SOCKS_ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
const SOCKS5_VERSION: u8 = 0x05;
const SOCKS5_CONNECT: u8 = 0x01;
const SOCKS5_NO_AUTH: u8 = 0x00;
const SOCKS5_DOMAIN: u8 = 0x03;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionTarget {
    pub host: String,
    pub user: String,
    pub port: u16,
}

impl ProvisionTarget {
    #[must_use]
    pub fn new(host: impl Into<String>, user: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            user: user.into(),
            port,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SshOutput {
    pub exit_status: i32,
    pub stdout: String,
}

impl SshOutput {
    #[must_use]
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            exit_status: 0,
            stdout: stdout.into(),
        }
    }
}

pub trait SshSession {
    fn run_setup_script(
        &mut self,
        target: &ProvisionTarget,
        setup_script: &str,
    ) -> Result<SshOutput, Error>;
}

/// Production SSH session backed by the system OpenSSH client.
///
/// OpenSSH keeps its normal host-key verification and known-hosts behavior;
/// password prompts are disabled so an unattended run cannot hang waiting for
/// interactive input. The caller must provide an explicit confirmation before
/// invoking a real provisioning operation.
#[derive(Clone, Copy, Debug, Default)]
pub struct OpenSshSession;

impl OpenSshSession {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl SshSession for OpenSshSession {
    fn run_setup_script(
        &mut self,
        target: &ProvisionTarget,
        setup_script: &str,
    ) -> Result<SshOutput, Error> {
        validate_target(target)?;
        if setup_script.trim().is_empty() {
            return Err(Error::VpsProvision);
        }

        let destination = format!("{}@{}", target.user, target.host);
        let proxy_command = proxy_command()?;
        let mut child = Command::new("ssh")
            .arg("-p")
            .arg(target.port.to_string())
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg("RequestTTY=no")
            .arg("-o")
            .arg("ConnectTimeout=10")
            .arg("-o")
            .arg(format!("ProxyCommand={proxy_command}"))
            .arg(destination)
            .arg("sh")
            .arg("-s")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| Error::VpsProvision)?;

        let Some(mut stdin) = child.stdin.take() else {
            return Err(Error::VpsProvision);
        };
        stdin
            .write_all(setup_script.as_bytes())
            .map_err(|_| Error::VpsProvision)?;
        drop(stdin);

        let output = child.wait_with_output().map_err(|_| Error::VpsProvision)?;
        Ok(SshOutput {
            exit_status: output.status.code().unwrap_or(255),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        })
    }
}

/// Entry point for OpenSSH's ProxyCommand. The target hostname is encoded as
/// a SOCKS5 domain request so name resolution happens at the Tor boundary and
/// never through the local resolver.
pub fn tor_connect(host: &str, port: u16) -> Result<(), Error> {
    validate_host_port(host, port)?;
    let stream = connect_through_socks5(TOR_SOCKS_ADDR, host, port)?;
    relay_stdio(stream)
}

fn connect_through_socks5(
    proxy_addr: SocketAddr,
    host: &str,
    port: u16,
) -> Result<TcpStream, Error> {
    validate_host_port(host, port)?;
    let mut stream = TcpStream::connect_timeout(&proxy_addr, Duration::from_secs(10))
        .map_err(|_| Error::VpsProvision)?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|_| Error::VpsProvision)?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|_| Error::VpsProvision)?;

    stream
        .write_all(&[SOCKS5_VERSION, 1, SOCKS5_NO_AUTH])
        .map_err(|_| Error::VpsProvision)?;
    let mut greeting = [0_u8; 2];
    stream
        .read_exact(&mut greeting)
        .map_err(|_| Error::VpsProvision)?;
    if greeting != [SOCKS5_VERSION, SOCKS5_NO_AUTH] {
        return Err(Error::VpsProvision);
    }

    let host_bytes = host.as_bytes();
    let host_len = u8::try_from(host_bytes.len()).map_err(|_| Error::VpsProvision)?;
    let mut request = Vec::with_capacity(7 + host_bytes.len());
    request.extend_from_slice(&[SOCKS5_VERSION, SOCKS5_CONNECT, 0, SOCKS5_DOMAIN, host_len]);
    request.extend_from_slice(host_bytes);
    request.extend_from_slice(&port.to_be_bytes());
    stream
        .write_all(&request)
        .map_err(|_| Error::VpsProvision)?;

    let mut response = [0_u8; 4];
    stream
        .read_exact(&mut response)
        .map_err(|_| Error::VpsProvision)?;
    if response[0] != SOCKS5_VERSION || response[1] != 0 {
        return Err(Error::VpsProvision);
    }
    consume_socks5_bound_address(&mut stream, response[3])?;

    stream
        .set_read_timeout(None)
        .map_err(|_| Error::VpsProvision)?;
    stream
        .set_write_timeout(None)
        .map_err(|_| Error::VpsProvision)?;
    Ok(stream)
}

fn consume_socks5_bound_address(stream: &mut TcpStream, address_type: u8) -> Result<(), Error> {
    let address_len = match address_type {
        0x01 => 4,
        0x04 => 16,
        0x03 => {
            let mut length = [0_u8; 1];
            stream
                .read_exact(&mut length)
                .map_err(|_| Error::VpsProvision)?;
            usize::from(length[0])
        }
        _ => return Err(Error::VpsProvision),
    };
    let mut address_and_port = vec![0_u8; address_len + 2];
    stream
        .read_exact(&mut address_and_port)
        .map_err(|_| Error::VpsProvision)
}

fn relay_stdio(mut stream: TcpStream) -> Result<(), Error> {
    let mut upload = stream.try_clone().map_err(|_| Error::VpsProvision)?;
    let _upload_thread = std::thread::spawn(move || {
        let mut stdin = io::stdin();
        let _ = io::copy(&mut stdin, &mut upload);
        let _ = upload.shutdown(Shutdown::Write);
    });

    let mut stdout = io::stdout();
    io::copy(&mut stream, &mut stdout).map_err(|_| Error::VpsProvision)?;
    stdout.flush().map_err(|_| Error::VpsProvision)
}

fn proxy_command() -> Result<String, Error> {
    let executable = std::env::current_exe().map_err(|_| Error::VpsProvision)?;
    let executable = executable.to_str().ok_or(Error::VpsProvision)?;
    Ok(format!("{} tor-connect %h %p", shell_quote(executable)))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvisionHandle {
    pub target: ProvisionTarget,
    pub stdout: String,
    pub elapsed: Duration,
}

pub fn provision(
    session: &mut impl SshSession,
    target: ProvisionTarget,
    setup_script: &str,
) -> Result<ProvisionHandle, Error> {
    validate_target(&target)?;
    if setup_script.trim().is_empty() {
        return Err(Error::VpsProvision);
    }

    let started = Instant::now();
    let output = session.run_setup_script(&target, setup_script)?;
    if output.exit_status != 0 {
        return Err(Error::VpsProvision);
    }

    Ok(ProvisionHandle {
        target,
        stdout: output.stdout,
        elapsed: started.elapsed(),
    })
}

fn validate_target(target: &ProvisionTarget) -> Result<(), Error> {
    if target.host.trim().is_empty()
        || target.user.trim().is_empty()
        || target.port == 0
        || target.host.starts_with('-')
        || !is_safe_host(&target.host)
        || target.user.starts_with('-')
        || target
            .host
            .chars()
            .any(|ch| ch.is_whitespace() || ch == '@')
        || target
            .user
            .chars()
            .any(|ch| ch.is_whitespace() || ch == '@')
    {
        return Err(Error::VpsProvision);
    }
    Ok(())
}

fn validate_host_port(host: &str, port: u16) -> Result<(), Error> {
    if port == 0 || !is_safe_host(host) {
        return Err(Error::VpsProvision);
    }
    Ok(())
}

fn is_safe_host(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 253
        && host.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b':' | b'[' | b']' | b'-' | b'_')
        })
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    use super::{connect_through_socks5, validate_target, ProvisionTarget};

    #[test]
    fn socks5_connect_sends_the_hostname_for_proxy_resolution() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind SOCKS fixture");
        let proxy_address = listener.local_addr().expect("proxy address");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept SOCKS client");
            let mut greeting = [0_u8; 3];
            stream.read_exact(&mut greeting).expect("read greeting");
            assert_eq!(greeting, [0x05, 0x01, 0x00]);
            stream.write_all(&[0x05, 0x00]).expect("write greeting");

            let mut request_header = [0_u8; 5];
            stream
                .read_exact(&mut request_header)
                .expect("read connect header");
            assert_eq!(&request_header[..4], &[0x05, 0x01, 0x00, 0x03]);
            let host_len = usize::from(request_header[4]);
            let mut host_and_port = vec![0_u8; host_len + 2];
            stream
                .read_exact(&mut host_and_port)
                .expect("read domain request");
            assert_eq!(&host_and_port[..host_len], b"hidden.onion");
            assert_eq!(&host_and_port[host_len..], &22_u16.to_be_bytes());

            stream
                .write_all(&[0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0, 22])
                .expect("write success response");
        });

        let stream = connect_through_socks5(proxy_address, "hidden.onion", 22)
            .expect("SOCKS domain connect succeeds");
        drop(stream);
        server.join().expect("SOCKS fixture succeeds");
    }

    #[test]
    fn target_validation_rejects_proxy_command_injection_characters() {
        for host in ["host;touch", "host$(id)", "host`id`", "host/name"] {
            let target = ProvisionTarget::new(host, "adad", 22);
            assert!(
                validate_target(&target).is_err(),
                "host should be rejected: {host}"
            );
        }
    }
}
