use adad_core::{Config, ConfigField, Error, Provider};

pub const DEFAULT_LOCAL_BASE_URL: &str = "http://127.0.0.1:8080/v1";
pub const DEFAULT_LOCAL_MODEL: &str = "qwen2.5-coder";
pub const DEFAULT_VENICE_BASE_URL: &str = "https://api.venice.ai/api/v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSelection {
    pub provider: Provider,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    pub warnings: Vec<ProviderWarning>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProviderWarning {
    VeniceAnonymizedModelEnabled,
}

pub fn provider_select(config: &Config) -> Result<ProviderSelection, Error> {
    match config.provider {
        Provider::Local => Ok(ProviderSelection {
            provider: Provider::Local,
            base_url: config
                .local_base_url
                .clone()
                .unwrap_or_else(|| DEFAULT_LOCAL_BASE_URL.to_owned()),
            api_key: String::new(),
            model: config
                .model
                .clone()
                .unwrap_or_else(|| DEFAULT_LOCAL_MODEL.to_owned()),
            warnings: Vec::new(),
        }),
        Provider::OpenAi => Ok(ProviderSelection {
            provider: Provider::OpenAi,
            base_url: config.openai_base_url.clone().ok_or(Error::Config {
                field: ConfigField::OpenAiBaseUrl,
            })?,
            api_key: config
                .openai_api_key
                .as_ref()
                .map(|secret| secret.expose().to_owned())
                .unwrap_or_default(),
            model: config.model.clone().ok_or(Error::Config {
                field: ConfigField::Model,
            })?,
            warnings: Vec::new(),
        }),
        Provider::Venice => {
            let model = config.model.clone().ok_or(Error::Config {
                field: ConfigField::Model,
            })?;
            let anonymized = model.to_ascii_lowercase().contains("anonymized");
            if anonymized && !config.venice_allow_anonymized {
                return Err(Error::Config {
                    field: ConfigField::VeniceAllowAnonymized,
                });
            }

            Ok(ProviderSelection {
                provider: Provider::Venice,
                base_url: config
                    .venice_base_url
                    .clone()
                    .unwrap_or_else(|| DEFAULT_VENICE_BASE_URL.to_owned()),
                api_key: config
                    .venice_api_key
                    .as_ref()
                    .map(|secret| secret.expose().to_owned())
                    .unwrap_or_default(),
                model,
                warnings: if anonymized {
                    vec![ProviderWarning::VeniceAnonymizedModelEnabled]
                } else {
                    Vec::new()
                },
            })
        }
    }
}
