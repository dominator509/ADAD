---
id: EP-013
status: complete
depends_on: [EP-012]
verify: scripts/verify.sh
---

# EP-013 — Independent Audit Remediation

## 1. Purpose / Big Picture

Correct the confirmed low-level defects identified by the independent audit and
remove false release confidence where the repository currently cannot prove a
production capability. This plan is deliberately bounded: external provider,
wallet, VPS, WireGuard, DMS, panic, hardware, and image-runtime work remain
separate engineering tracks unless their concrete adapters already exist in the
repository. The existing MetaFUSE boundary is extended only with a Linux
read-only adapter that reuses its current pure policy; this does not make live
image mounting or payload-level metadata rewriting a verified release claim.

## 2. Scope

- Use a certificate-validating Rust HTTPS client for inference requests.
- Make fallback egress fail closed by default.
- Make vault configuration string serialization and parsing symmetric.
- Pass vault passphrases to `cryptsetup` through a closed stdin pipe, never a
  plaintext temporary file.
- Bind image and on-image leak evidence to the current source revision and image
  digest.
- Make release-critical CI and local release gates fail when required image
  evidence is unavailable or stale.
- Prevent completed-plan bookkeeping from producing a readiness pass while the
  evidence-first production checklist still has open gates.
- Reconcile public release documentation with the current implementation.
- Make the shipped binaries dispatch to existing library behavior and make
  release smoke fail on version-only placeholders, without performing any
  real external operation in this session.
- Connect the existing agent/status state surfaces to a real terminal event
  loop and a conservative host daemon probe; unavailable or ambiguous runtime
  state must remain `Unknown`.
- Make the local model image boundary explicit: a release image must stage a
  real llama.cpp runtime and model artifact and install its boot-time service,
  or the image build must fail closed.
- Connect the existing `leakguard` boundary to a fail-closed WireGuard
  lifecycle using the vault-materialized runtime config path.
- Connect the existing agent loop to bounded, read-only native workspace tools
  so a shipped local chat can perform an observable safe tool action without
  gaining shell, network, or secret-file access.
- Add a concrete official-`rmcp` stdio client behind the existing MCP registry
  and executor seam, without inventing a new configuration format or enabling
  unqualified remote execution.
- Connect the shipped interactive agent TUI to the existing bounded agent loop
  and read-only workspace executor while preserving incremental text delivery.
- Route the existing production SSH adapter through the local Tor SOCKS
  boundary without adding a direct-clearnet fallback.
- Route configured non-loopback wallet RPC endpoints only through the local Tor
  SOCKS boundary, while retaining the supervised loopback path.
- Connect the existing leakguard netlink model to a real Linux link-event
  source and fail-closed nftables reaction in the shipped image.
- Connect the existing DMS state machine to a real, image-only LUKS2 header
  file adapter with strict regular-file validation and durable read-back
  verification; do not claim production block-device destruction or automatic
  Tor-NTP acquisition without their required runtime evidence.
- Add a Linux-only, read-only FUSE adapter behind the existing MetaFUSE policy;
  reject symlinks and special files, hide extended attributes, and keep the
  source tree unmodified. Live `/dev/fuse` mounting remains an external image
  validation gate.
- Pin the target live-image Debian mirrors to the reviewed dated snapshot and
  bind that input into image provenance/readiness.
- Make the existing minimum-system inference acceptance path fail closed when
  its caller explicitly requires model/server evidence; preserve optional local
  timing runs as explicitly skipped rather than falsely measured.

## 3. Non-goals

- No live-image FUSE mount, real wallet transfer, VPS provisioning, production
  DMS scheduling/block-device destruction, panic/kexec, hardware boot, or
  production deployment.
- No claims that pure models, mocks, or headless TUI tests prove those external
  behaviors.
- No real device, vault, remote host, wallet, or secret is touched.

## 4. Context and Orientation

The audit correctly identifies that the completed historical ExecPlans closed
mock/model seams while leaving production adapters deferred. The current source
also contains independently actionable defects in the provider client, config
parser, vault secret handling, and release evidence chain. The source and tests
are authoritative; historical readiness text is not evidence of current image
or external-service behavior.

## 5. Files to Read First

- `AGENTS.md`
- `COMMANDS.md`
- `scripts/production-readiness-check.sh`
- `scripts/build-image-inside.sh`
- `scripts/production-readiness-check.sh`
- `scripts/build-image.sh`
- `tests/os/run-qemu-leak-battery-inside.sh`
- `live-build/hooks/0100-adad-hardening.hook.chroot`
- `crates/agent-coding/src/client.rs`
- `crates/adad-core/src/config.rs`
- `crates/forge/src/vault.rs`
- `crates/agent-coding/src/lib.rs`
- `crates/agent-coding/src/execution.rs`
- `crates/agent-coding/src/loop.rs`
- `crates/agent-coding/src/main.rs`
- `crates/agent-coding/src/mcp.rs`
- `crates/agent-coding/tests/agent_loop.rs`
- `crates/agent-coding/tests/support/mock_inference.rs`
- `crates/forge/src/main.rs`
- `crates/forge/tests/failure_wrong_passphrase.rs`
- `crates/forge/tests/regression_vault_upgrade.rs`
- `crates/forge/tests/support/loop_image.rs`
- `crates/forge/tests/vault_boundary.rs`
- `crates/forge/tests/vault_roundtrip.rs`
- `crates/forge/tests/vault_upgrade.rs`
- `crates/git-spoof/src/main.rs`
- `crates/leakguard/src/main.rs`
- `crates/leakguard/src/netlink.rs`
- `crates/leakguard/src/wireguard.rs`
- `ARCHITECTURE.md`
- `crates/metafuse/src/main.rs`
- `crates/persona/src/main.rs`
- `crates/vps-deploy/src/lib.rs`
- `crates/vps-deploy/src/main.rs`
- `crates/vps-deploy/src/provision.rs`
- `crates/xmr-wallet/Cargo.toml`
- `crates/xmr-wallet/src/lib.rs`
- `crates/xmr-wallet/src/main.rs`
- `crates/xmr-wallet/src/rpc.rs`
- `ENVIRONMENT.md`
- `.github/workflows/ci.yml`

## 6. Files to Change

- `.agent/execplans/EP-013-independent-audit-remediation.md`
- `COMMANDS.md`
- `Cargo.lock`
- `crates/agent-coding/Cargo.toml`
- `crates/agent-coding/src/client.rs`
- `crates/agent-coding/tests/egress_guard.rs`
- `crates/agent-coding/tests/failure_provider.rs`
- `crates/adad-core/src/config.rs`
- `crates/adad-core/tests/config.rs`
- `crates/forge/src/vault.rs`
- `scripts/build-image-inside.sh`
- `scripts/build.sh`
- `scripts/production-readiness-check.sh`
- `scripts/smoke-test.sh`
- `scripts/test-integration.sh`
- `scripts/verify.sh`
- `ENVIRONMENT.md`
- `tests/e2e/run-leak-battery.sh`
- `tests/os/run-qemu-leak-battery-inside.sh`
- `live-build/builder/Dockerfile`
- `live-build/config/package-lists/adad-base.list.chroot`
- `live-build/hooks/0100-adad-hardening.hook.chroot`
- `rust-toolchain.toml`
- `.github/workflows/ci.yml`
- `scripts/fetch-llama-cpp-runtime.sh`
- `scripts/min-system-sim.sh`
- `scripts/min-system-sim-inside.sh`
- `README.md`
- `LICENSE`
- `HOW_TO_USE.md`
- `PROJECT_BRIEF.md`
- `.agent/specs/SPEC-003-api-contracts.md`
- `PRODUCTION_READINESS.md`
- `RELEASE.md`
- `AGENTS.md`
- `CONTRIBUTING.md`
- `ROLLBACK.md`
- `SECURITY.md`
- `ROADMAP.md`
- `crates/agent-coding/src/health.rs`
- `crates/agent-coding/src/lib.rs`
- `crates/agent-coding/src/loop.rs`
- `crates/agent-coding/src/tui/agent_chat.rs`
- `crates/agent-coding/src/tui/mod.rs`
- `crates/agent-coding/src/tui/status.rs`
- `crates/agent-coding/src/main.rs`
- `crates/agent-coding/tests/agent_loop.rs`
- `crates/agent-coding/tests/contract.rs`
- `crates/agent-coding/tests/support/mock_inference.rs`
- `crates/agent-coding/tests/status_accuracy.rs`
- `crates/agent-coding/tests/tui_acceptance.rs`
- `crates/forge/src/main.rs`
- `crates/forge/tests/failure_wrong_passphrase.rs`
- `crates/forge/tests/regression_vault_upgrade.rs`
- `crates/forge/tests/support/loop_image.rs`
- `crates/forge/tests/vault_boundary.rs`
- `crates/forge/tests/vault_roundtrip.rs`
- `crates/forge/tests/vault_upgrade.rs`
- `crates/git-spoof/src/main.rs`
- `crates/git-spoof/src/lib.rs`
- `crates/git-spoof/src/rewrite.rs`
- `crates/git-spoof/tests/regression_stable_pseudonym.rs`
- `crates/leakguard/src/lib.rs`
- `crates/leakguard/src/main.rs`
- `crates/leakguard/src/dms.rs`
- `crates/leakguard/Cargo.toml`
- `crates/leakguard/tests/dms.rs`
- `crates/leakguard/src/wireguard.rs`
- `crates/metafuse/src/main.rs`
- `crates/persona/src/main.rs`
- `crates/vps-deploy/src/lib.rs`
- `crates/vps-deploy/src/main.rs`
- `crates/vps-deploy/src/provision.rs`
- `crates/xmr-wallet/Cargo.toml`
- `crates/xmr-wallet/src/lib.rs`
- `crates/xmr-wallet/src/main.rs`
- `crates/xmr-wallet/src/rpc.rs`
- `scripts/build-image.sh`
- `tests/os/run-qemu-leak-battery-inside.sh`

## 7. Interfaces and Contracts

- `OpenAiCompatClient` supports local HTTP only for loopback endpoints and HTTPS
  for fallback endpoints, using normal certificate and hostname validation.
- A newly constructed fallback client denies requests until an authoritative
  egress state is injected.
- Rendered config strings round-trip all supported basic-string escapes.
- `cryptsetup` receives the passphrase on stdin and no passphrase file is
  created.
- `build/adad-image.provenance` records the source SHA, source tree, image
  SHA-256, source date, and the local model/runtime hashes; readiness verifies
  the source/image relationship against the current clean checkout and leak
  pass marker.
- Missing release image evidence is a failure for the release job, while normal
  source-only development verification remains explicit about its scope.
- The production-readiness gate invokes verification with both
  `ADAD_REQUIRE_IMAGE=1` and `ADAD_REQUIRE_VAULT=1`, so an image-backed release
  check cannot silently fall back to source-only integration or vault tests.
- The shipped local chat registers only `workspace_read_file` and
  `workspace_list_dir`; both accept relative workspace paths, reject traversal,
  symlink escapes, and sensitive names, and cap returned data. They do not
  execute shell commands or write to the workspace.
- The current `forge` vault boundary accepts only regular image files. Create
  may receive a missing target, but unlock and upgrade require an existing
  regular file; symlinks and device/non-regular targets are rejected before
  any host utility is invoked.
- The MCP boundary supports explicitly configured stdio and streamable-HTTP
  servers through the pinned `rmcp` SDK. Remote HTTP endpoints require HTTPS,
  are denied until the injected authoritative fallback egress is active, and
  loopback HTTP is allowed only for local services and fixtures.

## 8. Milestones

### M1 — Secure provider transport and egress default

- Goal: HTTPS provider URLs are supported with TLS and fallback clients deny by
  default.
- Files to read: `crates/agent-coding/src/client.rs`, provider tests, Cargo
  manifests.
- Files to change: `crates/agent-coding/Cargo.toml`, `Cargo.lock`,
  `crates/agent-coding/src/client.rs`, `crates/agent-coding/tests/egress_guard.rs`.
- Exact edits expected: replace raw provider transport with a rustls-backed
  client, retain the OpenAI-compatible request shape, and add the default-deny
  regression assertion.
- Validation command: `cargo test -p agent-coding`.
- Expected result: all agent-coding tests pass.
- Recovery: inspect the exact client API/compiler error and make one narrow
  compatibility correction; do not weaken TLS or egress assertions.

### M2 — Config and vault secret correctness

- Goal: config values round-trip exactly and no passphrase temp file is created.
- Files to read: `crates/adad-core/src/config.rs`, `crates/forge/src/vault.rs`,
  related tests.
- Files to change: `crates/adad-core/src/config.rs`,
  `crates/adad-core/tests/config.rs`, `crates/forge/src/vault.rs`.
- Exact edits expected: decode TOML basic-string escapes symmetrically and pipe
  the sensitive bytes to `cryptsetup --key-file -`.
- Validation command: `cargo test -p adad-core && cargo test -p forge`.
- Expected result: both package test commands pass.
- Recovery: preserve image-file-only vault tests and isolate any platform issue;
  never restore plaintext file handling.

### M3 — Artifact provenance and non-skipping release gates

- Goal: a readiness pass is impossible for an absent, stale, or unbound image.
- Files to read: build, integration, leak, CI, and readiness scripts.
- Files to change: `scripts/build-image-inside.sh`,
  `tests/os/run-qemu-leak-battery-inside.sh`,
  `scripts/production-readiness-check.sh`, `scripts/test-integration.sh`,
  `.github/workflows/ci.yml`.
- Exact edits expected: emit and verify provenance, include hashes in the leak
  marker, make the CI release job build and test the exact image, and reject any
  remaining unchecked production-readiness criterion.
- Validation command: `scripts/verify.sh` and shell lint through
  `scripts/lint.sh`.
- Expected result: source-only verification succeeds; release requirements are
  explicit and fail closed when image infrastructure is absent.
- Recovery: inspect the first failing gate and apply the bounded retry rule.

### M4 — Truthful public documentation

- Goal: public docs disclose the current experimental boundary and no longer
  describe claw-code as the active MCP implementation.
- Files to read: `README.md`, `HOW_TO_USE.md`, `SPEC-003`,
  `PRODUCTION_READINESS.md`.
- Files to change: those files plus `LICENSE`.
- Exact edits expected: add MIT text, document supported/currently deferred
  paths, and reconcile the MCP and release-readiness descriptions.
- Validation command: `scripts/security-check.sh`.
- Expected result: `security check: ok`.
- Recovery: keep docs factual and preserve the no-secret scan.

### M5 — Executable reachability and required vault integration

- Goal: remove version-only shipped entrypoints, connect the existing local
  provider/agent loop, wallet RPC, vault, metadata, identity, Git metadata,
  policy, and SSH seams to explicit commands, and make release CI fail when
  required vault prerequisites are unavailable.
- Files to change: the executable, adapter, test, environment, workflow, and
  smoke-test files listed in Section 6.
- Exact edits expected: provide safe command dispatch; keep external actions
  behind explicit confirmation; use loopback-only wallet HTTP; pipe SSH setup
  scripts from stdin; and test non-placeholder execution paths.
- Validation command: focused package tests, `scripts/build.sh`, and the
  native executable `--help`/safe-command checks on Windows; full
  `scripts/verify.sh` remains the plan gate.
- Expected result: all focused tests pass, native release compilation passes,
  and Linux release smoke requires command surfaces plus safe local behavior.
- Recovery: do not invoke a real vault, wallet, VPS, device, or model server;
  preserve the explicit remaining production-backend gaps.

### M6 — Real terminal and conservative daemon observation

- Goal: make the existing agent state machine reachable from a real keyboard
  terminal and make status observations come from host service/interface checks,
  without treating command availability or a synthetic state as service health.
- Files to change: `agent-coding` terminal, health, binary, and focused test
  files listed in Section 6.
- Exact edits expected: add a crossterm-backed event loop with cleanup on every
  exit path, use the existing provider client for a user-submitted prompt, and
  add a Linux-safe system probe that reports `Unknown` when checks are absent or
  ambiguous.
- Validation command: `cargo test -p agent-coding` and native `agent-coding
  --help` compilation/run.
- Expected result: all agent-coding tests pass and the binary documents the
  interactive command without claiming unavailable external backends.
- Recovery: do not invoke a real model, daemon, network tunnel, or external
  service; preserve fail-closed status semantics.

### M7 — Local model image contract and startup supervision

- Goal: prevent an image from being described or tested as local-first when it
  has no real llama-server runtime/model input or startup path.
- Files to change: the image-builder, live-build hook, environment, and CI
  files listed in Section 6.
- Exact edits expected: stage a repo-relative runtime and model artifact,
  install a loopback-only systemd service, and make the on-image battery fail
  if the runtime, model, or service wiring is absent.
- Validation command: shell lint, source verification, and image-path static
  checks available on the current host.
- Expected result: missing release model/runtime inputs fail the image build;
  source verification remains explicit about the unexecuted image gate.
- Recovery: do not download or execute a model in this session; preserve the
  requirement for a real supplied artifact.

### M8 — WireGuard lifecycle adapter and authoritative interface observation

- Goal: connect the existing `leakguard` executable to the documented vault
  runtime config path without allowing missing or ambiguous interface state to
  become an active egress claim.
- Files to change: `crates/leakguard/src/wireguard.rs`, its module exports and
  executable, `COMMANDS.md`, `ENVIRONMENT.md`, and the safe smoke command.
- Exact edits expected: invoke the documented `wg-quick up/down` operations,
  require `/run/adad/wg0.conf` to be a non-symlink root-owned mode-0600 file,
  verify both `ip link show dev wg0 up` and `wg show wg0`, and tear down a failed
  bring-up without exposing child-process output.
- Validation command: `cargo test -p leakguard` and native
  `leakguard wireguard status`.
- Expected result: active is reported only after both observers succeed;
  missing observers report unknown and lifecycle failures return a nonzero
  result without exposing config contents.
- Recovery: do not run `wireguard up/down` on this host; preserve the
  source-level adapter and defer real namespace/interface testing to the
  Linux image gate.

### M9 — Provider structured tool-call bridge

- Goal: preserve the existing ADAD-owned loop contract when a standard
  OpenAI-compatible provider returns a function tool call.
- Files to change: `crates/agent-coding/src/client.rs`,
  `crates/agent-coding/src/loop.rs`, `crates/agent-coding/src/lib.rs`, and
  the existing agent-coding integration support/tests.
- Exact edits expected: parse standard function tool calls, send the
  registered tool descriptors, retain call IDs across assistant/tool
  messages, reject malformed or ambiguous multi-call responses, and keep
  actual tool execution behind the existing registry/executor policy.
- Validation command: `cargo test --workspace`.
- Expected result: the provider bridge and existing loop tests pass without
  introducing an unrestricted native or MCP tool implementation.
- Recovery: preserve the single-call loop contract and return a typed provider
  error for unsupported response shapes; do not execute arbitrary model text.

### M10 — Bounded native workspace tool execution

- Goal: make the existing registered-tool path execute one concrete,
  policy-controlled native workspace action from the shipped local chat.
- Files to change: `crates/agent-coding/src/execution.rs`,
  `crates/agent-coding/src/lib.rs`, `crates/agent-coding/src/main.rs`, and
  the existing agent-coding integration tests.
- Exact edits expected: add read-only file and directory tools rooted at the
  selected workspace, reject traversal/symlink escapes and sensitive paths,
  cap inputs/outputs, and wire the tools through `AgentLoop` and
  `OpenAiAgentModel::with_tools`. No shell, write, network, or credential
  operation is added.
- Validation command: `cargo test -p agent-coding` and native
  `cargo run -p agent-coding -- --help`.
- Expected result: workspace tool policy tests pass and the shipped `chat`
  command constructs a registered executor instead of a no-op executor.
- Recovery: keep tool failures typed as the existing fail-closed provider/I/O
  errors; do not broaden path access or add an unreviewed command runner.

### M11 — Bounded MCP stdio client

- Goal: replace the MCP client-side mock-only gap with a concrete stdio
  transport using the already pinned official `rmcp` SDK.
- Files to change: `crates/agent-coding/Cargo.toml`, `Cargo.lock`,
  `crates/agent-coding/src/mcp.rs`, `crates/agent-coding/src/lib.rs`, and the
  existing agent-coding integration tests.
- Exact edits expected: accept explicitly supplied stdio server configs,
  spawn commands directly without a shell or inherited environment, route only
  registry-qualified MCP tools, bound JSON arguments and rendered results, and
  close the client after every call. HTTP/SSE configuration is not invented in
  this milestone because the active repository contract does not define its
  config shape or egress adapter.
- Validation command: `cargo test -p agent-coding` and `cargo test --workspace`.
- Expected result: MCP config/policy/result-rendering tests pass and the
  production adapter compiles with the official SDK.
- Recovery: if the pinned SDK feature requires a dependency unavailable to the
  static target, revert only the optional child-process feature and preserve
  the source-backed native workspace increment; do not replace it with a mock.

### M12 — Interactive agent loop reachability

- Goal: make the shipped interactive TUI use the same bounded agent loop and
  workspace execution policy as the `chat` command.
- Files to change: `crates/agent-coding/src/loop.rs`,
  `crates/agent-coding/src/tui/agent_chat.rs`, and the existing agent-coding
  integration tests.
- Exact edits expected: retain the TUI's real terminal/event loop and
  incremental rendering, run provider turns through `AgentLoop`, expose only
  the existing read-only workspace registry/executor, preserve conversation
  transcript state between prompts, and surface typed failures without adding
  shell, write, network, or credential operations.
- Validation command: `cargo test -p agent-coding` and
  `cargo test --workspace`.
- Expected result: an interactive prompt can execute a bounded workspace tool
  through the production loop and stream the final model text to the terminal.
- Recovery: keep the current direct streaming path only for a narrowly tested
  provider fallback if loop integration cannot preserve tool-call semantics;
  do not silently restore a tool-free production TUI.

### M13 — Tor-bound SSH transport

- Goal: make the shipped VPS adapter honor the service contract's Tor-only SSH
  boundary while retaining OpenSSH host-key verification and confirmation
  gating.
- Files to change: `crates/vps-deploy/src/provision.rs`,
  `crates/vps-deploy/src/main.rs`, and the existing provision tests.
- Exact edits expected: add a fixed loopback SOCKS5 CONNECT helper that sends
  the target hostname to Tor for resolution, invoke it through a safely quoted
  OpenSSH `ProxyCommand`, reject shell-unsafe target hosts, and never retry via
  direct SSH. The helper must relay stdin/stdout only after a successful
  SOCKS5 response.
- Validation command: `cargo test -p vps-deploy` and
  `cargo test --workspace`.
- Expected result: the transport policy and SOCKS5 handshake tests pass, while
  existing mock provisioning behavior remains unchanged.
- Recovery: if the host lacks a real Tor listener, retain only loopback fake
  SOCKS5 tests; do not add a clearnet fallback or invoke a real remote host.

### M14 — Tor-bound wallet RPC transport

- Goal: make the existing wallet RPC client support the active contract's
  Tor-routed endpoint without permitting a direct non-loopback request.
- Files to change: `crates/xmr-wallet/src/rpc.rs`,
  `crates/xmr-wallet/src/lib.rs`, and the existing wallet tests.
- Exact edits expected: retain direct HTTP only for loopback wallet RPC,
  recognize only validated `.onion` HTTP endpoints for remote operation, send
  the request through the fixed loopback Tor SOCKS5 listener with domain-form
  CONNECT, bound the HTTP response, and reject all other hosts/schemes.
  Transfer preparation remains `do_not_relay=true` and no funds are spent.
- Validation command: `cargo test -p xmr-wallet` and
  `cargo test --workspace`.
- Expected result: loopback behavior remains green and a fake SOCKS5 proxy
  proves onion hostname routing and JSON-RPC response handling.
- Recovery: if a Tor listener is unavailable, preserve the isolated fake-proxy
  test and fail closed at runtime; do not add DNS, clearnet, or proxy-variable
  fallback behavior.

### M15 — Linux link monitor and dynamic killswitch reaction

- Goal: replace the disconnected netlink event model in the shipped runtime
  with a real Linux link-event process and an observable fail-closed firewall
  reaction.
- Files to change: `crates/leakguard/src/netlink.rs`,
  `crates/leakguard/src/lib.rs`, `crates/leakguard/src/main.rs`, and the
  existing image hardening hook.
- Exact edits expected: add a bounded `ip monitor link` adapter that reacts
  only to down/deleted events, flushes the existing fixed
  `inet adad_killswitch` table so its drop policies remain active, returns an
  error when the event source terminates, and supervise it with a restartable
  systemd service. Add parser tests and require the service in the on-image
  battery. Do not add a direct network fallback or alter the existing pure
  policy model.
- Validation command: `cargo test -p leakguard`, `cargo test --workspace`,
  and `scripts/verify.sh`.
- Expected result: source tests cover conservative event classification and
  the release image contains an enabled monitor whose failure is visible to
  systemd and whose reaction leaves the nftables table fail-closed.
- Recovery: if Linux runtime execution is unavailable, retain only source
  tests and the image service contract; do not claim packet-level or QEMU
  evidence from Windows.

### M16 — Explicit release-image runtime inventory

- Goal: prevent the release image from omitting programs required by the
  shipped vault, Git, Tor, WireGuard, filesystem, and leakguard workflows.
- Files to change: `live-build/config/package-lists/adad-base.list.chroot`,
  the existing image hardening hook, and `ARCHITECTURE.md`.
- Exact edits expected: add the runtime packages to the target image package
  list and make the existing on-image leak battery require `cryptsetup`,
  `losetup`, `mkfs.ext4`, `mount`, `umount`, `truncate`, `git`, `tor`, `wg`,
  and `wg-quick` in addition to its existing requirements. Do not treat the
  container builder's packages as target-image evidence.
- Validation command: `scripts/verify.sh`.
- Expected result: source verification passes and a future image battery
  fails closed with a named missing-runtime marker instead of producing a
  green result for an incomplete image.
- Recovery: if image execution is unavailable, retain the explicit package and
  hook contract and leave actual on-image command presence as external image
  evidence; do not mark the image-ready checklist item complete.

### M17 — Connect stable identity to a real local Git commit

- Goal: replace the disconnected Git metadata renderer with the bounded local
  commit path required by SPEC-005.
- Files to change: `crates/git-spoof/src/main.rs`,
  `crates/git-spoof/src/lib.rs`, `crates/git-spoof/src/rewrite.rs`, the
  existing stable-pseudonym integration test, `scripts/smoke-test.sh`, the
  existing image hardening hook, `COMMANDS.md`, and `ARCHITECTURE.md`.
- Exact edits expected: add a `git-spoof commit <message>` command that only
  commits already-staged changes in the current repository, supplies the
  persona-derived author and committer environment, sets both dates to the
  fixed UTC value, and returns the resulting local commit id. Do not add push,
  arbitrary Git subcommands, host-identity fallback, or remote access.
- Validation command: `cargo test -p git-spoof`, `cargo test --workspace`, and
  `scripts/verify.sh`; Linux smoke and the image battery also exercise the
  disposable local commit path when their runtime gates are available.
- Expected result: an integration test creates a disposable local Git
  repository, stages a file, creates a commit through the library boundary,
  and observes only the stable pseudonym and normalized UTC metadata.
- Recovery: if Git is unavailable, retain the typed failure boundary and do
  not substitute a pure renderer for the commit path; the required Git runtime
  remains an image/host prerequisite.

### M18 — Image-only LUKS2 DMS header adapter

- Goal: replace the DMS model's only in-memory header target with a concrete,
  safe adapter for disposable LUKS2 image files.
- Files to change: `crates/leakguard/Cargo.toml`,
  `crates/leakguard/src/dms.rs`, `crates/leakguard/src/lib.rs`,
  `crates/leakguard/src/main.rs`, the existing DMS integration tests,
  `COMMANDS.md`, `ARCHITECTURE.md`, `SECURITY.md`, and `ENVIRONMENT.md`.
- Exact edits expected: validate LUKS2 magic/version and regular-file targets,
  reject symlinks/devices, open with Unix no-follow/close-on-exec flags, wipe in
  bounded chunks, flush and read back the selected header, and route the
  existing Tor-NTP-driven state machine through that target. Expose only an
  image-evaluation command; do not add device wiping, local-clock authority,
  invented time acquisition, or kexec.
- Validation command: `cargo test -p leakguard`, `cargo test --workspace`, and
  `scripts/verify.sh`.
- Expected result: a disposable LUKS2 image loses only its selected header on
  expiry, non-LUKS files are rejected, source and image smoke invoke the real
  adapter, and the remaining production DMS evidence stays unchecked.
- Recovery: if Linux no-follow flags or image execution are unavailable, retain
  the typed fail-closed boundary and image-only contract; never broaden the
  target to a block device.

### M19 — Pin target-image Debian inputs

- Goal: close the remaining source-visible reproducibility gap where the
  builder snapshot was pinned but live-build target mirrors were implicit.
- Files to change: `scripts/build-image-inside.sh`,
  `scripts/production-readiness-check.sh`, `ENVIRONMENT.md`,
  `ARCHITECTURE.md`, and this ExecPlan.
- Exact edits expected: pass the reviewed Debian and Debian-security snapshot
  URLs to live-build for bootstrap, chroot, and binary stages; disable mutable
  updates/backports; preserve apt signature checking while disabling only the
  snapshot `Valid-Until` expiry check; record the snapshot and mirror values in
  provenance; and require the expected snapshot in readiness validation.
- Validation command: `scripts/verify.sh`.
- Expected result: a future image build cannot silently resolve target packages
  from current mirrors, and readiness rejects provenance that omits or changes
  the pinned snapshot.
- Recovery: if the installed live-build version rejects a documented mirror
  option, preserve the pinned builder and fail the image build; do not fall back
  to an implicit current mirror.

### M20 — Required inference timing cannot silently skip

- Goal: prevent an explicitly requested local-model acceptance run from
  reporting success when its GGUF/model-server inputs are absent or unusable.
- Files to change: `scripts/min-system-sim.sh`,
  `scripts/min-system-sim-inside.sh`, `COMMANDS.md`, `ENVIRONMENT.md`, and
  this ExecPlan.
- Exact edits expected: validate `ADAD_REQUIRE_INFERENCE` at the host and
  container boundaries, require a model input before Docker starts, and turn
  missing GGUF, missing `llama-server`, readiness failure, or response parsing
  failure into an error when the flag is `1`; retain named `skipped` results
  only for explicitly optional runs.
- Validation command: `ADAD_REQUIRE_INFERENCE=1 scripts/min-system-sim.sh`
  with no model input for the early fail-closed path, plus `scripts/verify.sh`.
- Expected result: the required path exits nonzero with a named missing-input
  error before Docker/QEMU work; the source gate passes and optional runs keep
  their existing tabular skip semantics.
- Recovery: do not fabricate or download an unreviewed model; leave the
  required run failed until a reviewed model/runtime artifact is supplied.

### M21 — Full verification cannot omit the E2E harness

- Inspect: `scripts/verify.sh` and `scripts/test-e2e.sh`.
- Change: invoke the E2E entrypoint unconditionally from the full verifier. The
  E2E entrypoint remains responsible for the explicitly documented source-only
  versus image-required distinction; a missing acceptance harness is no longer
  reported as a green verifier result.
- Validation command: `scripts/verify.sh`.
- Expected result: the current source-only E2E/model battery passes, and a
  missing `tests/e2e/run-leak-battery.sh` would fail the verifier immediately.

### M23 — MCP streamable-HTTP client path

- Goal: close the active SPEC-003 MCP transport gap with a concrete official
  `rmcp` streamable-HTTP client while preserving the existing direct stdio
  adapter and fail-closed egress policy.
- Files to change: `crates/agent-coding/Cargo.toml`, `Cargo.lock`,
  `crates/agent-coding/src/mcp.rs`, `crates/agent-coding/src/lib.rs`,
  `ARCHITECTURE.md`, `.agent/specs/SPEC-003-api-contracts.md`, and this plan.
- Exact edits expected: add explicit streamable-HTTP server configuration,
  require HTTPS outside loopback, disable ambient proxy use and redirects,
  install the static-musl-compatible ring rustls provider, route remote calls
  through the existing authoritative egress state, bound calls/results, and
  close the rmcp client after each call. Do not add credentials or a direct
  network fallback.
- Validation command: `cargo test -p agent-coding`.
- Expected result: all agent-coding library and integration tests pass,
  including URL policy and default egress-blocking regressions.
- Recovery: preserve the stdio and egress tests if a platform-specific HTTP
  dependency issue appears; do not weaken the TLS, redirect, proxy, or
  egress restrictions.

### M24 — Readiness uses release-shaped verification

- Goal: prevent `scripts/production-readiness-check.sh` from validating an
  image while its nested verifier silently uses source-only integration and
  vault-test modes.
- Files to change: `scripts/production-readiness-check.sh` and this plan.
- Exact edits expected: pass `ADAD_REQUIRE_IMAGE=1` and
  `ADAD_REQUIRE_VAULT=1` to the nested `scripts/verify.sh` invocation while
  retaining the existing source-SHA, image-digest, provenance, and checklist
  assertions.
- Validation command: `scripts/lint.sh` and `scripts/verify.sh`.
- Expected result: source verification remains green, and the readiness gate's
  nested verification is release-shaped rather than source-only.
- Recovery: preserve the explicit fail-closed image and vault checks; do not
  turn unavailable Linux/image prerequisites into skips.

### M25 — Hosted release image receives reviewed external inputs

- Goal: make the existing release-image workflow executable without pretending
  that an uncommitted model or mutable runtime lookup is release evidence.
- Files to change: `.github/workflows/ci.yml`, `scripts/fetch-llama-cpp-runtime.sh`,
  `scripts/verify.sh`, `ENVIRONMENT.md`, and this plan.
- Exact edits expected: make the image job an explicit `workflow_dispatch`
  path with required model URL/hash and llama.cpp tag/archive-hash inputs;
  require HTTPS for the model download; verify both external artifacts before
  invoking the existing builder; and keep ordinary push/pull-request checks
  source-only and explicit.
- Validation command: `scripts/lint.sh`, `scripts/verify.sh`, and
  `git diff --check`.
- Expected result: a clean hosted release run has a documented, checksum-bound
  way to stage the required model/runtime, while no release artifact can be
  fabricated from missing or unverified inputs.
- Recovery: do not add or download an unreviewed model in this session; leave
  the manual workflow requiring operator-supplied reviewed inputs.

### M26 — Shipped agent consumes the documented provider configuration

- Goal: close the source-visible disconnect where the production `agent-coding`
  executable always constructed the local client even when the vault/runtime
  environment selected OpenAI-compatible or Venice fallback settings.
- Files to change: `crates/agent-coding/src/main.rs`,
  `crates/agent-coding/src/tui/agent_chat.rs`,
  `crates/agent-coding/src/tui/status.rs`, `crates/agent-coding/src/tui/mod.rs`,
  `crates/agent-coding/tests/status_accuracy.rs`, and this plan.
- Exact edits expected: construct the existing `adad-core::Config` from the
  documented runtime variables, route it through `provider_select`, use the
  selected URL/key/model for chat and TUI, and preserve default-deny fallback
  egress. Invalid configuration must fail before system probing or network I/O.
- Validation command: `cargo test -p agent-coding`, `scripts/verify.sh`, and
  native status/configuration smoke through the existing test binary.
- Expected result: the shipped binary no longer silently ignores provider
  selection or displays stale local metadata; fallback requests remain blocked
  until an authoritative egress state is supplied.
- Recovery: do not treat an environment variable as tunnel authorization and
  do not add a direct network fallback to make cloud tests pass.

The eight shipped entrypoints also map their existing typed error taxonomy to
the declared exit codes, so invalid runtime configuration is distinguishable
from provider, network, and I/O failures.

### M27 — Link-loss reaction remains fail-closed

- Goal: close a source-confirmed fail-open path in the existing Linux link
  monitor where flushing the normal nftables table removed its drop-policy
  chains.
- Files to change: `crates/leakguard/src/netlink.rs`,
  `live-build/hooks/0100-adad-hardening.hook.chroot`,
  `scripts/security-check.sh`, `COMMANDS.md`, `ARCHITECTURE.md`, `SECURITY.md`,
  and this plan.
- Exact edits expected: install a fixed drop-only ruleset for link-loss events,
  keep loopback only, reject any residual `wg0` allowance in the on-image
  battery, and add source checks for both the adapter and image file.
- Validation command: `cargo test -p leakguard`, `scripts/security-check.sh`,
  and `scripts/verify.sh`.
- Expected result: a down/deleted link cannot turn the nftables boundary into
  accept-by-default; the monitor still fails so systemd restarts it if the
  drop ruleset cannot be loaded.
- Recovery: never restore the old table-flush behavior; if Linux nftables
  execution is unavailable, retain the explicit fail-closed source contract
  and leave image packet evidence outstanding.

### M28 — Linux MetaFUSE read-only adapter

- Goal: close the source-confirmed gap where `metafuse-rs` exposed only a pure
  metadata transformation and its executable explicitly lacked a FUSE mount.
- Files to change: `crates/metafuse/Cargo.toml`, `Cargo.lock`,
  `crates/metafuse/src/lib.rs`, `crates/metafuse/src/main.rs`,
  `crates/metafuse/src/runtime.rs`, `crates/leakguard/src/main.rs`,
  `ARCHITECTURE.md`, `SECURITY.md`, `README.md`, `COMMANDS.md`, and this plan.
- Exact edits expected: add a pinned Linux-only `fuser` dependency with its
  default features disabled; implement a read-only FUSE view that reuses the
  existing scrub policy, rejects symlinks and special files, hides xattrs, uses
  a per-mount random policy seed, and never opens source files for writing;
  expose it as `metafuse mount <source-directory> <mountpoint>`; correct the
  stale link-monitor help text.
- Validation command: `cargo test -p metafuse`,
  `cargo check -p metafuse --target x86_64-unknown-linux-musl`,
  `scripts/verify.sh`, and `git diff --check`.
- Expected result: the Linux release target compiles a concrete MetaFUSE
  adapter and the native policy tests remain green. A real `/dev/fuse` mount,
  unmount, image boot, payload-level EXIF rewriting, and full DMS/panic backend
  remain explicitly outside source-only evidence.
- Recovery: do not fall back to a writable mount, symlink-following path walk,
  or source mutation to make the adapter usable. If the Linux target or FUSE
  device is unavailable, keep the source backend and record the runtime check
  as unavailable.

### M29 — Restore Linux CI shell-entrypoint execution

- Goal: fix the current GitHub Actions `source-verify` failure on a fresh Linux
  checkout, where `scripts/verify.sh` reached `scripts/preflight.sh` and got
  `Permission denied` because the tracked shell files were mode `100644`.
- Files to change: the tracked POSIX shell entrypoints under `scripts/`,
  `tests/e2e/`, and `tests/os/`, `.github/workflows/ci.yml`, the Forge vault
  integration-test support and callers, and this plan.
- Exact edits expected: record mode `100755` for every tracked `.sh` entrypoint
  without changing its content; add a source-job guard that fails if a tracked
  shell entrypoint loses its executable mode; install the `rustfmt` and
  `clippy` components required by the existing verifier for the pinned Rust
  toolchain; make the Forge integration-test preflight verify the disposable
  loop/LUKS/filesystem runtime rather than only checking command names. A
  source-only run may skip that privileged runtime only when it is unavailable;
  `ADAD_REQUIRE_VAULT=1` must continue to fail closed. Do not weaken or skip
  any verifier step.
- Validation command: `scripts/preflight.sh`, `scripts/verify.sh`, and
  `git diff --check`.
- Expected result: a clean Linux checkout can invoke the nested verification
  scripts with the pinned toolchain, the same source verifier exits 0, and a
  future mode or toolchain-component regression is reported before release
  work begins.
- Recovery: if the guard reports a path, restore only that path's tracked
  executable mode; do not replace direct execution with a skipped test.

## 9. Concrete Steps

1. Add the TLS transport and default-deny regression test.
2. Fix parser escaping and vault stdin handling with focused tests.
3. Add source/image provenance and required release CI execution.
4. Rewrite only the stale public claims directly contradicted by current code.
5. Run the full source verification gate and leave external validation clearly
   outstanding.
6. Add the active MCP streamable-HTTP transport through the existing
   `agent-coding` boundary without expanding the tool or credential surface.

## 10. Validation and Acceptance

- [x] Agent provider tests pass with HTTPS support and default-deny fallback.
- [x] Config escape round-trip tests pass.
- [x] Vault tests pass without passphrase temp-file code.
- [x] Source/image provenance is emitted and checked by release gates.
- [x] CI has a non-optional image build/boot/leak path.
- [x] Readiness rejects an unchecked production evidence criterion.
- [x] Public README, license, and MCP documentation are truthful.
- [x] Shipped binaries no longer use version-only entrypoints; safe commands
  reach existing library-backed behavior.
- [x] Wallet and agent adapters have controlled local integration coverage;
  SSH remains confirmation-gated and is not invoked here.
- [x] Required vault integration can be made non-skipping in release CI.
- [x] M15 adds a supervised Linux link monitor that reacts to down/deleted
  events through the nftables killswitch boundary; M27 corrects the reaction
  to load a complete drop-only ruleset rather than flush away policy chains.
  Source tests and the release-image service contract pass, while QEMU packet
  evidence remains external.
- [x] M16 adds explicit target-image packages and on-image checks for vault,
  filesystem, Git, Tor, and WireGuard runtime programs; actual image inspection
  remains external.
- [x] M17 connects `git-spoof commit` to an actual bounded local Git commit;
  disposable-repository metadata integration passes and remote push remains
  out of scope.
- [x] M18 connects the DMS state machine to a validated, image-only LUKS2 file
  target; source/image smoke and read-back tests cover expiry, while automatic
  Tor-NTP acquisition, block-device destruction, and kexec remain external.
- [x] M19 pins target-image Debian bootstrap/chroot/binary mirrors to the
  reviewed snapshot and requires that snapshot in image provenance; a clean
  image build remains external.
- [x] `ADAD_REQUIRE_INFERENCE=1` fails before Docker when no model input is
  supplied, while optional inference runs retain explicit skip statuses.
- [x] M10 — shipped chat executes bounded read-only workspace tools with
  traversal, symlink, sensitive-path, and output-size policy checks.
- [x] M11 — bounded MCP stdio client using the official `rmcp` child-process
  transport, with registry qualification, bounded arguments/results, direct
  process spawning, and per-call shutdown.
- [x] M12 — interactive agent TUI reaches the bounded agent loop and preserves
  incremental final-answer rendering.
- [x] M13 — VPS SSH uses the Tor SOCKS boundary with no direct fallback.
- [x] M14 — wallet RPC uses loopback or Tor-onion transport only.
- [x] `scripts/verify.sh` exits 0.
- [x] M6 — real terminal and conservative daemon observation.
- [x] M7 — local model image contract and startup supervision.
- [x] M8 — WireGuard lifecycle adapter and authoritative interface observation.
- [x] M23 — MCP streamable-HTTP uses the official `rmcp` client, requires TLS
  outside loopback, disables ambient proxy/redirect behavior, and blocks remote
  calls until authoritative fallback egress is active; live remote HTTPS and
  WireGuard packet validation remain external.

- [x] M28 adds a pinned fuser-backed Linux read-only MetaFUSE adapter with
  scrubbed ownership/timestamps, hidden xattrs, symlink/special-file rejection,
  and an explicit `metafuse mount` command; Linux target compilation and native
  policy tests pass, while live `/dev/fuse` mounting remains external.
- [x] M29 restores Linux CI shell-entrypoint execution, installs the verifier's
  pinned-toolchain components, and adds guards against both regressions; Forge
  integration tests now distinguish present command names from an unusable
  privileged runtime while preserving the required-mode failure.

## 11. Idempotence and Recovery

All changes are source-only until an explicitly invoked image build. Image and
provenance files remain under ignored `build/`. Tests use loopback or mock
services and image files only. If a validation command fails, follow the
repository three-strike rule and preserve the first exact error in this plan.

## 12. Progress

- [x] M1 — secure provider transport and egress default (`cargo test -p agent-coding`: 42 passed)
- [x] M2 — config and vault secret correctness (`cargo test -p adad-core`: 18 passed; `cargo test -p forge`: 13 passed)
- [x] M3 — artifact provenance and non-skipping release gates (`scripts/verify.sh`: 0; release mode requires image/provenance and rejects unchecked readiness criteria)
- [x] M4 — truthful public documentation (`scripts/security-check.sh`: ok)
- [x] M5 — executable reachability and required vault integration (focused tests: passing; native release compilation: passing)
- [x] bounded plan verification (`scripts/verify.sh`: exit 0; production-readiness remains NO-GO because current image, Linux runtime, external, hardware, and performance evidence is unavailable)
- [x] M6 — real terminal and conservative daemon observation (`cargo test -p agent-coding`: passing; native `--help`/`status`: passing)
- [x] M7 — local model image contract and startup supervision (shell lint/source verification: passing; missing runtime/model fails the image path; on-image battery now requires a real local agent request)
- [x] M8 — WireGuard lifecycle adapter and authoritative interface observation (`cargo test -p leakguard`: passing; native status is fail-closed on this host)
- [x] M9 — provider structured tool-call bridge (`cargo test --workspace`: passing; registered tool definitions and correlated call IDs are covered)
- [x] M10 — bounded native workspace tool execution (`cargo test -p agent-coding`: 53 passed; native `--help`: passing)
- [x] M11 — bounded MCP stdio client (`cargo test -p agent-coding`: 57 passed;
  `cargo test --workspace`: 156 passed; spawned shipped echo-server transport
  test passed)
- [x] M12 — interactive agent loop reachability (`cargo test -p agent-coding`:
  58 passed; `cargo test --workspace`: 157 passed; provider tool-call and
  streamed final-answer integration passed)
- [x] M13 — Tor-bound SSH transport (`cargo test -p vps-deploy`: 8 passed;
  SOCKS5 hostname-routing and proxy-command injection rejection covered)
- [x] M14 — Tor-bound wallet RPC transport (`cargo test -p xmr-wallet`: 10
  passed; onion HTTP request and bounded response parsing covered)
- [x] M15 — Linux link monitor and dynamic killswitch reaction (`cargo test -p
  leakguard`: 38 passed; `cargo test --workspace`: 161 passed;
  `scripts/verify.sh`: exit 0)
- [x] M16 — Explicit release-image runtime inventory (`scripts/verify.sh`:
  exit 0; target-image command presence remains image-gated)
- [x] M17 — Bounded local Git commit integration (`cargo test -p git-spoof`:
  7 passed; `cargo test --workspace`: 162 passed; `scripts/verify.sh`: exit 0;
  actual remote push remains out of scope)
- [x] M18 — Image-only LUKS2 DMS adapter (`cargo test -p leakguard`: 41
  passed; `cargo test --workspace`: 165 passed; `scripts/verify.sh`: exit 0;
  production scheduler and device evidence remain external)
- [x] M19 — Target-image Debian snapshot pinning (`scripts/verify.sh`: exit 0;
  future image provenance must include `20260801T000000Z`)
- [x] M20 — Required inference timing fails closed on missing/unusable model
  inputs (`ADAD_REQUIRE_INFERENCE=1` exits before Docker without a model);
  optional timing remains explicitly skippable; `scripts/verify.sh`: exit 0.
- [x] M21 — Full verification invokes the E2E entrypoint unconditionally;
  missing acceptance harnesses can no longer be silently skipped.
- [x] M22 — Forge vault image targets fail closed on symlinks (including
  symlinked parents), directories, missing unlock targets, and non-regular
  targets (`cargo test -p forge`: 6 Windows unit tests plus the Unix-only
  symlink regression and all forge integration tests pass on their applicable
  platforms).
- [x] M23 — MCP streamable-HTTP client path (`cargo test -p agent-coding`:
  60 tests passed; remote URL policy and default egress blocking are covered;
  live remote HTTPS and WireGuard packet validation remain external).
- [x] M24 — Production readiness invokes `scripts/verify.sh` with required
  image and vault integration flags; source verification remains green and the
  readiness gate still fails closed on this dirty checkout.
- [x] M25 — Hosted release-image CI now requires operator-supplied HTTPS model
  and SHA-256 inputs plus a verified llama.cpp archive before image build; the
  runtime tag is path-safe and covered by the source regression check;
- [x] M26 — Shipped agent chat, interactive TUI, and status now consume the
  documented provider configuration through `provider_select`; invalid config
  fails closed with its typed exit code, fallback egress remains denied without
  authority, and all shipped entrypoints preserve typed error exit codes.
- [x] M27 — Link-loss monitoring now loads a complete drop-only nftables
  ruleset instead of flushing away the policy chains; source checks pass and
  the image battery rejects a residual WireGuard allowance after link loss.
  ordinary pushes and pull requests cannot claim image readiness without that
  explicit release workflow.

- [x] M28 — Linux MetaFUSE adapter (`cargo test -p metafuse` and
  `cargo check -p metafuse --target x86_64-unknown-linux-musl`: passing);
  source-only verification remains green and live FUSE/image evidence remains
  external.
- [x] M29 — GitHub Actions source verification passes the shell-mode guard,
  pinned `rustfmt`/`clippy` components, and static-musl build after both CI jobs
  install `musl-tools`; the shared Forge preflight distinguishes command
  presence from a usable privileged runtime while `ADAD_REQUIRE_VAULT=1`
  remains fail-closed. Hosted push run 12 and pull-request run 13 pass.
- [x] M30 — Production readiness and the image builder ignore only the
  loop-mandated tracked `.agent/state/last-result.env` bookkeeping update when
  checking source cleanliness; all other source and untracked changes remain
  release-blocking. The artifact provenance and source/tree checks are
  unchanged.
- [x] M31 — The release image installs `openssh-client`, and the on-image leak
  battery requires the `ssh` executable before it can report a valid runtime;
  the existing Tor-bound, confirmation-gated VPS transport is unchanged.
- [x] M32 — The release image installs `iputils-ping`, the on-image leak battery
  requires `ping` before its clearnet/drop probes, and the source verifier keeps
  the package/assertion pair from drifting.
- [x] M33 — The on-image interface-drop probe fails closed when no non-loopback
  interface exists or the requested down/restore transition fails; the source
  verifier guards those assertions.
- [x] M34 — The target image explicitly installs `procps` for the hardening
  hook's `sysctl` checks, and the source verifier guards that package/runtime
  pairing.
- [x] M35 — The on-image static-tool smoke requires successful `--help` exit
  status as well as a usage marker, and the source verifier guards that
  non-masking behavior.
- [x] M36 — The boot MAC hardening and on-image MAC smoke fail closed when no
  non-loopback interface exists or a MAC transition fails; the source verifier
  guards those assertions.
- [x] M37 — Local provider URL validation matches exact loopback hosts, rejects
  host-prefix collisions, userinfo, and invalid ports, and preserves valid
  loopback forms through regression tests.

## 13. Surprises & Discoveries

- The repository had no active plan because EP-012 and every predecessor were
  marked complete even though EP-004 and EP-006 explicitly recorded deferred
  production adapters. This plan therefore starts as `in-progress` and does not
  retroactively convert model/mock acceptance into production evidence.
- The first post-change full library run had one transient Windows workspace
  directory I/O failure; the isolated test passed on immediate rerun and the
  complete package run passed, so no production policy was changed for that
  environment-only event.

## 14. Decision Log

- 2026-08-27: Created EP-013 as the first post-audit remediation plan because
  `scripts/next-step.sh` returned `DONE` while the audit identified source-level
  release blockers.
- 2026-08-27: Selected a rustls-backed `ureq` transport because the existing
  `std::net::TcpStream` implementation cannot perform HTTPS certificate
  validation and the repository requires static-musl-compatible binaries.
- 2026-08-27: Did not pin `rust-toolchain.toml` in this plan. The audit correctly
  identifies the stable channel as mutable, but this checkout lacks the pinned
  rustfmt/clippy components and cannot safely complete rustup downloads; the
  reproducibility claim is narrowed in documentation until a controlled builder
  can pin and verify all inputs.
- 2026-08-27: The continuation pinned `rust-toolchain.toml` to Rust 1.90.0,
  pinned the amd64 Rust builder image by digest, and replaced live Debian mirror
  resolution with the explicit 20260801 snapshot. Two isolated image builds
  and package-level checksum review remain required before reproducibility can
  be marked complete.
- 2026-08-27: Adjusted non-Linux `scripts/build.sh` to perform a native release
  compile and defer only the static-musl check to Linux CI. The prior ordering
  attempted the musl build first and failed on Windows when the TLS dependency
  required a musl C compiler that is intentionally not part of this checkout.
- 2026-08-27: Kept real wallet/VPS/security-backend implementation out of this
  bounded plan; no repository evidence or user-approved external environment
  can safely establish those irreversible or operator-owned behaviors here.
- 2026-08-27: Added an executable check for unchecked entries in
  `PRODUCTION_READINESS.md`; a clean image and completed historical plans are
  insufficient for a readiness pass while functional, security, hardware, or
  external-service evidence remains open.
- 2026-08-27: Reconciled active control-plane, contributor, security, rollback,
  and roadmap guidance with `rmcp`; historical ADR references to the superseded
  claw-code design remain intentionally marked as superseded history.
- 2026-08-27: Continued EP-013 rather than creating another plan because the
  user explicitly requested remediation within the existing plan. The shipped
  version-only entrypoints were replaced with explicit, bounded commands.
- 2026-08-27: Added `OpenAiAgentModel`, a loopback-only wallet HTTP transport,
  and an OpenSSH session adapter. The SSH adapter is confirmation-gated and no
  real remote operation was executed.
- 2026-08-27: Added `ADAD_REQUIRE_VAULT=1` so release CI cannot turn missing
  Linux loopback-vault prerequisites into a green skip; source-only Windows
  verification remains allowed to skip that Linux integration.
- 2026-08-27: Parallel workspace execution exposed an intermittent test-harness
  listener failure; isolated contract and egress tests passed. The mock server
  now retries transient listener errors, while no production transport behavior
  was weakened.
- 2026-08-27: Final preflight and source verification passed with exit 0. The
  production-readiness check correctly failed closed because the checkout is
  dirty and current image, Linux runtime, external-service, hardware, and
  performance evidence is not present; this plan is complete for bounded source
  remediation, not a production-readiness declaration.
- 2026-08-27: Continued the existing EP-013 instead of creating another plan,
  per the user instruction. The next locally verifiable audit gap is the
  missing production terminal/status adapter, so this continuation preserves
  the existing state/rendering contracts and adds only their runtime boundary.
- 2026-08-27: The repo-local llama.cpp fetcher now requires and verifies an
  explicit archive SHA-256 before extracting a release asset. A mutable release
  lookup without content verification would leave the image/runtime supply
  chain weaker than the newly pinned builder inputs.
- 2026-08-27: Updated CI and the environment registry to request Rust 1.90.0
  explicitly rather than the mutable stable channel, keeping hosted source and
  release jobs aligned with `rust-toolchain.toml`.
- 2026-08-27: Made the Debian snapshot a fixed Dockerfile environment value and
  fixed the builder platform to amd64 so the release path cannot silently select
  a different architecture or snapshot through an unpinned build argument.
- 2026-08-27: Strengthened the existing on-image leak battery so it verifies
  every shipped executable's help/dispatch surface and performs one real local
  `agent-coding chat` request through the supervised llama service before the
  final pass marker can be emitted.
- 2026-08-27: Registered the three runtime identity variables used by the
  shipped git-spoof command in `ENVIRONMENT.md`; the command remains explicitly
  dependent on persona/vault-provided pseudonymous values rather than inventing
  an identity or falling back to host metadata.
- 2026-08-27: Final verification after the terminal, image-battery, checksum,
  and toolchain edits passed `scripts/verify.sh` with exit 0. The standalone
  production-readiness gate still fails closed on the dirty working tree; no
  image, Linux backend, external-service, hardware, or performance evidence
  was claimed.
- 2026-08-27: Continued EP-013 with the existing `leakguard` boundary rather
  than creating another plan. Added a production `wg-quick` lifecycle adapter
  that consumes only the restricted `/run/adad/wg0.conf` path, verifies `ip`
  and `wg` observations, cleans up failed bring-up, and reports unknown when
  Linux observers are unavailable. Real interface and packet-leak validation
  remains image/operator-gated.
- 2026-08-27: Continued EP-013 rather than creating another plan, per the user
  instruction. Added standard OpenAI-compatible function-call decoding and
  registered-tool request wiring while preserving ADAD's existing bounded
  registry/executor boundary. Native workspace execution is the next bounded
  source-backed increment; arbitrary shell and remote MCP execution remain
  outside this continuation until their safety/configuration contracts are
  explicitly established.
- 2026-08-27: Added the first concrete native workspace executor behind the
  existing registry and loop. It is intentionally read-only and rooted at the
  canonical current workspace, with traversal/symlink escape rejection,
  sensitive-name filtering, deterministic directory output, and bounded file
  and response sizes. This closes a safe subset of the agent execution gap
  without treating arbitrary shell execution or unconfigured MCP servers as
  an acceptable default.
- 2026-08-27: Continued EP-013 for the MCP client-side gap. The active spec
  requires official `rmcp` support but does not define a production config
  format or remote egress adapter, so this milestone is limited to explicitly
  supplied stdio configs and direct child-process transport. Remote HTTP/SSE
  remains open rather than being approximated with an invented URL or secret
  policy.
- 2026-08-27: Completed M11 with the pinned official `rmcp` stdio child-process
  adapter. A real integration test spawns the shipped `mcp-echo-server`
  executable, performs a registry-qualified tool call, bounds its payload, and
  closes the client. At M11 completion, HTTP/SSE remained outside that
  milestone because its production configuration and egress contract were still
  undefined; M23 later adds the bounded streamable-HTTP path.
- 2026-08-27: Reopened EP-013 for M12 rather than creating another plan. The
  shipped TUI has a real terminal loop but still calls the provider's direct
  streaming method, so its production path bypasses the bounded AgentLoop and
  workspace executor used by `chat`. M12 addresses that concrete reachability
  gap without introducing a new tool or transport surface.
- 2026-08-27: Completed M12 by routing interactive prompts through the existing
  AgentLoop transcript path, bounded workspace executor, and callback-aware
  provider adapter. Streaming tool-call/final-answer integration passed with
  two provider requests; no shell, write, network, or credential operation was
  added.
- 2026-08-27: Reopened EP-013 for M13 rather than creating another plan. The
  existing OpenSSH adapter had host-key and confirmation safeguards but invoked
  the destination directly; the active service contract requires SSH over Tor.
  M13 uses a fixed loopback SOCKS5 proxy command and deliberately has no direct
  fallback or real-remote test path.
- 2026-08-27: Reopened EP-013 for M14 rather than creating another plan. The
  wallet client previously accepted only loopback HTTP and therefore had no
  concrete Tor route for a configured remote wallet RPC. The new path will
  allow only `.onion` HTTP endpoints through the fixed Tor SOCKS listener;
  arbitrary hosts and schemes remain rejected.
- 2026-08-27: Completed M13 and M14. OpenSSH now invokes the shipped
  `vps-deploy tor-connect` helper as a fixed Tor SOCKS5 ProxyCommand, and the
  wallet transport routes validated `.onion` HTTP RPC through the same local
  SOCKS boundary. Both adapters reject clearnet fallback paths and were tested
  with loopback protocol fixtures only; no external service was contacted.
- 2026-08-27: Reopened EP-013 for M15 rather than creating another plan. The
  pure netlink state machine was not connected to a production event source,
  so the image now supervises a fixed `ip monitor link` adapter that flushes
  the existing nftables table on down/deleted events. Monitor termination is
  treated as unhealthy and is restartable by systemd; Linux packet-level
  validation remains external evidence.
- 2026-08-27: Updated ARCHITECTURE.md for M15 because the leakguard boundary
  now has a real Linux event-source adapter and a supervised nftables reaction;
  the original pure netlink policy model remains intact underneath it.
- 2026-08-27: Reopened EP-013 for M16 rather than creating another plan. The
  image recipe now installs and verifies the runtime programs required by the
  vault and network workflows; the builder container's package set is no
  longer treated as evidence about the target image.
- 2026-08-27: Reopened EP-013 for M17 rather than creating another plan. The
  existing Git metadata rewrite now has a bounded local commit adapter that
  sets stable persona fields and fixed UTC dates; remote push and arbitrary Git
  execution remain intentionally outside the contract.
- 2026-08-27: Extended source smoke and the release-image battery to create a
  disposable local Git repository and inspect the resulting commit metadata;
  version/help and pure renderer success are no longer the only Git smoke
  evidence.
- 2026-08-27: Reopened EP-013 for M18 rather than creating another plan. The
  DMS state machine now has a regular-file-only LUKS2 adapter with no-follow
  opening, durable chunked wiping, and read-back verification; automatic
  Tor-NTP acquisition, production device handling, and kexec remain open.
- 2026-08-27: Reopened EP-013 for M19 rather than creating another plan. The
  live image now receives explicit dated Debian mirrors for all build stages,
  with the snapshot recorded in provenance and enforced by readiness; mutable
  current-mirror resolution is no longer an implicit image input.
- 2026-08-27: Reopened EP-013 for M20 rather than creating another plan. The
  minimum-system simulator now distinguishes optional timing exploration from
  a required inference acceptance run; required runs fail before Docker when
  no reviewed model input exists and fail inside the runner on missing or
  unusable inference services.
- 2026-08-27: Reopened EP-013 for M21 rather than creating another plan. The
  full verifier now always invokes the E2E entrypoint; source-only mode may
  still omit image execution by contract, but absence of the acceptance
  harness itself is a verification failure.
- 2026-08-27: Reopened EP-013 for M22 rather than creating another plan. Forge
  now validates the vault image target and every existing parent with
  `symlink_metadata` before and after creation setup and before unlock/upgrade,
  accepting only regular files (or a missing create target). This keeps the
  image-only backend from following a symlink or operating on a real block
  device; production-device handling remains explicitly outside this
  source-level adapter.
- 2026-08-27: Reopened EP-013 for M23 rather than creating another plan. The
  active SPEC-003 requires MCP HTTP/SSE in addition to stdio, so the existing
  stdio-only adapter was incomplete. The implementation uses rmcp's
  streamable-HTTP transport, modern SSE responses, certificate-verified
  reqwest TLS with an explicitly installed ring provider, no ambient proxies
  or redirects, and the same fail-closed fallback egress state as provider
  calls. Loopback HTTP is limited to local services and fixtures; no remote
  service was contacted.
- 2026-08-27: Reopened EP-013 for M24 rather than creating another plan. The
  readiness script previously nested an unconfigured source-only verifier even
  when checking an image and leak marker, so release readiness could omit the
  required image and vault integration modes. The nested verifier now receives
  both release flags; the existing readiness provenance checks remain in force.
- 2026-08-27: Reopened EP-013 for M25 rather than creating another plan. The
  hosted image job previously had no source for the required reviewed GGUF
  model or llama.cpp archive checksum, so a clean checkout could not execute
  the image path. The workflow is now an explicit manual release path that
  stages the model over HTTPS, verifies its SHA-256, fetches the runtime with
  the required archive checksum, and only then invokes the existing builder.
- 2026-08-27: Reopened EP-013 for M26 rather than creating another plan. The
  shipped agent entrypoint was bypassing the existing provider selector and
  therefore ignored documented cloud-provider configuration. It now builds a
  validated runtime config from the documented environment and passes the
  selected provider/model metadata into chat, TUI, and status; fallback
  requests still use the client default-deny boundary.
- 2026-08-27: Reopened EP-013 for M27 rather than creating another plan. Review
  of the actual nftables command showed that `flush table` removes the rules
  in every chain, so the prior link-loss reaction could remove the very drop
  policy it was meant to enforce. The monitor now loads a separate fixed
  drop-only ruleset and the image battery checks that the WireGuard allowance
  is gone after a link-down event.
- 2026-08-27: Reopened EP-013 for M28 rather than creating another plan. The
  existing MetaFUSE policy remained useful but its executable had no mount
  path, so the Linux-only adapter reuses that policy behind a pinned
  `fuser = 0.18.0` dependency with read-only, no-device, no-setuid, and no-exec
  mount options. It rejects symlinks and special files, hides xattrs, and does
  not claim embedded payload rewriting or live image evidence.
- 2026-08-31: Reopened EP-013 for M29 rather than creating another plan after
  the GitHub Actions run for commit `03e865bd634af30b39226578dd6257cb5a47ec59`
  failed before Rust verification with `scripts/preflight.sh: Permission
  denied`. The repository tracked all POSIX shell entrypoints as `100644`.
  The remediation records their existing executable contract as `100755` and
  adds a CI guard; no verifier step is skipped or changed to a model-only pass.
- 2026-08-31: The first hosted run of the M29 commit confirmed the shell guard
  and then failed because the pinned Rust action installed only its minimal
  profile, leaving `cargo-fmt` unavailable to `scripts/format-check.sh`.
  The workflow now requests the existing verifier's `rustfmt` and `clippy`
  components explicitly in both source and release jobs.
- 2026-08-31: The second hosted run passed the shell and Rust-component guards,
  then Clippy 1.90 rejected the Linux MetaFUSE runtime's redundant crate-level
  `cfg` because its parent module is already target-gated. The duplicate
  attribute is removed; the module boundary remains the sole Linux compilation
  gate.
- 2026-08-31: The third hosted run passed the shell guard, toolchain setup, and
  most source tests, then `forge` attempted a privileged loop/LUKS operation on
  the GitHub runner because its test preflight checked only command presence.
  The shared disposable-image harness now probes the complete loop/LUKS/
  filesystem/mount lifecycle and reports unavailability before the test body;
  required release mode remains fail-closed through `ADAD_REQUIRE_VAULT=1`.
- 2026-08-31: The fourth hosted run passed the shell guard, toolchain setup, and
  source tests, then the static-musl build failed because `ring` could not find
  `x86_64-linux-musl-gcc`. Both CI jobs now install the existing Debian
  `musl-tools` package before Rust verification or release-image builds.
- 2026-08-31: The corrected hosted push run (GitHub Actions run 11) passed the
  shell-mode guard, pinned Rust components, full source verifier, static-musl
  build, security/dependency checks, and source-only E2E checks. The
  release-image job was skipped as designed because reviewed model/runtime
  inputs are manual workflow inputs; image and live-system evidence remain
  external release gates.
- 2026-08-31: M30 excludes only `.agent/state/last-result.env` from the
  readiness and image-builder clean-checks because the loop contract requires
  that tracked file to be rewritten after every session. All other source and
  untracked changes still invalidate source-to-artifact evidence; provenance
  and source/tree matching checks are unchanged.
- 2026-09-01: M31 validation passed with `scripts/verify.sh` (`verify: ok`),
  including shell/source checks and the existing image-battery contract checks.
  The actual package install and `ssh` assertion remain exercised only when a
  Linux image build is run; no remote host or provisioning action was performed.
- 2026-09-01: M31 adds the target-image `openssh-client` package required by
  the existing `vps-deploy` OpenSSH adapter and makes the on-image battery
  fail closed if `ssh` is absent. No remote host or provisioning action is
  performed by the source or image smoke checks.
- 2026-09-01: M32 closes a target-image dependency gap for the existing
  clearnet and interface-drop probes: `iputils-ping` is now installed, `ping`
  is required before either probe, and `scripts/verify.sh` asserts that the
  package and runtime check remain coupled. This improves test validity without
  changing the network policy or performing live traffic in this session.
- 2026-09-01: M32 validation passed with `preflight: ok`, `git diff --check`,
  the isolated workspace-executor regression test, and the full verifier
  (`image leak-probe dependency check: ok`; `verify: ok`).
- 2026-09-01: M33 closes a false-positive path in the on-image killswitch
  battery: it now requires an actual non-loopback interface and reports a
  failed down or restore transition instead of continuing to a pass marker.
  The source verifier couples these checks to the hook; live QEMU reaction
  evidence remains external.
- 2026-09-01: M33 validation passed with `preflight: ok`, `git diff --check`,
  the candidate bypass review, and the full verifier (`verify: ok`). Windows
  source execution still labels Linux musl/image checks as unavailable rather
  than treating them as live evidence.
- 2026-09-01: M34 closes the explicit target-image dependency gap for `sysctl`:
  `procps` is now in the image package list and the source verifier requires
  both the package and the hook assertion. This preserves the existing IPv6
  policy and does not claim live image evidence.
- 2026-09-01: M34 validation passed with `preflight: ok`, `git diff --check`,
  the package/runtime coupling review, and the full verifier (`verify: ok`).
- 2026-09-01: M35 closes a pipeline-status false positive in the on-image
  static-tool smoke: command failure is checked before the usage output is
  inspected. The source verifier requires this two-step check; no executable
  behavior or external operation is changed.
- 2026-09-01: M35 validation passed with `preflight: ok`, `git diff --check`,
  focused source review, and the full verifier (`verify: ok`), including the
  new image help-exit dependency check.
- 2026-09-01: M36 closes a false-positive hardening path: the boot service no
  longer emits the MAC-randomization marker after ignored interface failures,
  and the on-image MAC check cannot pass with only loopback. The existing
  fail-closed firewall remains in place when boot setup aborts.
- 2026-09-01: M36 validation passed with `preflight: ok`, `git diff --check`,
  focused caller/marker review, and the full verifier (`verify: ok`), including
  the MAC-randomization dependency check.
- 2026-09-01: M37 closes a local-provider trust-boundary prefix collision:
  `localhost.evil.example` and `127.0.0.1.evil.example` are no longer accepted
  as loopback endpoints, while exact IPv4, hostname, and IPv6 loopback forms
  with valid ports remain allowed.
- 2026-09-01: M37 validation passed with `preflight: ok`, `git diff --check`,
  focused configuration tests (8 passed), and the full verifier (`verify: ok`).
- 2026-08-31: After M30, the full isolated-cache verifier completed with
  `verify: ok`; the readiness gate still rejected the worktree because the
  implementation and plan edits were intentionally uncommitted, as required
  for a source-to-artifact change.

## 15. Outcomes & Retrospective

The bounded source-level remediation now makes executable reachability
observable, provides a real terminal/status boundary, binds the local model
runtime to image composition, removes the release CI vault skip, and connects
the existing leakguard boundary to a fail-closed WireGuard lifecycle adapter.
The image battery now exercises shipped command dispatch and one real local
agent request before it can emit a pass marker. The provider bridge now carries
standard function calls and correlated results through the existing loop policy,
and the shipped `chat` command now connects
that loop to bounded read-only workspace tools. M11 adds a concrete,
registry-qualified rmcp stdio transport with a real spawned executable test,
M12 routes the interactive TUI through the same bounded loop while preserving
streamed final-answer text, and M23 adds the active contract's streamable-HTTP
transport with fail-closed remote egress. Live remote HTTPS, WireGuard
activation, and packet-level validation remain external evidence. M15 now connects the shipped Linux link
monitor to the existing nftables drop table, M16 makes target-image
runtime dependencies explicit, M17 connects stable identity to a real bounded
local Git commit, M18 connects DMS expiry to a validated image-file header
target, and M19 pins target-image Debian inputs, but image/QEMU execution and
packet-level reaction evidence remain required. The audit's Linux FUSE, automatic Tor-NTP DMS scheduling, production
device destruction, panic/kexec backends, real wallet-over-Tor and VPS acceptance, and current
hardware/performance evidence remain open. M20 now prevents required local
inference timing from silently skipping, but it does not supply a model or
runtime artifact. M21 also makes absence of the E2E harness a verifier failure
rather than a green omission. The readiness gate continues to
fail closed until those production and external evidence gates are satisfied.
M22 also makes the forge image boundary fail closed before host utilities are
invoked, reducing the risk that an image-only operation is redirected to a
symlink or device target. This is a defensive source fix, not evidence for
production-device vault support. M25 makes the hosted image job's required
external inputs explicit and checksum-bound; it does not itself prove a clean
hosted run, a reviewed model's quality, or production readiness. M26 closes
the executable's provider-selection disconnect without claiming that an
environment variable is authoritative WireGuard evidence. M27 corrects the
dynamic killswitch reaction's fail-open table flush; live QEMU packet evidence
remains required. M28 adds the first concrete Linux MetaFUSE mount adapter and
keeps the pure policy as its source of scrubbed attributes. Source compilation
and policy tests pass, but live `/dev/fuse` mounting, on-image behavior, and
payload-level EXIF rewriting remain release evidence gaps.
M29 restores the tracked executable mode required by fresh Linux checkouts and
adds an early CI guard for that contract. It declares the format, lint, and
musl compiler inputs required by the pinned Rust toolchain, and its Forge test
preflight distinguishes command presence from a usable privileged runtime.
The local source verifier and hosted source-verify run 11 pass. The manual
release-image workflow remains input-gated, and image/live-system evidence is
still external.
M32 closes the image leak battery's missing `ping` dependency and adds a
source-level package/assertion regression check; the actual target image and
packet behavior remain release validation gates.
M33 makes the interface-drop section fail closed when its required interface
transition cannot be exercised; source checks pass only when the battery cannot
silently convert that missing exercise into a success marker.
M34 makes `procps` explicit for the hook's `sysctl` hardening checks and keeps
that target dependency paired with a source assertion.
M35 prevents the static-tool help pipeline from masking application failures;
the on-image smoke now validates both process success and usage output.
M36 makes MAC randomization and its boot/smoke markers fail closed when an
interface is absent or any down/address/up transition fails.
M37 makes local provider URL validation compare the parsed authority rather
than a raw host prefix, with exact-host and port regression coverage.
