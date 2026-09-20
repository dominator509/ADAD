use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::time::Duration;

use adad_core::Error;

use serde::{de::DeserializeOwned, Deserialize};
use serde_json::{json, Value};

pub trait WalletRpcTransport {
    fn post_json(&mut self, url: &str, body: &str) -> Result<String, Error>;
}

const TOR_SOCKS_ADDR: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
const SOCKS5_VERSION: u8 = 0x05;
const SOCKS5_CONNECT: u8 = 0x01;
const SOCKS5_NO_AUTH: u8 = 0x00;
const SOCKS5_DOMAIN: u8 = 0x03;
const MAX_RPC_RESPONSE_BYTES: usize = 1024 * 1024;

/// HTTP transport for a locally supervised `monero-wallet-rpc` instance or a
/// Tor-published onion endpoint.
///
/// Loopback RPC is sent directly to the local wallet process. Remote RPC is
/// accepted only for `.onion` hosts and is sent through the fixed Tor SOCKS5
/// listener; no clearnet or ambient-proxy path is available.
#[derive(Clone, Copy, Debug, Default)]
pub struct UreqWalletRpcTransport;

impl UreqWalletRpcTransport {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    fn endpoint(url: &str) -> Result<WalletEndpoint, Error> {
        let (scheme, rest) = url.split_once("://").ok_or(Error::WalletRpc)?;
        if scheme != "http" {
            return Err(Error::WalletRpc);
        }

        let (authority, path) = rest.split_once('/').map_or((rest, "/"), |(authority, _)| {
            (authority, &rest[authority.len()..])
        });
        if authority.is_empty() || authority.contains('@') {
            return Err(Error::WalletRpc);
        }

        let (host, port) = authority.rsplit_once(':').map_or_else(
            || (authority.trim_matches(['[', ']']), 80),
            |(host, port)| {
                if host.contains(':') && !host.starts_with('[') {
                    (authority.trim_matches(['[', ']']), 80)
                } else {
                    (
                        host.trim_matches(['[', ']']),
                        port.parse::<u16>().unwrap_or_default(),
                    )
                }
            },
        );
        if host.is_empty()
            || port == 0
            || host
                .bytes()
                .any(|byte| !(byte.is_ascii_alphanumeric() || b".-_[]:".contains(&byte)))
            || path
                .bytes()
                .any(|byte| byte.is_ascii_control() || byte == b' ')
        {
            return Err(Error::WalletRpc);
        }

        if matches!(host, "127.0.0.1" | "localhost" | "::1") {
            return Ok(WalletEndpoint::Loopback {
                url: url.to_owned(),
            });
        }

        if host.to_ascii_lowercase().ends_with(".onion") && !host.contains(['[', ']', ':']) {
            return Ok(WalletEndpoint::Onion {
                host: host.to_owned(),
                port,
                path: path.to_owned(),
            });
        }

        Err(Error::WalletRpc)
    }
}

impl WalletRpcTransport for UreqWalletRpcTransport {
    fn post_json(&mut self, url: &str, body: &str) -> Result<String, Error> {
        match Self::endpoint(url)? {
            WalletEndpoint::Loopback { url } => post_loopback(&url, body),
            WalletEndpoint::Onion { host, port, path } => {
                post_onion_via_proxy(TOR_SOCKS_ADDR, &host, port, &path, body)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum WalletEndpoint {
    Loopback {
        url: String,
    },
    Onion {
        host: String,
        port: u16,
        path: String,
    },
}

fn post_loopback(url: &str, body: &str) -> Result<String, Error> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .proxy(None)
        .build()
        .new_agent();
    let mut response = agent
        .post(url)
        .header("Content-Type", "application/json")
        .send(body)
        .map_err(|_| Error::WalletRpc)?;
    response
        .body_mut()
        .read_to_string()
        .map_err(|_| Error::WalletRpc)
}

fn post_onion_via_proxy(
    proxy_addr: SocketAddr,
    host: &str,
    port: u16,
    path: &str,
    body: &str,
) -> Result<String, Error> {
    let mut stream = connect_through_socks5(proxy_addr, host, port)?;
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|_| Error::WalletRpc)?;

    let mut response = Vec::new();
    stream
        .take((MAX_RPC_RESPONSE_BYTES + 1) as u64)
        .read_to_end(&mut response)
        .map_err(|_| Error::WalletRpc)?;
    if response.len() > MAX_RPC_RESPONSE_BYTES {
        return Err(Error::WalletRpc);
    }
    parse_http_response(&response)
}

fn connect_through_socks5(
    proxy_addr: SocketAddr,
    host: &str,
    port: u16,
) -> Result<TcpStream, Error> {
    if port == 0
        || host.is_empty()
        || host.len() > 253
        || !host
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b".-_".contains(&byte))
    {
        return Err(Error::WalletRpc);
    }

    let mut stream = TcpStream::connect_timeout(&proxy_addr, Duration::from_secs(10))
        .map_err(|_| Error::WalletRpc)?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|_| Error::WalletRpc)?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|_| Error::WalletRpc)?;
    stream
        .write_all(&[SOCKS5_VERSION, 1, SOCKS5_NO_AUTH])
        .map_err(|_| Error::WalletRpc)?;

    let mut greeting = [0_u8; 2];
    stream
        .read_exact(&mut greeting)
        .map_err(|_| Error::WalletRpc)?;
    if greeting != [SOCKS5_VERSION, SOCKS5_NO_AUTH] {
        return Err(Error::WalletRpc);
    }

    let host_bytes = host.as_bytes();
    let host_len = u8::try_from(host_bytes.len()).map_err(|_| Error::WalletRpc)?;
    let mut request = Vec::with_capacity(host_bytes.len() + 7);
    request.extend_from_slice(&[SOCKS5_VERSION, SOCKS5_CONNECT, 0, SOCKS5_DOMAIN, host_len]);
    request.extend_from_slice(host_bytes);
    request.extend_from_slice(&port.to_be_bytes());
    stream.write_all(&request).map_err(|_| Error::WalletRpc)?;

    let mut response = [0_u8; 4];
    stream
        .read_exact(&mut response)
        .map_err(|_| Error::WalletRpc)?;
    if response[0] != SOCKS5_VERSION || response[1] != 0 {
        return Err(Error::WalletRpc);
    }
    consume_bound_address(&mut stream, response[3])?;
    stream
        .set_read_timeout(None)
        .map_err(|_| Error::WalletRpc)?;
    stream
        .set_write_timeout(None)
        .map_err(|_| Error::WalletRpc)?;
    Ok(stream)
}

fn consume_bound_address(stream: &mut TcpStream, address_type: u8) -> Result<(), Error> {
    let address_len = match address_type {
        0x01 => 4,
        0x04 => 16,
        0x03 => {
            let mut length = [0_u8; 1];
            stream
                .read_exact(&mut length)
                .map_err(|_| Error::WalletRpc)?;
            usize::from(length[0])
        }
        _ => return Err(Error::WalletRpc),
    };
    let mut address_and_port = vec![0_u8; address_len + 2];
    stream
        .read_exact(&mut address_and_port)
        .map_err(|_| Error::WalletRpc)
}

fn parse_http_response(response: &[u8]) -> Result<String, Error> {
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or(Error::WalletRpc)?;
    let headers = std::str::from_utf8(&response[..header_end]).map_err(|_| Error::WalletRpc)?;
    let status_line = headers.lines().next().ok_or(Error::WalletRpc)?;
    if status_line
        .split_whitespace()
        .nth(1)
        .is_none_or(|status| status != "200")
    {
        return Err(Error::WalletRpc);
    }
    let body_start = header_end + 4;
    let body = response.get(body_start..).ok_or(Error::WalletRpc)?;
    if let Some(length) = headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length")
            .then(|| value.trim().parse::<usize>().ok())
            .flatten()
    }) {
        if body.len() < length {
            return Err(Error::WalletRpc);
        }
        return String::from_utf8(body[..length].to_vec()).map_err(|_| Error::WalletRpc);
    }
    String::from_utf8(body.to_vec()).map_err(|_| Error::WalletRpc)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WalletRpcClient<T> {
    rpc_url: String,
    transport: T,
}

impl<T> WalletRpcClient<T>
where
    T: WalletRpcTransport,
{
    #[must_use]
    pub fn new(rpc_url: impl Into<String>, transport: T) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            transport,
        }
    }

    pub fn balance(&mut self) -> Result<Balance, Error> {
        self.call("get_balance", json!({ "account_index": 0 }))
    }

    pub fn address(&mut self) -> Result<WalletAddress, Error> {
        self.call("get_address", json!({ "account_index": 0 }))
    }

    pub fn prepare_transfer(
        &mut self,
        address: impl Into<String>,
        amount_atomic: u64,
    ) -> Result<PreparedTransfer, Error> {
        self.call(
            "transfer",
            json!({
                "destinations": [{
                    "amount": amount_atomic,
                    "address": address.into(),
                }],
                "account_index": 0,
                "do_not_relay": true,
                "get_tx_metadata": true,
            }),
        )
    }

    fn call<R>(&mut self, method: &str, params: Value) -> Result<R, Error>
    where
        R: DeserializeOwned,
    {
        let body = json!({
            "jsonrpc": "2.0",
            "id": "0",
            "method": method,
            "params": params,
        })
        .to_string();
        let response = self.transport.post_json(&self.rpc_url, &body)?;
        let envelope: JsonRpcResponse<R> =
            serde_json::from_str(&response).map_err(|_| Error::WalletRpc)?;

        match (envelope.result, envelope.error) {
            (Some(result), None) => Ok(result),
            _ => Err(Error::WalletRpc),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Balance {
    pub balance: u64,
    pub unlocked_balance: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct WalletAddress {
    pub address: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct PreparedTransfer {
    pub amount: u64,
    pub fee: u64,
    pub tx_hash: String,
    pub tx_metadata: String,
}

#[derive(Deserialize)]
struct JsonRpcResponse<R> {
    result: Option<R>,
    error: Option<Value>,
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    use adad_core::Error;

    use super::{post_onion_via_proxy, UreqWalletRpcTransport, WalletEndpoint, WalletRpcTransport};

    #[test]
    fn transport_rejects_non_loopback_urls() {
        let mut transport = UreqWalletRpcTransport::new();
        assert_eq!(
            transport.post_json("https://wallet.example.test/json_rpc", "{}"),
            Err(Error::WalletRpc)
        );
        assert_eq!(
            transport.post_json("http://wallet.example.test/json_rpc", "{}"),
            Err(Error::WalletRpc)
        );
    }

    #[test]
    fn transport_posts_json_to_a_loopback_wallet_rpc() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback listener");
        let address = listener.local_addr().expect("listener address");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept local request");
            let mut request = Vec::new();
            let mut chunk = [0_u8; 1024];
            loop {
                let length = stream.read(&mut chunk).expect("read local request");
                assert!(length > 0, "request closed before its body arrived");
                request.extend_from_slice(&chunk[..length]);
                if let Some(separator) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                    let body_start = separator + 4;
                    let headers = String::from_utf8_lossy(&request[..separator]);
                    let body_length = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().ok())
                                .flatten()
                        })
                        .expect("content length");
                    if request.len() >= body_start + body_length {
                        break;
                    }
                }
            }
            let request = String::from_utf8_lossy(&request);
            assert!(request.lines().any(|line| {
                let Some((name, value)) = line.split_once(':') else {
                    return false;
                };
                name.eq_ignore_ascii_case("content-type") && value.trim() == "application/json"
            }));
            assert!(request.ends_with("{}"));
            let body = r#"{"jsonrpc":"2.0","id":"0","result":{}}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .expect("write local response");
        });

        let mut transport = UreqWalletRpcTransport::new();
        let body = transport
            .post_json(&format!("http://{address}/json_rpc"), "{}")
            .expect("loopback request succeeds");
        assert!(body.contains("\"result\":{}"));
        server.join().expect("local server succeeds");
    }

    #[test]
    fn transport_routes_onion_rpc_through_socks5_without_local_dns() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind SOCKS fixture");
        let proxy_address = listener.local_addr().expect("proxy address");
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept SOCKS client");
            let mut greeting = [0_u8; 3];
            stream
                .read_exact(&mut greeting)
                .expect("read SOCKS greeting");
            assert_eq!(greeting, [0x05, 0x01, 0x00]);
            stream
                .write_all(&[0x05, 0x00])
                .expect("write SOCKS greeting");

            let mut connect_header = [0_u8; 5];
            stream
                .read_exact(&mut connect_header)
                .expect("read SOCKS connect header");
            assert_eq!(&connect_header[..4], &[0x05, 0x01, 0x00, 0x03]);
            let host_len = usize::from(connect_header[4]);
            let mut host_and_port = vec![0_u8; host_len + 2];
            stream
                .read_exact(&mut host_and_port)
                .expect("read SOCKS domain");
            assert_eq!(&host_and_port[..host_len], b"wallet.onion");
            assert_eq!(&host_and_port[host_len..], &18_082_u16.to_be_bytes());
            stream
                .write_all(&[0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0, 1])
                .expect("write SOCKS success");

            let mut request = Vec::new();
            let mut chunk = [0_u8; 1024];
            loop {
                let length = stream.read(&mut chunk).expect("read onion HTTP request");
                if length == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..length]);
                if let Some(separator) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                    let header = String::from_utf8_lossy(&request[..separator]);
                    let body_length = header
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().ok())
                                .flatten()
                        })
                        .expect("HTTP content length");
                    if request.len() >= separator + 4 + body_length {
                        break;
                    }
                }
            }
            let request = String::from_utf8_lossy(&request);
            assert!(request.starts_with("POST /json_rpc HTTP/1.1\r\n"));
            assert!(request.contains("Host: wallet.onion:18082\r\n"));
            assert!(request.ends_with("{}"));

            let body = r#"{"jsonrpc":"2.0","id":"0","result":{}}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .expect("write onion HTTP response");
        });

        assert_eq!(
            UreqWalletRpcTransport::endpoint("http://wallet.onion:18082/json_rpc"),
            Ok(WalletEndpoint::Onion {
                host: "wallet.onion".to_owned(),
                port: 18_082,
                path: "/json_rpc".to_owned(),
            })
        );
        let body = post_onion_via_proxy(proxy_address, "wallet.onion", 18_082, "/json_rpc", "{}")
            .expect("onion request succeeds through fake SOCKS");
        assert!(body.contains("\"result\":{}"));
        server.join().expect("SOCKS fixture succeeds");
    }
}
