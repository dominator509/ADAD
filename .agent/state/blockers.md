# Blockers

## BLK-001 (EP-002, M4) — RESOLVED
Blocker: ADR-002 assumes a standalone claw-code MCP crate plus a standalone tool-execution crate can be vendored in isolation, but upstream `ultraworkers/claw-code` at commit `4ea31c1bc91c4e9bcbd67d51c550c01e127e6d0d` no longer has that shape.
Evidence: `build/vendor-src/claw-code/rust/crates/runtime/src/lib.rs:3-4` says `runtime` owns session persistence, MCP plumbing, tool-facing file operations, and the core conversation loop; `build/vendor-src/claw-code/rust/crates/tools/Cargo.toml:9-13` depends on `api`, `commands`, `plugins`, and `runtime`; `build/vendor-src/claw-code/rust/crates/api/Cargo.toml:10,13` and `build/vendor-src/claw-code/rust/crates/commands/Cargo.toml:12-13` pull `runtime`/`telemetry`; crate inventory under `build/vendor-src/claw-code/rust/crates/*/Cargo.toml` shows no package named `mcp`.
Smallest decision needed: Decide whether ADAD may vendor a larger frozen claw-code subset and accept that runtime surface, or whether ADR-002/EP-002 should switch to a different MCP/tool-execution source.
Recommended default: Revise ADR-002/EP-002 before further implementation; do not vendor the current claw-code runtime stack under the existing two-crate isolation requirement.
Resolution: User chose the architecture pivot on 2026-07-03: replace the failed claw-code vendoring seam with the official MCP Rust SDK plus ADAD-owned execution logic, while targeting Claude-Code-like features and feel through first-party `agent-coding` work.
