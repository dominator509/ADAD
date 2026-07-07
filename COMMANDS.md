# COMMANDS.md — Allowed Commands

These are the ONLY commands agents may run. **Coding agents must not invent
commands. If a command is missing, update this file first with evidence from the
repository.** Every entry either wraps a script in `scripts/` or is a primitive
the scripts themselves call.

## Working directory rule

All commands run from the repository root. Every script `cd`s to the repo root
itself (`cd "$(dirname "$0")/.."`), so they are safe to invoke from anywhere,
but you should be at the root.

## Package manager rule

- Rust crates: `cargo` (stable toolchain, `x86_64-unknown-linux-musl` target).
- Host OS packages: `apt` (Debian). Agents do NOT run `apt install` in a
  session — missing host tools are a STOP condition (see `scripts/install.sh`).
- EP-009 builder exception: when the user explicitly approves resolving the
  image-builder blocker, agents may build the repo-owned Docker builder image
  with `scripts/build-image-builder.sh`. The only `apt` use is inside
  `live-build/builder/Dockerfile`; it installs the image-build tools listed in
  `ENVIRONMENT.md` and does not touch host packages or physical devices.
- No other package managers. No `npm`, no `pip` in the core build.

## Lifecycle commands (use these — do not inline their internals)

| Purpose | Command | Success line |
|---|---|---|
| Preflight | `scripts/preflight.sh [EXECPLAN_PATH]` | `preflight: ok` |
| Install deps | `scripts/install.sh` | `install: ok` |
| Lint | `scripts/lint.sh` | `lint: ok` |
| Format check | `scripts/format-check.sh` | `format check: ok` |
| Typecheck (cargo check) | `scripts/typecheck.sh` | `typecheck: ok` |
| Unit tests | `scripts/test-unit.sh` | `unit tests: ok` |
| Integration tests | `scripts/test-integration.sh` | `integration tests: ok` |
| E2E / leak battery | `scripts/test-e2e.sh` | `e2e tests: ok` |
| Build (static musl) | `scripts/build.sh` | `build: ok` |
| Security check | `scripts/security-check.sh` | `security check: ok` |
| Dependency audit | `scripts/dependency-audit.sh` | `dependency audit: ok` |
| Smoke test | `scripts/smoke-test.sh` | `smoke test: ok` |
| Full verify | `scripts/verify.sh` | `verify: ok` |
| Production readiness | `scripts/production-readiness-check.sh` | `production readiness: ok` |
| Build EP-009 image builder | `scripts/build-image-builder.sh` | `image builder: ok` |
| Check EP-009 image builder | `scripts/check-image-builder.sh` | `image builder check: ok` |
| Build bootable image | `scripts/build-image.sh` | `image build: ok` |
| OS boot smoke | `tests/os/boot-smoke.sh` | `boot smoke: ok` |
| Rollback drill | `tests/os/rollback-drill.sh` | `rollback drill: ok` |
| Minimum system simulation | `scripts/min-system-sim.sh [profile ...]` | `min system sim: ok` |
| Fetch repo-local llama runtime | `scripts/fetch-llama-cpp-runtime.sh` | `llama runtime: ok` |

Host notes:
- `scripts/build.sh` performs the actual static-link check on Linux. On
  non-Linux hosts it prints an explicit skip line and relies on Linux CI for the
  authoritative static assertion.
- `scripts/smoke-test.sh` executes the musl binaries on Linux. On non-Linux
  hosts it prints an explicit skip line instead of silently passing without
  execution.

## Local development commands

- Apply Rust formatting after `scripts/format-check.sh` reports diffs:
  `cargo fmt --all`
  (shape confirmed by `scripts/format-check.sh`, which runs `cargo fmt --all --check`)
- Run a core tool locally (after build):
  `target/x86_64-unknown-linux-musl/release/<tool> --help`
- Run one crate's tests only: `cargo test -p <crate-name>`
- Fetch the historical claw-code diagnostic snapshot:
  `git clone --depth 1 https://github.com/ultraworkers/claw-code.git build/vendor-src/claw-code`
  (used for BLK-001 / ADR-009 evidence; not part of the active MCP foundation path)
- Run the local model server (llama.cpp), used by agent-coding integration
  tests: `llama-server --model <path-to-gguf> --host 127.0.0.1 --port 8080`
  (OpenAI-compatible endpoint at `/v1/chat/completions`). Confirm exact flags
  with `llama-server --help`; do not guess.
- Prepare a containerized EP-009 image builder when host tools are unavailable
  and the blocker has an explicit user resolution:
  `scripts/build-image-builder.sh` then `scripts/check-image-builder.sh`
  (Dockerfile lives at `live-build/builder/Dockerfile` and installs the
  `ENVIRONMENT.md` image-build tools inside the container only).
- Build the EP-009 Debian-Live image artifact:
  `scripts/build-image.sh`
  (runs live-build inside `adad-ep009-builder:local` and writes
  `build/adad.img`; the container receives mount capability for live-build's
  chroot `/proc` and `/dev/pts` mounts, but no host block devices are bound).

## Database / migrations

Not applicable. ADAD has no traditional database. "Persistence" is the LUKS2
vault (an image file in tests) and blockchain state via `monero-wallet-rpc`.
There are no schema migrations; vault layout changes are handled by
`forge-rs`/`persona-rs` and covered in EP-003. Do not invent migration
commands.

## Loop commands

- Outer loop driver:
  ```sh
  AGENT_CMD='<your coding-agent CLI that takes a prompt as its last arg>' \
    MAX_ITERATIONS=100 scripts/loop.sh
  ```
  `AGENT_CMD` must be a command that accepts a single prompt string as its final
  argument and can read files, edit files, and run terminal commands in the
  repo. Example:
  ```sh
  AGENT_CMD='codex --cd . --ask-for-approval never --sandbox workspace-write' \
    scripts/loop.sh
  ```
- Next-step selector (prints next ExecPlan path, or `DONE`, or `BLOCKED:<path>`):
  `scripts/next-step.sh`
- Loop status (one-screen summary):
  `scripts/loop-status.sh`

## Forbidden commands / actions

- Inventing any command not listed here.
- `apt install` / `apt-get install` inside an agent session (STOP instead),
  except the EP-009 Docker builder path explicitly listed above.
- `cargo install claw-code` (it is a deprecated stub — see EP-002 vendoring
  notes; use the pinned vendored crates instead).
- Any write to a real block device: `dd of=/dev/...`, `mkfs`, `wipefs`,
  `cryptsetup luksFormat /dev/...`, `kexec` on a live host. Image files only.
- Pushing to a real remote, provisioning a real VPS, or spending real XMR.
- Editing `.agent/state/` files by hand **except** filling the `Resolution:`
  section of `.agent/state/blockers.md`.
- Disabling, deleting, or `#[ignore]`-ing a leak/security test to make a gate
  pass.

## Recovery instructions

- If a lifecycle script fails because its target does not exist yet (e.g.
  `no Cargo.toml`), that means an earlier ExecPlan milestone has not run. Do not
  fake the command — implement the milestone that creates the target.
- If a command is genuinely missing from this file, add it here with a one-line
  justification citing the repository file that proves its shape (e.g. the
  crate name from `Cargo.toml`), then use it.
- If `scripts/verify.sh` fails, use `.agent/prompts/debug-validation-failure.md`
  and the bounded-retry rule. Do not broaden scope to "fix everything."
