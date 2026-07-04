use crate::Error;

/// Available inference backends for ADAD.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Provider {
    #[default]
    Local,
    OpenAi,
    Venice,
}

impl Provider {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::OpenAi => "openai",
            Self::Venice => "venice",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, Error> {
        match raw {
            "local" => Ok(Self::Local),
            "openai" => Ok(Self::OpenAi),
            "venice" => Ok(Self::Venice),
            _ => Err(Error::Config {
                field: crate::ConfigField::Provider,
            }),
        }
    }
}
