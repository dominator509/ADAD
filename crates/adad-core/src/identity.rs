use core::fmt;

use crate::Error;

/// Stable per-session pseudonymous identity shared across metadata-sensitive tools.
#[derive(Clone, Eq, PartialEq)]
pub struct SessionIdentity {
    pseudonym: String,
    git_author_name: String,
    git_author_email: String,
    forgejo_onion_service: Option<String>,
}

impl SessionIdentity {
    pub fn new(
        pseudonym: impl Into<String>,
        git_author_name: impl Into<String>,
        git_author_email: impl Into<String>,
        forgejo_onion_service: Option<String>,
    ) -> Result<Self, Error> {
        let pseudonym = pseudonym.into();
        let git_author_name = git_author_name.into();
        let git_author_email = git_author_email.into();

        if pseudonym.trim().is_empty()
            || git_author_name.trim().is_empty()
            || git_author_email.trim().is_empty()
            || !git_author_email.contains('@')
            || git_author_email.contains(char::is_whitespace)
            || has_control_chars(&pseudonym)
            || has_control_chars(&git_author_name)
            || has_control_chars(&git_author_email)
            || forgejo_onion_service
                .as_deref()
                .is_some_and(has_control_chars)
        {
            return Err(Error::Identity);
        }

        Ok(Self {
            pseudonym,
            git_author_name,
            git_author_email,
            forgejo_onion_service,
        })
    }

    #[must_use]
    pub fn pseudonym(&self) -> &str {
        &self.pseudonym
    }

    #[must_use]
    pub fn git_author_name(&self) -> &str {
        &self.git_author_name
    }

    #[must_use]
    pub fn git_author_email(&self) -> &str {
        &self.git_author_email
    }

    #[must_use]
    pub fn forgejo_onion_service(&self) -> Option<&str> {
        self.forgejo_onion_service.as_deref()
    }
}

impl fmt::Debug for SessionIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionIdentity")
            .field("pseudonym", &"[REDACTED]")
            .field("git_author_name", &"[REDACTED]")
            .field("git_author_email", &"[REDACTED]")
            .field(
                "forgejo_onion_service",
                &self.forgejo_onion_service.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}

fn has_control_chars(value: &str) -> bool {
    value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::SessionIdentity;
    use crate::Error;

    #[test]
    fn control_characters_are_rejected_in_identity_fields() {
        assert_eq!(
            SessionIdentity::new("bad\nname", "Name", "a@b.invalid", None),
            Err(Error::Identity)
        );
        assert_eq!(
            SessionIdentity::new("ok", "bad\tescape", "a@b.invalid", None),
            Err(Error::Identity)
        );
        assert_eq!(
            SessionIdentity::new("ok", "Name", "a@b.invalid", Some("bad\x1bonion".to_owned())),
            Err(Error::Identity)
        );
    }

    #[test]
    fn clean_identity_still_constructs() {
        assert!(
            SessionIdentity::new("aurora", "Aurora Maintainer", "aurora@adad.invalid", None)
                .is_ok()
        );
    }
}
