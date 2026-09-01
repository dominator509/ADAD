use std::env;
use std::io::Read;
use std::path::Path;

use adad_core::Error;
use forge::Vault;

fn main() {
    if let Err(error) = run() {
        eprintln!("forge: {error}");
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
            println!("forge {}", adad_core::version());
            Ok(())
        }
        "create" => {
            let path = args.next().ok_or(Error::VaultUnlock)?;
            reject_extra_args(&mut args)?;
            Vault::create(Path::new(&path), &read_passphrase()?)
        }
        "inspect" => {
            let path = args.next().ok_or(Error::VaultUnlock)?;
            reject_extra_args(&mut args)?;
            let vault = Vault::unlock(Path::new(&path), &read_passphrase()?)?;
            let config = vault.load_config()?;
            println!("{config:?}");
            vault.seal()
        }
        _ => Err(Error::VaultUnlock),
    }
}

fn read_passphrase() -> Result<String, Error> {
    let mut passphrase = String::new();
    std::io::stdin()
        .read_to_string(&mut passphrase)
        .map_err(|_| Error::Io)?;
    Ok(passphrase.trim_end_matches(['\r', '\n']).to_owned())
}

fn reject_extra_args(args: &mut impl Iterator<Item = String>) -> Result<(), Error> {
    if args.next().is_some() {
        Err(Error::VaultUnlock)
    } else {
        Ok(())
    }
}

fn print_help() {
    println!(
        "forge {}\n\nUsage:\n  forge create <vault-image>\n  forge inspect <vault-image>\n\nThe passphrase is read from stdin and is never written to a temporary file. Use image files only; never pass a real block device.",
        adad_core::version()
    );
}
