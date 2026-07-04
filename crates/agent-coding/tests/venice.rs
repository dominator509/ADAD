use adad_core::{Config, ConfigField, Error, Provider};
use agent_coding::{
    provider_select, ProviderWarning, DEFAULT_LOCAL_BASE_URL, DEFAULT_LOCAL_MODEL,
    DEFAULT_VENICE_BASE_URL,
};

#[test]
fn default_provider_is_local_with_loopback_model_defaults() {
    let config = Config::from_toml_str("config_version = 1").expect("minimal config is valid");

    let selection = provider_select(&config).expect("local provider should select");

    assert_eq!(selection.provider, Provider::Local);
    assert_eq!(selection.base_url, DEFAULT_LOCAL_BASE_URL);
    assert_eq!(selection.model, DEFAULT_LOCAL_MODEL);
    assert_eq!(selection.api_key, "");
    assert!(selection.warnings.is_empty());
}

#[test]
fn venice_private_model_is_allowed_by_default() {
    let config = Config::from_toml_str(
        r#"
config_version = 1
provider = "venice"
model = "venice-private-coder"
"#,
    )
    .expect("private Venice model config is valid");

    let selection = provider_select(&config).expect("private Venice model should select");

    assert_eq!(selection.provider, Provider::Venice);
    assert_eq!(selection.base_url, DEFAULT_VENICE_BASE_URL);
    assert_eq!(selection.model, "venice-private-coder");
    assert!(selection.warnings.is_empty());
}

#[test]
fn venice_anonymized_model_requires_explicit_opt_in() {
    let config = venice_anonymized_config(false);

    let error = provider_select(&config).expect_err("anonymized Venice model must be gated");

    assert_eq!(
        error,
        Error::Config {
            field: ConfigField::VeniceAllowAnonymized,
        }
    );
}

#[test]
fn venice_anonymized_opt_in_returns_warning_marker() {
    let config = venice_anonymized_config(true);

    let selection = provider_select(&config).expect("explicit anonymized opt-in should select");

    assert_eq!(selection.model, "venice-anonymized-large");
    assert_eq!(
        selection.warnings,
        vec![ProviderWarning::VeniceAnonymizedModelEnabled]
    );
}

fn venice_anonymized_config(allow: bool) -> Config {
    Config {
        config_version: 1,
        provider: Provider::Venice,
        local_base_url: None,
        openai_base_url: None,
        venice_base_url: None,
        venice_allow_anonymized: allow,
        openai_api_key: None,
        venice_api_key: None,
        model: Some("venice-anonymized-large".to_owned()),
        dms_window_hours: None,
        wg_conf: None,
        monero_rpc_url: None,
    }
}
