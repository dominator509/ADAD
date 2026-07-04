# SPEC-006 — Error Handling

- **Status:** active
- **Owner:** architect
- **Roadmap phase:** Phases 1–5 (cross-cutting)
- **Linked ExecPlans:** EP-002 (taxonomy), EP-004, EP-006, EP-007

## User-visible goal
Errors are typed, redacted, and actionable; failures fail closed (safe), and no
error ever leaks a secret, real identity, or full onion address.

## Non-goals
No stack traces or raw provider payloads shown to the user; no error that
weakens the security posture as a "recovery."

## Terms
- **Fail-closed:** on ambiguity or failure in an egress/security path, drop/deny
  rather than proceed.

## Required behavior
- `adad-core` MUST define a single `Error` enum (taxonomy below); all library
  functions return `Result<_, Error>`.
- Binaries MUST map `Error` to stable exit codes and redacted user messages.
- Security/egress errors MUST fail closed: e.g. `EgressBlocked` means the request
  was NOT sent; `VaultUnlock` leaves nothing mounted.
- No error `Display`/`Debug` output MUST contain a secret, key, passphrase, real
  identity field, or full onion address (truncated hash allowed).

## Error taxonomy (adad-core::Error)
| Variant | Meaning | User message (redacted) | Fail mode |
|---|---|---|---|
| `Config` | invalid/unknown config | "Configuration invalid: <field>" | refuse to run |
| `Identity` | bad session identity | "Session identity error" | refuse |
| `VaultUnlock` | wrong passphrase / locked | "Vault unlock failed" | nothing mounted |
| `VaultVersion` | incompatible vault version | "Vault version incompatible" | refuse; suggest upgrade |
| `Provider` | inference backend error | "AI provider unavailable" | no partial output |
| `EgressBlocked` | tunnel not active for fallback | "Blocked: tunnel not active" | request not sent |
| `Killswitch` | posture forced drop | "Network dropped (killswitch)" | all egress dropped |
| `WalletRpc` | monero-wallet-rpc failure | "Wallet operation failed" | no state change |
| `VpsProvision` | SSH/setup failure | "Provisioning failed" | no partial infra claimed |
| `GitSpoof` | commit rewrite failure | "Commit blocked (identity)" | commit not made |
| `Metafuse` | FUSE metadata error | "Metadata layer error" | file op refused |
| `Io` | generic I/O within boundary | "I/O error" | operation aborted |

## Inputs
Any failing operation across crates.

## Outputs
Typed `Error`; redacted message; stable exit code (binaries map variant→code).

## Error states
Retry behavior: transient provider/RPC errors MAY retry with bounded backoff
(client-side); security/egress errors MUST NOT be retried in a way that bypasses
the posture.

## Data rules
Error context carries only non-secret, typed fields.

## Security rules
Redaction is enforced at the boundary; a value not proven non-secret is redacted.
Fail-closed for all egress/security variants.

## Accessibility rules
Messages render in the high-contrast error state with a text label (not
color-only).

## Performance rules
N/A.

## Observability rules
Errors logged (redacted) with `component`, `event`, `outcome=error`, variant.

## Required tests
- Each variant maps to its exit code and redacted message.
- Redaction: property test asserts no secret/identity/onion in any rendered
  error.
- Fail-closed: `EgressBlocked`/`Killspace` paths prove the request was not sent.

## Acceptance criteria
- [ ] Taxonomy implemented in `adad-core`; all crates use it.
- [ ] Redaction test passes (no secret leaks in errors).
- [ ] Fail-closed tests pass.
- [ ] `scripts/verify.sh` exits 0.
