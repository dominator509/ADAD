use core::fmt;

use crate::{ConfigField, Error, Provider};

/// Redacted secret wrapper for config values loaded from the vault.
#[derive(Clone, Eq, PartialEq)]
pub struct SecretString(String);

impl SecretString {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

/// Shared ADAD runtime configuration, loaded from vault-backed text.
#[derive(Clone, Eq, PartialEq)]
pub struct Config {
    pub config_version: u32,
    pub provider: Provider,
    pub local_base_url: Option<String>,
    pub openai_base_url: Option<String>,
    pub venice_base_url: Option<String>,
    pub venice_allow_anonymized: bool,
    pub openai_api_key: Option<SecretString>,
    pub venice_api_key: Option<SecretString>,
    pub model: Option<String>,
    pub dms_window_hours: Option<u32>,
    pub wg_conf: Option<SecretString>,
    pub monero_rpc_url: Option<String>,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("config_version", &self.config_version)
            .field("provider", &self.provider)
            .field("local_base_url", &self.local_base_url)
            .field("openai_base_url", &self.openai_base_url)
            .field("venice_base_url", &self.venice_base_url)
            .field("venice_allow_anonymized", &self.venice_allow_anonymized)
            .field("openai_api_key", &self.openai_api_key)
            .field("venice_api_key", &self.venice_api_key)
            .field("model", &self.model)
            .field("dms_window_hours", &self.dms_window_hours)
            .field("wg_conf", &self.wg_conf)
            .field("monero_rpc_url", &self.monero_rpc_url)
            .finish()
    }
}

impl Config {
    pub fn from_toml_str(input: &str) -> Result<Self, Error> {
        let mut config_version = None;
        let mut provider = None;
        let mut local_base_url = None;
        let mut openai_base_url = None;
        let mut venice_base_url = None;
        let mut venice_allow_anonymized = None;
        let mut openai_api_key = None;
        let mut venice_api_key = None;
        let mut model = None;
        let mut dms_window_hours = None;
        let mut wg_conf = None;
        let mut monero_rpc_url = None;

        for raw_line in input.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (key, raw_value) = line.split_once('=').ok_or(Error::Config {
                field: ConfigField::UnknownKey,
            })?;
            let key = key.trim();
            let raw_value = raw_value.trim();

            match key {
                "config_version" => {
                    config_version = Some(parse_u32(raw_value, ConfigField::ConfigVersion)?);
                }
                "provider" => {
                    provider = Some(Provider::parse(&parse_string(
                        raw_value,
                        ConfigField::Provider,
                    )?)?);
                }
                "local_base_url" => {
                    local_base_url = Some(parse_string(raw_value, ConfigField::LocalBaseUrl)?);
                }
                "openai_base_url" => {
                    openai_base_url = Some(parse_string(raw_value, ConfigField::OpenAiBaseUrl)?);
                }
                "venice_base_url" => {
                    venice_base_url = Some(parse_string(raw_value, ConfigField::VeniceBaseUrl)?);
                }
                "venice_allow_anonymized" => {
                    venice_allow_anonymized =
                        Some(parse_bool(raw_value, ConfigField::VeniceAllowAnonymized)?);
                }
                "openai_api_key" => {
                    openai_api_key = Some(SecretString::new(parse_string(
                        raw_value,
                        ConfigField::OpenAiApiKey,
                    )?));
                }
                "venice_api_key" => {
                    venice_api_key = Some(SecretString::new(parse_string(
                        raw_value,
                        ConfigField::VeniceApiKey,
                    )?));
                }
                "model" => {
                    model = Some(parse_string(raw_value, ConfigField::Model)?);
                }
                "dms_window_hours" => {
                    dms_window_hours = Some(parse_u32(raw_value, ConfigField::DmsWindowHours)?);
                }
                "wg_conf" => {
                    wg_conf = Some(SecretString::new(parse_string(
                        raw_value,
                        ConfigField::WgConf,
                    )?));
                }
                "monero_rpc_url" => {
                    monero_rpc_url = Some(parse_string(raw_value, ConfigField::MoneroRpcUrl)?);
                }
                _ => {
                    return Err(Error::Config {
                        field: ConfigField::UnknownKey,
                    });
                }
            }
        }

        let config = Self {
            config_version: config_version.unwrap_or_default(),
            provider: provider.unwrap_or_default(),
            local_base_url,
            openai_base_url,
            venice_base_url,
            venice_allow_anonymized: venice_allow_anonymized.unwrap_or(false),
            openai_api_key,
            venice_api_key,
            model,
            dms_window_hours,
            wg_conf,
            monero_rpc_url,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn from_bytes(input: &[u8]) -> Result<Self, Error> {
        let text = core::str::from_utf8(input).map_err(|_| Error::Config {
            field: ConfigField::UnknownKey,
        })?;
        Self::from_toml_str(text)
    }

    pub fn validate(&self) -> Result<(), Error> {
        if self.config_version == 0 {
            return Err(Error::Config {
                field: ConfigField::ConfigVersion,
            });
        }

        if let Some(model) = &self.model {
            if model.trim().is_empty() {
                return Err(Error::Config {
                    field: ConfigField::Model,
                });
            }
            if self.provider == Provider::Venice
                && model.to_ascii_lowercase().contains("anonymized")
                && !self.venice_allow_anonymized
            {
                return Err(Error::Config {
                    field: ConfigField::VeniceAllowAnonymized,
                });
            }
        }

        if let Some(hours) = self.dms_window_hours {
            if hours == 0 {
                return Err(Error::Config {
                    field: ConfigField::DmsWindowHours,
                });
            }
        }

        validate_url(
            self.local_base_url.as_deref(),
            ConfigField::LocalBaseUrl,
            UrlPolicy::LoopbackHttp,
        )?;
        validate_url(
            self.openai_base_url.as_deref(),
            ConfigField::OpenAiBaseUrl,
            UrlPolicy::Https,
        )?;
        validate_url(
            self.venice_base_url.as_deref(),
            ConfigField::VeniceBaseUrl,
            UrlPolicy::Https,
        )?;

        Ok(())
    }
}

fn parse_string(raw: &str, field: ConfigField) -> Result<String, Error> {
    if raw.len() < 2 || !raw.starts_with('"') || !raw.ends_with('"') {
        return Err(Error::Config { field });
    }

    let mut parsed = String::with_capacity(raw.len() - 2);
    let mut chars = raw[1..raw.len() - 1].chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            parsed.push(ch);
            continue;
        }

        let escaped = chars.next().ok_or(Error::Config { field })?;
        match escaped {
            'b' => parsed.push('\u{0008}'),
            't' => parsed.push('\t'),
            'n' => parsed.push('\n'),
            'f' => parsed.push('\u{000C}'),
            'r' => parsed.push('\r'),
            '"' => parsed.push('"'),
            '\\' => parsed.push('\\'),
            'u' => parsed.push(parse_unicode_escape(&mut chars, 4, field)?),
            'U' => parsed.push(parse_unicode_escape(&mut chars, 8, field)?),
            _ => return Err(Error::Config { field }),
        }
    }

    Ok(parsed)
}

fn parse_unicode_escape(
    chars: &mut impl Iterator<Item = char>,
    digits: usize,
    field: ConfigField,
) -> Result<char, Error> {
    let mut value = 0_u32;
    for _ in 0..digits {
        let digit = chars.next().and_then(|ch| ch.to_digit(16));
        value = value
            .checked_mul(16)
            .and_then(|value| value.checked_add(digit?))
            .ok_or(Error::Config { field })?;
    }

    char::from_u32(value).ok_or(Error::Config { field })
}

fn parse_bool(raw: &str, field: ConfigField) -> Result<bool, Error> {
    match raw {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(Error::Config { field }),
    }
}

fn parse_u32(raw: &str, field: ConfigField) -> Result<u32, Error> {
    raw.parse::<u32>().map_err(|_| Error::Config { field })
}

#[derive(Clone, Copy)]
enum UrlPolicy {
    Https,
    LoopbackHttp,
}

fn validate_url(value: Option<&str>, field: ConfigField, policy: UrlPolicy) -> Result<(), Error> {
    let Some(raw) = value else {
        return Ok(());
    };

    let valid = match policy {
        UrlPolicy::Https => raw.starts_with("https://"),
        UrlPolicy::LoopbackHttp => {
            raw.starts_with("http://127.0.0.1") || raw.starts_with("http://localhost")
        }
    };

    if valid {
        Ok(())
    } else {
        Err(Error::Config { field })
    }
}
