#!/usr/bin/env sh
# Full local validation sequence. This is the `verify` command referenced by
# every ExecPlan's front matter. Runs the whole gate in order and stops at the
# first failure.
set -eu
cd "$(dirname "$0")/.."

scripts/preflight.sh
scripts/format-check.sh
scripts/lint.sh
scripts/typecheck.sh
scripts/test-unit.sh
scripts/test-integration.sh
scripts/build.sh
scripts/security-check.sh
scripts/dependency-audit.sh
grep -Fx 'cargo audit --deny warnings' scripts/dependency-audit.sh >/dev/null || {
  echo "ERROR: dependency audit is allowed to pass with warnings." >&2
  exit 1
}
grep -F '0\.22\.2' scripts/dependency-audit.sh >/dev/null || {
  echo "ERROR: dependency audit does not require the reviewed cargo-audit version." >&2
  exit 1
}
echo "dependency audit fail-closed check: ok"
grep -Fx '| cargo-audit | 0.22.2 | dependency vuln scan |' ENVIRONMENT.md >/dev/null || {
  echo "ERROR: environment contract leaves cargo-audit version mutable." >&2
  exit 1
}
grep -F 'The current source has no production DMS scheduler, live panic button, or' \
  docs/runbooks/dms-panic.md >/dev/null || {
  echo "ERROR: DMS runbook overstates unavailable live panic/DMS behavior." >&2
  exit 1
}
grep -F 'That record is not current release evidence' docs/EP-010-rollback-drill.md >/dev/null || {
  echo "ERROR: rollback runbook treats historical execution as current evidence." >&2
  exit 1
}
echo "documentation evidence-boundary check: ok"
grep -Fx '4. **Smoke** — on Linux, built musl binaries execute `--help` and safe local-only commands without a full boot. Non-Linux hosts report an explicit Linux-musl execution skip; native `cargo run ... -- --help` checks are separate and do not prove Linux execution.' TESTING.md >/dev/null || {
  echo "ERROR: testing documentation still describes the obsolete version-only smoke contract." >&2
  exit 1
}
grep -Fx -- '- `scripts/smoke-test.sh` executes each built tool with `--help` and safe local-only dispatch checks on Linux. Non-Linux hosts report an explicit Linux-musl execution skip; missing binaries or failed commands fail the smoke gate.' TESTING.md >/dev/null || {
  echo "ERROR: testing documentation does not describe the fail-closed smoke contract." >&2
  exit 1
}
grep -F 'let output_chain = chain_body(rules, "output").unwrap_or("");' crates/leakguard/src/egress.rs >/dev/null || {
  echo "ERROR: egress classification is not scoped to the nftables output chain." >&2
  exit 1
}
grep -F 'fn output_controls_cannot_be_satisfied_by_another_chain()' crates/leakguard/src/egress.rs >/dev/null || {
  echo "ERROR: egress classification lacks a cross-chain false-positive regression test." >&2
  exit 1
}
echo "smoke and egress documentation/runtime checks: ok"
grep -F 'if output_chain_has_drop_policy(&rules)' crates/agent-coding/src/health.rs >/dev/null || {
  echo "ERROR: production killswitch status is not scoped to the output chain." >&2
  exit 1
}
grep -F 'fn killswitch_status_requires_the_hooked_output_chain_policy()' crates/agent-coding/src/health.rs >/dev/null || {
  echo "ERROR: killswitch status lacks a cross-chain false-positive regression test." >&2
  exit 1
}
echo "status output-chain check: ok"
# Boot smoke must not pass on hardening markers alone. The boot service first
# executes every shipped binary's local help path and emits the application
# reachability marker only after all of them succeed.
grep -Fx 'for tool in forge leakguard agent-coding xmr-wallet vps-deploy persona metafuse git-spoof; do' \
  live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: boot hardening does not exercise every shipped binary." >&2
  exit 1
}
grep -Fx "printf '%s\\n' 'adad-tools: reachable' > /dev/console || true" \
  live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: boot hardening has no application reachability marker." >&2
  exit 1
}
grep -F "&& grep -q 'adad-tools: reachable' \"\$log\"; then" \
  tests/os/boot-smoke-inside.sh >/dev/null || {
  echo "ERROR: QEMU boot smoke does not require the application marker." >&2
  exit 1
}
grep -Fx 'if [ "$status" -ne 0 ] && [ "$status" -ne 124 ]; then' \
  tests/os/boot-smoke-inside.sh >/dev/null || {
  echo "ERROR: QEMU boot smoke ignores unexpected emulator exits." >&2
  exit 1
}
echo "boot application reachability check: ok"
# On-image nftables smoke must inspect the target table's output chain. A
# policy in input/forward or another table cannot authorize fallback egress.
grep -Fx 'extract_output_chain_from_stdin() {' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image nftables smoke has no output-chain extractor." >&2
  exit 1
}
output_chain_extract_count=$(grep -F 'extract_output_chain_from_stdin <<EOF' live-build/hooks/0100-adad-hardening.hook.chroot | wc -l | tr -d '[:space:]')
[ "$output_chain_extract_count" -eq 2 ] || {
  echo "ERROR: on-image nftables smoke does not inspect both normal and drop output chains." >&2
  exit 1
}
grep -F "output_chain_contains 'type filter hook output'" live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image nftables smoke does not require the normal output hook." >&2
  exit 1
}
grep -F 'drop-nft-output-hook' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image drop smoke does not require the output hook." >&2
  exit 1
}
echo "image output-chain scope check: ok"
grep -Fx '  cargo build --locked --workspace --release' scripts/build.sh >/dev/null || {
  echo "ERROR: native build is not locked to Cargo.lock." >&2
  exit 1
}
grep -Fx 'cargo build --locked --workspace --release --target x86_64-unknown-linux-musl' scripts/build.sh >/dev/null || {
  echo "ERROR: musl build is not locked to Cargo.lock." >&2
  exit 1
}
grep -Fx 'cargo clippy --locked --workspace --all-targets --all-features -- -D warnings' scripts/lint.sh >/dev/null || {
  echo "ERROR: lint is not locked to Cargo.lock." >&2
  exit 1
}
grep -Fx 'cargo test --locked --workspace --lib' scripts/test-unit.sh >/dev/null || {
  echo "ERROR: unit tests are not locked to Cargo.lock." >&2
  exit 1
}
grep -Fx 'cargo test --locked --workspace --tests' scripts/test-integration.sh >/dev/null || {
  echo "ERROR: integration tests are not locked to Cargo.lock." >&2
  exit 1
}
grep -Fx 'cargo check --locked --workspace --all-targets --all-features' scripts/typecheck.sh >/dev/null || {
  echo "ERROR: typecheck is not locked to Cargo.lock." >&2
  exit 1
}
grep -Fx '            "$cargo_bin" test --locked -p forge --tests' .github/workflows/ci.yml >/dev/null || {
  echo "ERROR: hosted vault integration is not locked to Cargo.lock." >&2
  exit 1
}
echo "cargo lockfile checks: ok"
# The fetcher uses the release tag in repository-relative paths. Keep this
# fail-closed regression in the source gate so a future edit cannot turn an
# operator-provided tag into a path traversal or recursive-delete target.
if ADAD_LLAMA_CPP_RELEASE_TAG='../escape' sh scripts/fetch-llama-cpp-runtime.sh >/dev/null 2>&1; then
  echo "ERROR: llama runtime input validation accepted a path-like release tag." >&2
  exit 1
fi
echo "llama runtime input validation: ok"
# The on-image leak battery uses ping as its controlled clearnet probe. Keep
# the target package list and runtime assertion coupled so a missing binary
# cannot be mistaken for a successful blocked-traffic test.
grep -Fx 'iputils-ping' live-build/config/package-lists/adad-base.list.chroot >/dev/null || {
  echo "ERROR: target image is missing the iputils-ping package required by the leak battery." >&2
  exit 1
}
grep -Fx 'require_cmd ping' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image leak battery does not require ping before using it." >&2
  exit 1
}
echo "image leak-probe dependency check: ok"
# The drop probe must exercise a real interface transition. A missing
# interface or ignored transition is not evidence that the killswitch reacted.
grep -Fx '[ -n "$drop_iface" ] || console_fail "drop-interface"' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image leak battery can pass without a non-loopback interface." >&2
  exit 1
}
grep -Fx 'ip link set dev "$drop_iface" down 2>/dev/null || console_fail "drop-interface-down"' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image leak battery does not fail when interface down fails." >&2
  exit 1
}
echo "image interface-drop dependency check: ok"
# The boot posture must not claim MAC randomization when there is no usable
# interface or when an assignment transition fails. Keep the boot marker and
# leak-battery assertion tied to those fail-closed checks.
grep -Fx 'mac_ifaces=0' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: boot hardening can claim MAC randomization without an interface." >&2
  exit 1
}
grep -Fx '  mac_ifaces=$((mac_ifaces + 1))' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: boot hardening does not count non-loopback interfaces." >&2
  exit 1
}
grep -Fx '  ip link set dev "$iface" down 2>/dev/null || exit 1' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: boot hardening can ignore a failed MAC transition." >&2
  exit 1
}
grep -Fx '  ip link set dev "$iface" address "$mac" 2>/dev/null || exit 1' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: boot hardening can ignore a failed MAC assignment." >&2
  exit 1
}
grep -Fx '  ip link set dev "$iface" up 2>/dev/null || exit 1' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: boot hardening can ignore a failed MAC restore." >&2
  exit 1
}
grep -Fx '[ "$mac_ifaces" -gt 0 ] || exit 1' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: boot hardening lacks a non-loopback-interface fail-closed check." >&2
  exit 1
}
grep -Fx 'mac_iface_count=0' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image MAC smoke can pass without observing an interface." >&2
  exit 1
}
grep -Fx '  mac_iface_count=$((mac_iface_count + 1))' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image MAC smoke does not count non-loopback interfaces." >&2
  exit 1
}
grep -Fx '[ "$mac_iface_count" -gt 0 ] || console_fail "mac-interface"' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image MAC smoke lacks a non-loopback-interface failure." >&2
  exit 1
}
echo "image mac-randomization dependency check: ok"
# The hardening hook uses sysctl for IPv6 policy checks. Keep its target
# package explicit so a missing command cannot masquerade as a passing probe.
grep -Fx 'procps' live-build/config/package-lists/adad-base.list.chroot >/dev/null || {
  echo "ERROR: target image is missing the procps package required by sysctl checks." >&2
  exit 1
}
grep -Fx 'require_cmd sysctl' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image hardening battery does not require sysctl before using it." >&2
  exit 1
}
echo "image sysctl dependency check: ok"
# Do not let a pipeline hide a non-zero application exit: the image smoke
# requires both a successful --help process and an actual usage surface.
grep -Fx '  help_output=$("/usr/local/bin/$tool" --help 2>/dev/null) || console_fail "help-$tool"' live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: on-image static-tool smoke can mask a failed --help exit." >&2
  exit 1
}
echo "image help-exit dependency check: ok"
# Runtime downloads are release inputs. Keep both metadata and asset fetches
# on HTTPS, including redirects, so a changed release response cannot add a
# cleartext egress path to the builder.
grep -Fx '  https://*) ;;' scripts/fetch-llama-cpp-runtime.sh >/dev/null || {
  echo "ERROR: llama runtime fetcher does not require HTTPS asset URLs." >&2
  exit 1
}
proto_count=$(grep -F -- "--proto '=https'" scripts/fetch-llama-cpp-runtime.sh | wc -l | tr -d '[:space:]')
redir_count=$(grep -F -- "--proto-redir '=https'" scripts/fetch-llama-cpp-runtime.sh | wc -l | tr -d '[:space:]')
[ "$proto_count" -ge 2 ] && [ "$redir_count" -ge 2 ] || {
  echo "ERROR: llama runtime downloads do not enforce HTTPS redirects." >&2
  exit 1
}
echo "llama runtime HTTPS transport check: ok"
grep -F 'const LEAKGUARD_COMMAND: &str = "/usr/local/bin/leakguard";' \
  crates/agent-coding/src/client.rs >/dev/null || {
  echo "ERROR: agent fallback egress is not bound to the fixed leakguard query." >&2
  exit 1
}
grep -F '.with_egress_state(SystemEgressState::new())' \
  crates/agent-coding/src/main.rs >/dev/null || {
  echo "ERROR: production agent does not inject live leakguard egress state." >&2
  exit 1
}
grep -F 'println!("egress={}", leakguard::system_status().label())' \
  crates/leakguard/src/main.rs >/dev/null || {
  echo "ERROR: leakguard egress status boundary is missing." >&2
  exit 1
}
echo "runtime egress authority check: ok"
grep -F '/usr/local/bin/leakguard egress status 2>/dev/null' \
  live-build/hooks/0100-adad-hardening.hook.chroot >/dev/null || {
  echo "ERROR: the on-image smoke does not exercise the egress status command." >&2
  exit 1
}
echo "image egress dispatch check: ok"
grep -F '"tui" =>' crates/xmr-wallet/src/main.rs >/dev/null || {
  echo "ERROR: xmr-wallet has no production TUI command." >&2
  exit 1
}
grep -F '"tui" =>' crates/vps-deploy/src/main.rs >/dev/null || {
  echo "ERROR: vps-deploy has no production TUI command." >&2
  exit 1
}
grep -F 'backend::CrosstermBackend' crates/xmr-wallet/src/tui/mod.rs >/dev/null || {
  echo "ERROR: xmr-wallet TUI is still headless-only." >&2
  exit 1
}
grep -F 'backend::CrosstermBackend' crates/vps-deploy/src/tui/mod.rs >/dev/null || {
  echo "ERROR: vps-deploy TUI is still headless-only." >&2
  exit 1
}
echo "wallet/vps terminal runtime check: ok"
# Image inputs are supplied through ignored build paths. Keep their resolved
# targets inside the checked-out tree and reject symlinks before the builder
# copies bytes into the release image.
grep -Fx 'repo_real=$(readlink -f -- "$repo") || {' scripts/build-image-inside.sh >/dev/null || {
  echo "ERROR: image builder does not resolve the checkout before copying release inputs." >&2
  exit 1
}
grep -Fx 'ensure_repo_path "$llama_runtime"' scripts/build-image-inside.sh >/dev/null || {
  echo "ERROR: image builder does not bind the llama runtime to the checkout." >&2
  exit 1
}
grep -Fx 'ensure_repo_path "$llama_model"' scripts/build-image-inside.sh >/dev/null || {
  echo "ERROR: image builder does not bind the model artifact to the checkout." >&2
  exit 1
}
grep -Fx '[ ! -L "$llama_model" ] || {' scripts/build-image-inside.sh >/dev/null || {
  echo "ERROR: image builder can follow a model symlink into an unreviewed path." >&2
  exit 1
}
grep -Fx 'runtime_symlink=$(find "$llama_runtime" -type l -print -quit)' scripts/build-image-inside.sh >/dev/null || {
  echo "ERROR: image builder does not reject symlinks in the runtime tree." >&2
  exit 1
}
echo "image input provenance check: ok"
# The minimum-system simulator can fetch a reviewed llama runtime from inside
# the repo-owned builder. Keep the builder's explicit network/archive tools
# coupled to that fetch path rather than relying on incidental base-image state.
grep -Fx "      curl \\" live-build/builder/Dockerfile >/dev/null || {
  echo "ERROR: image builder is missing curl for the llama HTTPS fetch path." >&2
  exit 1
}
grep -Fx "      unzip \\" live-build/builder/Dockerfile >/dev/null || {
  echo "ERROR: image builder is missing unzip for ZIP llama runtime archives." >&2
  exit 1
}
echo "image builder llama tools: ok"
# Keep GitHub Actions inputs immutable. The comments retain the human-facing
# release labels while the commit IDs make the workflow source reproducible.
for action_pin in \
  '        uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4' \
  '        uses: dtolnay/rust-toolchain@06e5a564a0556e338780f5aecf2e7dcc9b267f07 # 1.90.0' \
  '        uses: taiki-e/install-action@8e38755317fb11cc24a0cd3b573a64008362d207 # cargo-audit' \
  '        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4'
do
  grep -Fx "$action_pin" .github/workflows/ci.yml >/dev/null || {
    echo "ERROR: CI workflow contains an unpinned or unexpected action reference." >&2
    exit 1
  }
done
action_lines=$(grep -E '^[[:space:]]+uses:' .github/workflows/ci.yml || true)
while IFS= read -r action_line; do
  case "$action_line" in
    '        uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4'|'        uses: dtolnay/rust-toolchain@06e5a564a0556e338780f5aecf2e7dcc9b267f07 # 1.90.0'|'        uses: taiki-e/install-action@8e38755317fb11cc24a0cd3b573a64008362d207 # cargo-audit'|'        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4')
      ;;
    *)
      echo "ERROR: CI workflow contains an unexpected action reference: $action_line" >&2
      exit 1
      ;;
  esac
done <<EOF
$action_lines
EOF
checkout_ref_count=$(grep -F '          ref: ${{ github.event.pull_request.head.sha || github.sha }}' .github/workflows/ci.yml | wc -l | tr -d '[:space:]')
[ "$checkout_ref_count" -eq 2 ] || {
  echo "ERROR: CI checkout is not bound to the PR head or event SHA." >&2
  exit 1
}
echo "workflow action pins: ok"

# Public-repository dependency automation must cover every detected ecosystem
# and keep all third-party workflow actions immutable. Dependency review is
# intentionally pull-request-only; CodeQL uses advanced manual Rust setup
# because default setup does not cover this Rust-only repository.
for automation_file in \
  .github/dependabot.yml \
  .github/workflows/dependency-review.yml \
  .github/workflows/codeql.yml
do
  [ -f "$automation_file" ] || {
    echo "ERROR: required GitHub automation file is missing: $automation_file" >&2
    exit 1
  }
done
for ecosystem in cargo docker github-actions; do
  grep -Fx "  - package-ecosystem: $ecosystem" .github/dependabot.yml >/dev/null || {
    echo "ERROR: Dependabot coverage is missing ecosystem: $ecosystem" >&2
    exit 1
  }
done
grep -Fx '    directory: "/live-build/builder"' .github/dependabot.yml >/dev/null || {
  echo "ERROR: Docker Dependabot entry is not bound to the builder Dockerfile directory." >&2
  exit 1
}
grep -Fx '    target-branch: main' .github/dependabot.yml >/dev/null || {
  echo "ERROR: Dependabot update is not targeted at main." >&2
  exit 1
}
grep -Fx '    open-pull-requests-limit: 5' .github/dependabot.yml >/dev/null || {
  echo "ERROR: Dependabot update limit is not five." >&2
  exit 1
}
grep -Fx '          - minor' .github/dependabot.yml >/dev/null || {
  echo "ERROR: Dependabot minor updates are not grouped." >&2
  exit 1
}
grep -Fx '          - patch' .github/dependabot.yml >/dev/null || {
  echo "ERROR: Dependabot patch updates are not grouped." >&2
  exit 1
}
if grep -Eq '(^|[[:space:]])major([[:space:]]|$)' .github/dependabot.yml; then
  echo "ERROR: Dependabot major updates must remain ungrouped." >&2
  exit 1
fi
grep -Fx '  pull_request:' .github/workflows/dependency-review.yml >/dev/null || {
  echo "ERROR: dependency review is not pull-request-triggered." >&2
  exit 1
}
grep -Fx '      - main' .github/workflows/dependency-review.yml >/dev/null || {
  echo "ERROR: dependency review is not restricted to main-targeting PRs." >&2
  exit 1
}
if grep -Eq '^[[:space:]]+(push|schedule|workflow_dispatch):' .github/workflows/dependency-review.yml; then
  echo "ERROR: dependency review has an unexpected non-PR trigger." >&2
  exit 1
fi
grep -Fx '  contents: read' .github/workflows/dependency-review.yml >/dev/null || {
  echo "ERROR: dependency review contents permission is not read-only." >&2
  exit 1
}
grep -Fx '  pull-requests: read' .github/workflows/dependency-review.yml >/dev/null || {
  echo "ERROR: dependency review pull-request permission is not read-only." >&2
  exit 1
}
grep -Fx '        uses: actions/dependency-review-action@a1d282b36b6f3519aa1f3fc636f609c47dddb294 # v5.0.0' \
  .github/workflows/dependency-review.yml >/dev/null || {
  echo "ERROR: dependency review action is not pinned to the verified v5.0.0 commit." >&2
  exit 1
}
grep -Fx '          fail-on-severity: high' .github/workflows/dependency-review.yml >/dev/null || {
  echo "ERROR: dependency review does not fail on high severity vulnerabilities." >&2
  exit 1
}
grep -Fx '  security-events: write' .github/workflows/codeql.yml >/dev/null || {
  echo "ERROR: CodeQL cannot publish code-scanning results." >&2
  exit 1
}
grep -Fx '          build-mode: manual' .github/workflows/codeql.yml >/dev/null || {
  echo "ERROR: Rust CodeQL analysis is not using its required manual build mode." >&2
  exit 1
}
grep -Fx '        run: cargo build --locked --workspace' .github/workflows/codeql.yml >/dev/null || {
  echo "ERROR: CodeQL Rust build is not locked to Cargo.lock." >&2
  exit 1
}
for codeql_action_pin in \
  '        uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4' \
  '        uses: dtolnay/rust-toolchain@06e5a564a0556e338780f5aecf2e7dcc9b267f07 # 1.90.0' \
  '        uses: github/codeql-action/init@1c5b675653bb5c22dbe9b12b556ec555138e09fd # v4.38.1' \
  '        uses: github/codeql-action/analyze@1c5b675653bb5c22dbe9b12b556ec555138e09fd # v4.38.1'
do
  grep -Fx "$codeql_action_pin" .github/workflows/codeql.yml >/dev/null || {
    echo "ERROR: CodeQL workflow contains an unexpected or unpinned action reference." >&2
    exit 1
  }
done
echo "GitHub security and dependency automation checks: ok"

# Keep the audit executable itself immutable as well. The pinned action
# supports an exact semver tool selector and a no-fallback mode; accepting a
# registry-resolved latest binary would reintroduce a mutable CI input.
audit_tool_count=$(grep -F '          tool: cargo-audit@0.22.2' .github/workflows/ci.yml | wc -l | tr -d '[:space:]')
[ "$audit_tool_count" -eq 2 ] || {
  echo "ERROR: CI does not install the reviewed cargo-audit version in every job." >&2
  exit 1
}
audit_fallback_count=$(grep -F '          fallback: none' .github/workflows/ci.yml | wc -l | tr -d '[:space:]')
[ "$audit_fallback_count" -eq 2 ] || {
  echo "ERROR: CI cargo-audit installation can fall back to an unpinned tool." >&2
  exit 1
}
echo "cargo-audit input pin: ok"
# Required inference acceptance must enforce the product's documented lower
# bound; optional exploratory runs may still record out-of-band measurements.
grep -Fx '  minimum_tok_s=4.0' scripts/min-system-sim-inside.sh >/dev/null || {
  echo "ERROR: required inference acceptance has no SPEC-000 throughput floor." >&2
  exit 1
}
grep -F 'if [ "$require_inference" = "1" ] && ! awk' scripts/min-system-sim-inside.sh >/dev/null || {
  echo "ERROR: required inference acceptance does not guard its throughput floor." >&2
  exit 1
}
grep -F 'BEGIN { exit !(actual >= minimum) }' scripts/min-system-sim-inside.sh >/dev/null || {
  echo "ERROR: required inference acceptance does not enforce the throughput floor." >&2
  exit 1
}
echo "inference throughput gate: ok"
# Pull requests must exercise the real disposable LUKS runtime. Keep the
# source job's package installation and fail-closed environment coupled so a
# future workflow edit cannot restore a green privileged-test skip.
source_ci_block=$(awk '/^  source-verify:/{in_source=1} /^  release-image:/{in_source=0} in_source {print}' .github/workflows/ci.yml)
printf '%s\n' "$source_ci_block" | grep -Fx '      - name: Install vault integration tools' >/dev/null || {
  echo "ERROR: hosted source verification does not install vault integration tools." >&2
  exit 1
}
printf '%s\n' "$source_ci_block" | grep -Fx '        run: sudo apt-get update && sudo apt-get install -y cryptsetup e2fsprogs util-linux' >/dev/null || {
  echo "ERROR: hosted source verification does not install the complete vault toolchain." >&2
  exit 1
}
printf '%s\n' "$source_ci_block" | grep -Fx '      - name: Verify required vault integration' >/dev/null || {
  echo "ERROR: hosted source verification has no required vault integration step." >&2
  exit 1
}
printf '%s\n' "$source_ci_block" | grep -Fx '          ADAD_REQUIRE_VAULT: "1"' >/dev/null || {
  echo "ERROR: hosted source verification can silently skip vault integration." >&2
  exit 1
}
printf '%s\n' "$source_ci_block" | grep -Fx '            "$cargo_bin" test --locked -p forge --tests' >/dev/null || {
  echo "ERROR: hosted source verification does not execute the Forge integration suite." >&2
  exit 1
}
echo "hosted vault integration gate: ok"
scripts/smoke-test.sh
# The E2E leak battery is a required repository control. It may explicitly omit
# the expensive image run during source-only verification, but a missing
# harness must never turn the full verifier green.
scripts/test-e2e.sh

echo "verify: ok"
