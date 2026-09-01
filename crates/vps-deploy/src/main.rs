use std::env;
use std::io::Read;

use adad_core::Error;
use vps_deploy::{provision, tor_connect, OpenSshSession, ProvisionTarget};

fn main() {
    if let Err(error) = run() {
        eprintln!("vps-deploy: {error}");
        std::process::exit(error.exit_code());
    }
}

fn run() -> Result<(), Error> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "--help".to_owned());
    match command.as_str() {
        "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "--version" | "-V" => {
            println!("vps-deploy {}", adad_core::version());
            Ok(())
        }
        "provision" => {
            let host = args.next().ok_or(Error::VpsProvision)?;
            let user = args.next().ok_or(Error::VpsProvision)?;
            if args.next().as_deref() != Some("--script-stdin")
                || args.next().as_deref() != Some("--confirm")
                || args.next().is_some()
            {
                return Err(Error::VpsProvision);
            }
            let mut script = String::new();
            std::io::stdin()
                .read_to_string(&mut script)
                .map_err(|_| Error::Io)?;
            let target = ProvisionTarget::new(host, user, 22);
            let mut session = OpenSshSession::new();
            let result = provision(&mut session, target, &script)?;
            println!("{}", result.stdout);
            Ok(())
        }
        "tor-connect" => {
            let host = args.next().ok_or(Error::VpsProvision)?;
            let port = args
                .next()
                .ok_or(Error::VpsProvision)?
                .parse::<u16>()
                .map_err(|_| Error::VpsProvision)?;
            if args.next().is_some() {
                return Err(Error::VpsProvision);
            }
            tor_connect(&host, port)
        }
        _ => Err(Error::VpsProvision),
    }
}

fn print_help() {
    println!(
        "vps-deploy {}\n\nUsage:\n  vps-deploy provision <host> <user> --script-stdin --confirm < setup.sh\n\nThe command uses OpenSSH with normal host-key verification, BatchMode=yes, and a fixed Tor SOCKS5 ProxyCommand. --confirm is mandatory and the setup script is read from stdin; no provisioning is attempted by --help or --version.",
        adad_core::version()
    );
}
