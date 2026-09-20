use std::env;
use std::path::Path;
use std::time::Duration;

use adad_core::Error;
use leakguard::{
    Dms, Killswitch, LocalClockTime, LuksHeaderFile, NetworkPosture, RoutingPosture, TorNtpTime,
    TunnelHealth, WireGuardController,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("leakguard: {error}");
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
            println!("leakguard {}", adad_core::version());
            Ok(())
        }
        "status" => {
            let routing = RoutingPosture::leak_free();
            routing.validate_leak_free()?;
            let wireguard = WireGuardController::default().status();
            let mut killswitch = Killswitch::new();
            killswitch.arm(NetworkPosture::new(TunnelHealth::Unknown, wireguard));
            println!(
                "state={:?} wireguard={wireguard:?} routing=leak-free firewall={:?}",
                killswitch.state(),
                killswitch.firewall()
            );
            Ok(())
        }
        "monitor" => leakguard::run_system_monitor(),
        "wireguard" => match args.next().as_deref() {
            Some("status") => {
                println!("wireguard={:?}", WireGuardController::default().status());
                Ok(())
            }
            Some("up") => {
                WireGuardController::from_env()?.up()?;
                println!("wireguard=Active interface=wg0");
                Ok(())
            }
            Some("down") => {
                WireGuardController::default().down()?;
                println!("wireguard=Inactive interface=wg0");
                Ok(())
            }
            _ => Err(Error::Killswitch),
        },
        "dms" => run_dms(&mut args),
        _ => Err(Error::Killswitch),
    }
}

fn run_dms(args: &mut impl Iterator<Item = String>) -> Result<(), Error> {
    if args.next().as_deref() != Some("evaluate-image") {
        return Err(Error::Killswitch);
    }
    let path = args.next().ok_or(Error::VaultUnlock)?;
    let header_len = parse_u64(args.next())?;
    let last_access = parse_u64(args.next())?;
    let now = parse_u64(args.next())?;
    let window = parse_u64(args.next())?;
    if args.next().is_some() || window == 0 {
        return Err(Error::Killswitch);
    }

    let mut image = LuksHeaderFile::open(Path::new(&path), header_len)?;
    let mut dms = Dms::new(
        Duration::from_secs(window),
        TorNtpTime::from_unix_seconds(last_access),
    )?;
    let outcome = dms.evaluate_file(
        TorNtpTime::from_unix_seconds(now),
        LocalClockTime::from_unix_seconds(0),
        &mut image,
    )?;
    match outcome {
        leakguard::DmsOutcome::Armed { remaining } => {
            println!("dms=Armed remaining_seconds={}", remaining.as_secs());
        }
        leakguard::DmsOutcome::Expired { header_wiped } => {
            println!("dms=Expired header_wiped={header_wiped} image_only=true");
        }
    }
    Ok(())
}

fn parse_u64(value: Option<String>) -> Result<u64, Error> {
    value
        .ok_or(Error::Killswitch)?
        .parse::<u64>()
        .map_err(|_| Error::Killswitch)
}

fn print_help() {
    println!(
          "leakguard {}\n\nUsage:\n  leakguard status\n  leakguard monitor\n  leakguard wireguard status\n  leakguard wireguard up\n  leakguard wireguard down\n  leakguard dms evaluate-image <image> <header-bytes> <last-tor-ntp-seconds> <now-tor-ntp-seconds> <window-seconds>\n\nmonitor observes Linux link events and loads a complete drop-only nftables killswitch ruleset on down/deleted links. It returns an error if the event source terminates so systemd can restart it. wireguard up/down require the vault runtime to provide ADAD_WG_CONF=/run/adad/wg0.conf; configuration contents are never printed. dms evaluate-image accepts authoritative Tor-NTP seconds from its caller and can only open a regular LUKS2 image file; it rejects block devices and symlinks and ignores local clock input.",
        adad_core::version()
    );
}
