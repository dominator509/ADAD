use adad_core::{Config, Error, Provider};

#[test]
fn parses_and_validates_a_positive_config() {
    let config = Config::from_toml_str(
        r#"
config_version = 1
provider = "local"
local_base_url = "http://127.0.0.1:8080/v1"
model = "qwen2.5-coder"
dms_window_hours = 72
"#,
    )
    .expect("config should parse");

    assert_eq!(config.config_version, 1);
    assert_eq!(config.provider, Provider::Local);
}

#[test]
fn rejects_unknown_keys() {
    let error = Config::from_toml_str(
        r#"
config_version = 1
provider = "local"
mystery = true
"#,
    )
    .expect_err("unknown key must fail");

    assert_eq!(
        error,
        Error::Config {
            field: adad_core::ConfigField::UnknownKey,
        }
    );
}

#[test]
fn rejects_invalid_venice_anonymized_combo() {
    let error = Config::from_toml_str(
        r#"
config_version = 1
provider = "venice"
model = "venice-anonymized-large"
venice_allow_anonymized = false
venice_base_url = "https://api.venice.ai/api/v1"
"#,
    )
    .expect_err("venice anonymized should require opt-in");

    assert_eq!(
        error,
        Error::Config {
            field: adad_core::ConfigField::VeniceAllowAnonymized,
        }
    );
}

#[test]
fn parses_from_bytes() {
    let config = Config::from_bytes(
        br#"
config_version = 1
provider = "openai"
openai_base_url = "https://api.example.com/v1"
"#,
    )
    .expect("bytes should parse");

    assert_eq!(config.provider, Provider::OpenAi);
}

#[test]
fn decodes_basic_string_escapes_symmetrically() {
    let config = Config::from_toml_str(
        r#"
config_version = 2
provider = "local"
model = "line\nquote\"slash\\"
"#,
    )
    .expect("escaped config should parse");

    assert_eq!(config.model.as_deref(), Some("line\nquote\"slash\\"));
}

#[test]
fn rejects_unknown_and_incomplete_string_escapes() {
    for model in [r#""bad\q""#, r#""bad\""#] {
        let input = format!("config_version = 2\nprovider = \"local\"\nmodel = {model}\n");
        assert!(Config::from_toml_str(&input).is_err(), "input: {input}");
    }
}
