use adad_core::{ConfigField, Error};

#[test]
fn each_variant_has_a_stable_message_and_exit_code() {
    let cases = [
        (
            Error::Config {
                field: ConfigField::Provider,
            },
            "Configuration invalid: provider",
            10,
        ),
        (Error::Identity, "Session identity error", 11),
        (Error::VaultUnlock, "Vault unlock failed", 12),
        (Error::VaultVersion, "Vault version incompatible", 13),
        (Error::Provider, "AI provider unavailable", 14),
        (Error::EgressBlocked, "Blocked: tunnel not active", 15),
        (Error::Killswitch, "Network dropped (killswitch)", 16),
        (Error::WalletRpc, "Wallet operation failed", 17),
        (Error::VpsProvision, "Provisioning failed", 18),
        (Error::GitSpoof, "Commit blocked (identity)", 19),
        (Error::Metafuse, "Metadata layer error", 20),
        (Error::Io, "I/O error", 21),
    ];

    for (error, message, code) in cases {
        assert_eq!(error.to_string(), message);
        assert_eq!(error.user_message(), message);
        assert_eq!(error.exit_code(), code);
    }
}

#[test]
fn rendered_errors_never_echo_secret_like_strings() {
    let probes = [
        "sk-secret-123",
        "real.person@example.com",
        "abcdefghijklmnop.onion",
        "wireguard-private-key",
        "monero-wallet-seed",
    ];
    let variants = [
        Error::Config {
            field: ConfigField::UnknownKey,
        },
        Error::Identity,
        Error::VaultUnlock,
        Error::VaultVersion,
        Error::Provider,
        Error::EgressBlocked,
        Error::Killswitch,
        Error::WalletRpc,
        Error::VpsProvision,
        Error::GitSpoof,
        Error::Metafuse,
        Error::Io,
    ];

    let rendered = variants
        .iter()
        .flat_map(|error| [format!("{error:?}"), error.to_string()])
        .collect::<Vec<_>>()
        .join("\n");

    for probe in probes {
        assert!(!rendered.contains(probe), "rendered error leaked {probe}");
    }
}
