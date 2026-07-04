use std::fs;
use std::path::{Path, PathBuf};

use adad_core::{Error, SessionIdentity};

const IDENTITY_RELATIVE_PATH: &str = "identity/session-identity.toml";

pub struct PersonaStore;

impl PersonaStore {
    pub fn save(vault_root: &Path, identity: &SessionIdentity) -> Result<(), Error> {
        let identity_path = identity_path(vault_root);
        if let Some(parent) = identity_path.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        fs::write(identity_path, serialize(identity).as_bytes()).map_err(io_error)
    }

    pub fn load(vault_root: &Path) -> Result<SessionIdentity, Error> {
        let raw = fs::read_to_string(identity_path(vault_root)).map_err(io_error)?;
        deserialize(&raw)
    }
}

fn identity_path(vault_root: &Path) -> PathBuf {
    vault_root.join(IDENTITY_RELATIVE_PATH)
}

fn serialize(identity: &SessionIdentity) -> String {
    let mut lines = vec![
        format!(
            "pseudonym = \"{}\"",
            escape_toml_string(identity.pseudonym())
        ),
        format!(
            "git_author_name = \"{}\"",
            escape_toml_string(identity.git_author_name())
        ),
        format!(
            "git_author_email = \"{}\"",
            escape_toml_string(identity.git_author_email())
        ),
    ];

    if let Some(onion) = identity.forgejo_onion_service() {
        lines.push(format!(
            "forgejo_onion_service = \"{}\"",
            escape_toml_string(onion)
        ));
    }

    lines.join("\n") + "\n"
}

fn deserialize(raw: &str) -> Result<SessionIdentity, Error> {
    let mut pseudonym = None;
    let mut git_author_name = None;
    let mut git_author_email = None;
    let mut forgejo_onion_service = None;

    for raw_line in raw.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value) = line.split_once('=').ok_or(Error::Identity)?;
        let value = parse_string(value.trim())?;

        match key.trim() {
            "pseudonym" => pseudonym = Some(value),
            "git_author_name" => git_author_name = Some(value),
            "git_author_email" => git_author_email = Some(value),
            "forgejo_onion_service" => forgejo_onion_service = Some(value),
            _ => return Err(Error::Identity),
        }
    }

    SessionIdentity::new(
        pseudonym.ok_or(Error::Identity)?,
        git_author_name.ok_or(Error::Identity)?,
        git_author_email.ok_or(Error::Identity)?,
        forgejo_onion_service,
    )
}

fn parse_string(raw: &str) -> Result<String, Error> {
    if raw.len() < 2 || !raw.starts_with('"') || !raw.ends_with('"') {
        return Err(Error::Identity);
    }

    let inner = &raw[1..raw.len() - 1];
    let mut value = String::with_capacity(inner.len());
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let escaped = chars.next().ok_or(Error::Identity)?;
            match escaped {
                '\\' | '"' => value.push(escaped),
                _ => return Err(Error::Identity),
            }
        } else {
            value.push(ch);
        }
    }
    Ok(value)
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn io_error(_: std::io::Error) -> Error {
    Error::Io
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use adad_core::SessionIdentity;

    use super::PersonaStore;

    static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn identity_round_trips_through_the_vault_layout() {
        let root = temp_root();
        let identity = SessionIdentity::new(
            "aurora",
            "Aurora Maintainer",
            "aurora@adad.invalid",
            Some("auroraaaaaaaaaaaaaaaaa.onion".to_string()),
        )
        .expect("valid identity");

        PersonaStore::save(&root, &identity).expect("identity saves");
        let loaded = PersonaStore::load(&root).expect("identity loads");

        assert_eq!(loaded, identity);
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn identity_debug_output_stays_redacted_after_load() {
        let root = temp_root();
        let identity = SessionIdentity::new(
            "lumen",
            "Lumen Operator",
            "lumen@adad.invalid",
            Some("lumenbbbbbbbbbbbbbbbbbbbb.onion".to_string()),
        )
        .expect("valid identity");

        PersonaStore::save(&root, &identity).expect("identity saves");
        let loaded = PersonaStore::load(&root).expect("identity loads");
        let debug = format!("{loaded:?}");

        assert!(!debug.contains("lumen"));
        assert!(!debug.contains("Operator"));
        assert!(!debug.contains("adad.invalid"));
        assert!(debug.contains("[REDACTED]"));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn invalid_identity_payload_fails_cleanly() {
        let root = temp_root();
        let path = root.join("identity").join("session-identity.toml");
        fs::create_dir_all(path.parent().expect("identity parent")).expect("create identity dir");
        fs::write(&path, "pseudonym = \"\"\n").expect("write invalid payload");

        assert_eq!(PersonaStore::load(&root), Err(adad_core::Error::Identity));
        fs::remove_dir_all(root).ok();
    }

    fn temp_root() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);
        env::temp_dir().join(format!("adad-persona-store-{nanos}-{counter}"))
    }
}
