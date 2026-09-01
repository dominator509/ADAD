use std::env;
use std::fs;
use std::time::UNIX_EPOCH;

#[cfg(target_os = "linux")]
use std::io::Read;

use adad_core::Error;
use metafuse::{scrub_metadata, ScrubPolicy, VaultMetadata};

fn main() {
    if let Err(error) = run() {
        eprintln!("metafuse: {error}");
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
            println!("metafuse {}", adad_core::version());
            Ok(())
        }
        "scrub" => {
            let path = args.next().ok_or(Error::Metafuse)?;
            if args.next().is_some() {
                return Err(Error::Metafuse);
            }
            let timestamp = fs::metadata(&path)
                .map_err(|_| Error::Io)?
                .modified()
                .map_err(|_| Error::Io)?
                .duration_since(UNIX_EPOCH)
                .map_err(|_| Error::Io)?
                .as_secs();
            let timestamp = i64::try_from(timestamp).map_err(|_| Error::Metafuse)?;
            let metadata = VaultMetadata::new(&path, 1, 1, timestamp, timestamp, timestamp, vec![]);
            let policy = ScrubPolicy::new(65_534, 65_533, [0xAD; 32])?;
            let presented = scrub_metadata(&metadata, &policy)?;
            println!("{}", presented.rendered_public_fields());
            Ok(())
        }
        "mount" => {
            let source = args.next().ok_or(Error::Metafuse)?;
            let mountpoint = args.next().ok_or(Error::Metafuse)?;
            if args.next().is_some() {
                return Err(Error::Metafuse);
            }
            let policy = runtime_policy()?;
            metafuse::mount_read_only(&source, &mountpoint, policy)
        }
        _ => Err(Error::Metafuse),
    }
}

#[cfg(target_os = "linux")]
fn runtime_policy() -> Result<ScrubPolicy, Error> {
    let mut seed = [0_u8; 32];
    fs::File::open("/dev/urandom")
        .map_err(|_| Error::Metafuse)?
        .read_exact(&mut seed)
        .map_err(|_| Error::Metafuse)?;
    ScrubPolicy::new(65_534, 65_533, seed)
}

#[cfg(not(target_os = "linux"))]
fn runtime_policy() -> Result<ScrubPolicy, Error> {
    Err(Error::Metafuse)
}

fn print_help() {
    println!(
        "metafuse {}\n\nUsage:\n  metafuse scrub <path>\n  metafuse mount <source-directory> <mountpoint>\n\nThe mount command presents a read-only Linux FUSE view with scrubbed ownership and timestamps, rejects symlinks and special files, hides extended attributes, and uses a per-mount random policy seed. The source is never modified. The scrub command renders the same metadata policy without mounting.",
        adad_core::version()
    );
}
