# SPEC-001 — Core Domain

- **Status:** active
- **Owner:** architect
- **Roadmap phase:** Phase 1
- **Linked ExecPlans:** EP-002

## User-visible goal
The foundational types and sterile-creation logic behave correctly and leak no
host metadata, so every higher layer builds on a trustworthy core.

## Non-goals
- No I/O in `adad-core` (no network, filesystem, or process spawning).
- No provider/networking logic here (that is EP-004/EP-006).

## Terms
- **Zero-Clock epoch:** a randomized, host-independent timestamp base used for
  all filesystem writes by `forge-rs` to prevent host wall-clock leakage.
- **Config-version:** an integer in the vault config marking layout compatibility.

## Required behavior
- `adad-core` MUST provide: the config schema + validator, the error taxonomy
  (SPEC-006), the session-identity type (owned by persona), and the
  provider-selection enum (`local`|`openai`|`venice`).
- The config validator MUST reject unknown keys and invalid combinations (e.g.
  `provider=venice` + anonymized without the opt-in flag).
- `forge-rs` sterile logic MUST compute a Zero-Clock epoch deterministically
  from a seed (testable) and apply it uniformly; no host wall-clock value may
  appear in any produced timestamp.
- `adad-core` MUST NOT reference any other ADAD crate or perform I/O.

## Inputs
Config text/bytes; a seed for the Zero-Clock epoch; identity fields.

## Outputs
Validated `Config`; typed errors; a `ZeroClockEpoch`; a `SessionIdentity`.

## Error states
Invalid config → `Error::Config`; invalid identity → `Error::Identity`; each
with a redacted, deterministic message (SPEC-006).

## Data rules
Config-version is validated on load; identity fields are typed (no free-form
that could leak real data unintentionally).

## Security rules
No secret is embedded in an error or debug output. Sensitive fields use a
redacting `Debug` impl.

## Accessibility rules
N/A (no UI in this layer).

## Performance rules
Validation and epoch computation are O(n) in input size; negligible.

## Observability rules
Pure logic emits no logs; callers log outcomes.

## Required tests
- Config validator: positive, unknown-key, invalid-combination cases.
- Zero-Clock: determinism from seed; no host clock in output.
- Redacting Debug: secrets never render.
- Compile-time/architecture: `adad-core` has no ADAD-crate deps (checked via
  its `Cargo.toml`).

## Acceptance criteria
- [ ] `cargo test -p adad-core` passes with the above cases.
- [ ] `adad-core/Cargo.toml` depends on no other ADAD crate.
- [ ] `scripts/verify.sh` exits 0.
