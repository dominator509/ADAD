use std::process::{Command, Stdio};

use adad_core::EgressSnapshot;

use crate::{TunnelHealth, WireGuardController};

const NFT_COMMAND: &str = "/usr/sbin/nft";
const SYSCTL_COMMAND: &str = "/usr/sbin/sysctl";

/// Machine-readable result of the local egress posture observation.
///
/// `Ready` is intentionally stricter than an active WireGuard interface: the
/// killswitch, DNS/discovery blocks, and IPv6 sysctls must all be observed at
/// the same query boundary before an API fallback may be used.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EgressStatus {
    Ready,
    Blocked,
    Unknown,
}

impl EgressStatus {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Blocked => "blocked",
            Self::Unknown => "unknown",
        }
    }
}

/// Observe the live Linux controls that authorize WireGuard API egress.
///
/// This function never changes network state and never prints command output.
/// An unavailable command is `Unknown`; callers must treat both `Unknown` and
/// `Blocked` as not authorized.
#[must_use]
pub fn system_status() -> EgressStatus {
    let wireguard = WireGuardController::default().status();
    if wireguard == TunnelHealth::Unknown {
        return EgressStatus::Unknown;
    }

    let Some(rules) = command_stdout(NFT_COMMAND, &["list", "table", "inet", "adad_killswitch"])
    else {
        return EgressStatus::Unknown;
    };
    let Some(ipv6_all) = sysctl_disabled("net.ipv6.conf.all.disable_ipv6") else {
        return EgressStatus::Unknown;
    };
    let Some(ipv6_default) = sysctl_disabled("net.ipv6.conf.default.disable_ipv6") else {
        return EgressStatus::Unknown;
    };
    let Some(ipv6_loopback) = sysctl_disabled("net.ipv6.conf.lo.disable_ipv6") else {
        return EgressStatus::Unknown;
    };

    classify(wireguard, &rules, ipv6_all && ipv6_default && ipv6_loopback)
}

fn classify(wireguard: TunnelHealth, rules: &str, ipv6_disabled: bool) -> EgressStatus {
    let output_chain = chain_body(rules, "output").unwrap_or("");
    let firewall_ready = contains_all(
        output_chain,
        &[
            "type filter hook output",
            "policy drop",
            "oifname \"lo\" accept",
            "oifname \"wg0\" accept",
        ],
    ) && rules.contains("table inet adad_killswitch");
    let direct_dns_blocked = output_chain.contains("udp dport { 53, 5353, 1900, 137, 138 } drop");
    let discovery_blocked =
        direct_dns_blocked && output_chain.contains("tcp dport { 139, 445 } drop");
    let snapshot = EgressSnapshot::new(
        wireguard == TunnelHealth::Active,
        firewall_ready,
        direct_dns_blocked,
        ipv6_disabled,
        discovery_blocked,
    );

    if snapshot.leak_free_fallback_ready() {
        EgressStatus::Ready
    } else {
        EgressStatus::Blocked
    }
}

fn contains_all(value: &str, required: &[&str]) -> bool {
    required.iter().all(|needle| value.contains(needle))
}

fn chain_body<'a>(rules: &'a str, chain_name: &str) -> Option<&'a str> {
    let marker = format!("chain {chain_name} {{");
    let marker_start = rules.find(&marker)?;
    let body_start = marker_start + marker.len();
    let bytes = rules.as_bytes();
    let mut depth = 1usize;
    let mut in_quote = false;
    let mut escaped = false;

    for (offset, byte) in bytes[body_start..].iter().copied().enumerate() {
        if in_quote {
            if byte == b'\\' && !escaped {
                escaped = true;
                continue;
            }
            if byte == b'"' && !escaped {
                in_quote = false;
            }
            escaped = false;
            continue;
        }

        match byte {
            b'"' => in_quote = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&rules[body_start..body_start + offset]);
                }
            }
            _ => {}
        }
    }

    None
}

fn command_stdout(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

fn sysctl_disabled(key: &str) -> Option<bool> {
    let value = command_stdout(SYSCTL_COMMAND, &["-n", key])?;
    Some(value.trim() == "1")
}

#[cfg(test)]
mod tests {
    use super::{classify, EgressStatus};
    use crate::TunnelHealth;

    const READY_RULES: &str = r#"
table inet adad_killswitch {
  chain output {
    type filter hook output priority 0; policy drop;
    oifname "lo" accept
    oifname "wg0" accept
    udp dport { 53, 5353, 1900, 137, 138 } drop
    tcp dport { 139, 445 } drop
  }
}
"#;

    const CONTROLS_IN_INPUT_CHAIN: &str = r#"
table inet adad_killswitch {
  chain input {
    type filter hook input priority 0; policy drop;
    oifname "lo" accept
    oifname "wg0" accept
    udp dport { 53, 5353, 1900, 137, 138 } drop
    tcp dport { 139, 445 } drop
  }
  chain output {
    type filter hook output priority 0; policy accept;
  }
}
"#;

    #[test]
    fn ready_requires_the_wireguard_and_all_drop_controls() {
        assert_eq!(
            classify(TunnelHealth::Active, READY_RULES, true),
            EgressStatus::Ready
        );
        assert_eq!(
            classify(TunnelHealth::Inactive, READY_RULES, true),
            EgressStatus::Blocked
        );
        assert_eq!(
            classify(
                TunnelHealth::Active,
                &READY_RULES.replace("policy drop", "policy accept"),
                true
            ),
            EgressStatus::Blocked
        );
        assert_eq!(
            classify(TunnelHealth::Active, READY_RULES, false),
            EgressStatus::Blocked
        );
    }

    #[test]
    fn missing_rule_cannot_be_treated_as_ready() {
        let rules = READY_RULES.replace("tcp dport { 139, 445 } drop", "");
        assert_eq!(
            classify(TunnelHealth::Active, &rules, true),
            EgressStatus::Blocked
        );
    }

    #[test]
    fn output_controls_cannot_be_satisfied_by_another_chain() {
        assert_eq!(
            classify(TunnelHealth::Active, CONTROLS_IN_INPUT_CHAIN, true),
            EgressStatus::Blocked
        );
    }
}
