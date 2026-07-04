use core::fmt;

/// Non-secret configuration fields that may fail validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigField {
    ConfigVersion,
    Provider,
    LocalBaseUrl,
    OpenAiBaseUrl,
    VeniceBaseUrl,
    VeniceAllowAnonymized,
    OpenAiApiKey,
    VeniceApiKey,
    Model,
    DmsWindowHours,
    WgConf,
    MoneroRpcUrl,
    UnknownKey,
}

impl ConfigField {
    #[must_use]
    pub fn as_message_field(self) -> &'static str {
        match self {
            Self::ConfigVersion => "config_version",
            Self::Provider => "provider",
            Self::LocalBaseUrl => "local_base_url",
            Self::OpenAiBaseUrl => "openai_base_url",
            Self::VeniceBaseUrl => "venice_base_url",
            Self::VeniceAllowAnonymized => "venice_allow_anonymized",
            Self::OpenAiApiKey => "openai_api_key",
            Self::VeniceApiKey => "venice_api_key",
            Self::Model => "model",
            Self::DmsWindowHours => "dms_window_hours",
            Self::WgConf => "wg_conf",
            Self::MoneroRpcUrl => "monero_rpc_url",
            Self::UnknownKey => "unknown_key",
        }
    }
}

/// Shared error taxonomy for pure ADAD core logic.
#[derive(Clone, Eq, PartialEq)]
pub enum Error {
    Config { field: ConfigField },
    Identity,
    VaultUnlock,
    VaultVersion,
    Provider,
    EgressBlocked,
    Killswitch,
    WalletRpc,
    VpsProvision,
    GitSpoof,
    Metafuse,
    Io,
}

impl Error {
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config { .. } => 10,
            Self::Identity => 11,
            Self::VaultUnlock => 12,
            Self::VaultVersion => 13,
            Self::Provider => 14,
            Self::EgressBlocked => 15,
            Self::Killswitch => 16,
            Self::WalletRpc => 17,
            Self::VpsProvision => 18,
            Self::GitSpoof => 19,
            Self::Metafuse => 20,
            Self::Io => 21,
        }
    }

    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            Self::Config { field } => {
                format!("Configuration invalid: {}", field.as_message_field())
            }
            Self::Identity => "Session identity error".to_owned(),
            Self::VaultUnlock => "Vault unlock failed".to_owned(),
            Self::VaultVersion => "Vault version incompatible".to_owned(),
            Self::Provider => "AI provider unavailable".to_owned(),
            Self::EgressBlocked => "Blocked: tunnel not active".to_owned(),
            Self::Killswitch => "Network dropped (killswitch)".to_owned(),
            Self::WalletRpc => "Wallet operation failed".to_owned(),
            Self::VpsProvision => "Provisioning failed".to_owned(),
            Self::GitSpoof => "Commit blocked (identity)".to_owned(),
            Self::Metafuse => "Metadata layer error".to_owned(),
            Self::Io => "I/O error".to_owned(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.user_message())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error {{ variant: {}, message: {} }}",
            match self {
                Self::Config { .. } => "Config",
                Self::Identity => "Identity",
                Self::VaultUnlock => "VaultUnlock",
                Self::VaultVersion => "VaultVersion",
                Self::Provider => "Provider",
                Self::EgressBlocked => "EgressBlocked",
                Self::Killswitch => "Killswitch",
                Self::WalletRpc => "WalletRpc",
                Self::VpsProvision => "VpsProvision",
                Self::GitSpoof => "GitSpoof",
                Self::Metafuse => "Metafuse",
                Self::Io => "Io",
            },
            self
        )
    }
}

impl std::error::Error for Error {}
