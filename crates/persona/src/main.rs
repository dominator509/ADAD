use std::env;
use std::path::Path;

use adad_core::Error;
use persona::PersonaStore;

fn main() {
    if let Err(error) = run() {
        eprintln!("persona: {error}");
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
            println!("persona {}", adad_core::version());
            Ok(())
        }
        "load" => {
            let root = args.next().ok_or(Error::Identity)?;
            if args.next().is_some() {
                return Err(Error::Identity);
            }
            let identity = PersonaStore::load(Path::new(&root))?;
            println!("{identity:?}");
            Ok(())
        }
        _ => Err(Error::Identity),
    }
}

fn print_help() {
    println!(
        "persona {}\n\nUsage:\n  persona load <vault-root>\n\nLoads the session identity from the vault layout. Identity values are redacted by the displayed debug representation.",
        adad_core::version()
    );
}
