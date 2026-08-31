use std::env;
use std::path::Path;

use adad_core::{Error, SessionIdentity};
use git_spoof::{commit, rewrite, CommitMetadata};

fn main() {
    if let Err(error) = run() {
        eprintln!("git-spoof: {error}");
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
            println!("git-spoof {}", adad_core::version());
            Ok(())
        }
        "rewrite-metadata" => {
            let message = args.collect::<Vec<_>>().join(" ");
            if message.trim().is_empty() {
                return Err(Error::GitSpoof);
            }
            let identity = identity_from_environment()?;
            let raw = CommitMetadata::new(
                "untrusted",
                "untrusted@invalid",
                "unknown",
                "untrusted",
                "untrusted@invalid",
                "unknown",
                message,
            );
            let rewritten = rewrite(&raw, &identity)?;
            println!("{}", rewritten.rendered_identity_fields());
            Ok(())
        }
        "commit" => {
            let message = args.collect::<Vec<_>>().join(" ");
            let identity = identity_from_environment()?;
            let hash = commit(Path::new("."), &message, &identity)?;
            println!("git-spoof committed {hash}");
            Ok(())
        }
        _ => Err(Error::GitSpoof),
    }
}

fn identity_from_environment() -> Result<SessionIdentity, Error> {
    SessionIdentity::new(
        env::var("ADAD_PSEUDONYM").map_err(|_| Error::Identity)?,
        env::var("ADAD_GIT_AUTHOR_NAME").map_err(|_| Error::Identity)?,
        env::var("ADAD_GIT_AUTHOR_EMAIL").map_err(|_| Error::Identity)?,
        env::var("ADAD_FORGEJO_ONION_SERVICE").ok(),
    )
}

fn print_help() {
    println!(
        "git-spoof {}\n\nUsage:\n  git-spoof commit <commit-message>\n  git-spoof rewrite-metadata <commit-message>\n\nThe commit command uses the stable identity from ADAD_PSEUDONYM, ADAD_GIT_AUTHOR_NAME, and ADAD_GIT_AUTHOR_EMAIL, with fixed UTC timestamps. Changes must be staged first. The rewrite-metadata command only renders the transformed fields.",
        adad_core::version()
    );
}
