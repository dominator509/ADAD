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
