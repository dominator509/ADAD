use adad_core::{Error, SessionIdentity};

pub const FIXED_UTC_TIMESTAMP: &str = "2000-01-01T00:00:00Z";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommitMetadata {
    pub author_name: String,
    pub author_email: String,
    pub author_timestamp: String,
    pub committer_name: String,
    pub committer_email: String,
    pub committer_timestamp: String,
    pub message: String,
}

impl CommitMetadata {
    #[must_use]
    pub fn new(
        author_name: impl Into<String>,
        author_email: impl Into<String>,
        author_timestamp: impl Into<String>,
        committer_name: impl Into<String>,
        committer_email: impl Into<String>,
        committer_timestamp: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            author_name: author_name.into(),
            author_email: author_email.into(),
            author_timestamp: author_timestamp.into(),
            committer_name: committer_name.into(),
            committer_email: committer_email.into(),
            committer_timestamp: committer_timestamp.into(),
            message: message.into(),
        }
    }

    #[must_use]
    pub fn rendered_identity_fields(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            self.author_name,
            self.author_email,
            self.author_timestamp,
            self.committer_name,
            self.committer_email,
            self.committer_timestamp
        )
    }
}

pub fn rewrite(
    commit: &CommitMetadata,
    identity: &SessionIdentity,
) -> Result<CommitMetadata, Error> {
    if identity.git_author_name().trim().is_empty() || identity.git_author_email().trim().is_empty()
    {
        return Err(Error::GitSpoof);
    }

    Ok(CommitMetadata {
        author_name: identity.git_author_name().to_owned(),
        author_email: identity.git_author_email().to_owned(),
        author_timestamp: FIXED_UTC_TIMESTAMP.to_owned(),
        committer_name: identity.git_author_name().to_owned(),
        committer_email: identity.git_author_email().to_owned(),
        committer_timestamp: FIXED_UTC_TIMESTAMP.to_owned(),
        message: commit.message.clone(),
    })
}

#[cfg(test)]
mod tests {
    use adad_core::SessionIdentity;

    use super::{rewrite, CommitMetadata, FIXED_UTC_TIMESTAMP};

    #[test]
    fn rewrite_sets_author_committer_and_fixed_utc_timestamp() {
        let identity = test_identity();
        let raw = raw_commit("Local User", "local@example.com", "-0700");

        let rewritten = rewrite(&raw, &identity).expect("rewrite should succeed");

        assert_eq!(rewritten.author_name, "Stable Persona");
        assert_eq!(rewritten.author_email, "stable@example.invalid");
        assert_eq!(rewritten.committer_name, "Stable Persona");
        assert_eq!(rewritten.committer_email, "stable@example.invalid");
        assert_eq!(rewritten.author_timestamp, FIXED_UTC_TIMESTAMP);
        assert_eq!(rewritten.committer_timestamp, FIXED_UTC_TIMESTAMP);
    }

    #[test]
    fn rewrite_strips_real_metadata_from_identity_fields() {
        let identity = test_identity();
        let raw = raw_commit("Real Name", "real@example.com", "-0700");

        let rendered = rewrite(&raw, &identity)
            .expect("rewrite should succeed")
            .rendered_identity_fields();

        assert!(!rendered.contains("Real Name"));
        assert!(!rendered.contains("real@example.com"));
        assert!(!rendered.contains("-0700"));
    }

    fn test_identity() -> SessionIdentity {
        SessionIdentity::new(
            "adad-user",
            "Stable Persona",
            "stable@example.invalid",
            None,
        )
        .expect("identity should be valid")
    }

    fn raw_commit(name: &str, email: &str, tz: &str) -> CommitMetadata {
        CommitMetadata::new(
            name,
            email,
            format!("2026-07-03T21:00:00{tz}"),
            name,
            email,
            format!("2026-07-03T21:00:00{tz}"),
            "commit message",
        )
    }
}
