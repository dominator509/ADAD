# ENVIRONMENT.md — ADAD

## Required tools
| Tool | Version (min) | Purpose |
|---|---|---|
| rustup + cargo | stable (1.75+) | build the Rust workspace |
| musl target | `x86_64-unknown-linux-musl` | static core binaries |
| cargo-audit | latest | dependency vuln scan |
| cryptsetup | current | LUKS2 vault create/unlock/lock lifecycle in EP-003 |
| util-linux (`losetup`, `mount`, `umount`) | current | loopback devices + filesystem mount cycle in EP-003 |
| e2fsprogs (`mkfs.ext4`) | current | format the vault filesystem in EP-003 tests |
| coreutils (`truncate`) | current | create sparse vault images in EP-003 tests |
| live-build (`lb`) | Debian current | assemble the bootable image (EP-009) |
| squashfs-tools | current | SquashFS for the live image |
| qemu-system-x86 | current | OS boot + leak-battery testing |
| git | 2.x | version control; git-spoof wraps it |

Host OS for building: Debian/Ubuntu x86_64. Agents do not `apt install` in a
session; missing host tools are a STOP (see `scripts/install.sh`).

When the user explicitly resolves the EP-009 builder blocker, the repo-owned
container builder may be prepared with `scripts/build-image-builder.sh` and
verified with `scripts/check-image-builder.sh`. That installs the required
image-build tools inside the `adad-ep009-builder:local` Docker image only; it
does not install host packages and still writes no physical device.

## Package manager
`cargo` for crates; `apt` for host packages (human-run only).

## Environment variables
Exact values are confirmed per-crate as they are implemented; this table is the
authoritative registry. Anything not here must be added here (with evidence)
before use. Secrets are NEVER committed; they are loaded from the vault into the
process environment at runtime.

| Name | Req/Opt | Env | Example | Secret? | Description | Validation |
|---|---|---|---|---|---|---|
| `ADAD_VAULT_PATH` | required | all | `/run/adad/vault.img` (test) | no | Path to the LUKS2 vault image/device. | must exist; must be image in tests |
| `ADAD_PROVIDER` | optional | runtime | `local` | no | `local`\|`openai`\|`venice`. Default `local`. | one of the three |
| `ADAD_LOCAL_BASE_URL` | optional | runtime | `http://127.0.0.1:8080/v1` | no | llama-server endpoint. | loopback only |
| `ADAD_OPENAI_BASE_URL` | optional | runtime | `https://api.example.com/v1` | no | OpenAI-compatible fallback base URL. | https; routed via WireGuard |
| `ADAD_VENICE_BASE_URL` | optional | runtime | `https://api.venice.ai/api/v1` | no | Venice base URL. | https; routed via WireGuard |
| `ADAD_VENICE_ALLOW_ANONYMIZED` | optional | runtime | `false` | no | Opt-in to Venice anonymized models. | default false; warns if true |
| `OPENAI_API_KEY` | optional | runtime | `sk-...` | YES | Key for OpenAI-compatible fallback. | loaded from vault only |
| `VENICE_API_KEY` | optional | runtime | `vapi_...` | YES | Key for Venice fallback. | loaded from vault only |
| `ADAD_MODEL` | optional | runtime | `qwen2.5-coder` (local) | no | Model id for the active provider. | non-empty |
| `ADAD_DMS_WINDOW_HOURS` | optional | runtime | `72` | no | Dead Man's Switch window. | positive integer |
| `ADAD_WG_CONF` | optional | runtime | `/run/adad/wg0.conf` | YES | WireGuard config (contains private key). | vault-loaded; never committed |
| `MONERO_RPC_URL` | optional | runtime | `http://127.0.0.1:18082/json_rpc` | no | monero-wallet-rpc endpoint (over Tor). | via Tor only |

> This table needs per-crate confirmation as crates are implemented. Each
> ExecPlan that reads a variable must confirm the name here first.

## Secrets
Loaded from the vault at unlock into the process environment; scrubbed on lock/
shutdown. Never written to host disk, never logged, never committed.

## Local development setup
1. Install rustup; `rustup target add x86_64-unknown-linux-musl`.
2. `cargo install cargo-audit`.
3. Install host tools (`cryptsetup`, `util-linux`, `e2fsprogs`, `coreutils`, `live-build`, `squashfs-tools`, `qemu-system-x86`).
4. `scripts/preflight.sh` then `scripts/install.sh`.

Containerized image-builder setup for EP-009:
1. `scripts/build-image-builder.sh`
2. `scripts/check-image-builder.sh`
3. `scripts/build-image.sh` to produce `build/adad.img`

## Local database setup
Not applicable (no database).

## Test environment setup
Loopback LUKS images + QEMU + mock servers, all created at test time. No real
devices or remotes. EP-003 vault tests require a Linux host that already
provides `cryptsetup`, `losetup`, `mkfs.ext4`, `mount`, `umount`, and
`truncate`; agents do not install them in-session. A tiny GGUF is used for
inference tests.

## Staging environment setup
"Staging" = a QEMU VM booting the built image with a monitored NIC/disk. No
cloud staging.

## Production environment setup
A physical USB 3.2 Gen2 / NVMe-in-USB drive imaged from the release artifact,
booted on x86_64 (16–64 GB RAM, AVX). Imaging is a human runbook (DEPLOYMENT.md).

## Configuration validation
`adad-core` validates config on load and fails closed on unknown/invalid keys.
`ADAD_PROVIDER=venice` with anonymized models requires
`ADAD_VENICE_ALLOW_ANONYMIZED=true` or it refuses.

## Environment parity rules
Test, staging (QEMU), and production run the SAME static binaries and the SAME
image recipe. The only differences are the vault backing (image vs device) and
whether providers are mocked.

## Troubleshooting
- `cargo not found` → install rustup, re-run.
- static build fails → a dependency pulled glibc; find and replace or feature-
  gate it (EP-001/EP-002 record such cases).
- QEMU has no KVM → tests run slower under TCG; allowed, just slower.
