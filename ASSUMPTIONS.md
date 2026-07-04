# ASSUMPTIONS.md — ADAD

Assumptions made during blueprint generation. Each session that touches a
related area must verify the relevant assumption and update this file (confirm
or overturn) with a dated note.

| # | Assumption | Reason | Risk if wrong | How to verify | Blocks impl? |
|---|---|---|---|---|---|
| A1 | Base OS is Debian stable (bookworm or current stable) via `live-build`, hardened, not built from scratch. | User chose "hardened Debian Live base to build over." | Recipe/package names differ; hooks misfire. | `lb --version`; confirm suite in `live-build/config`. | No — EP-009 confirms. |
| A2 | Two claw-code crates are vendorable in isolation: an MCP integration crate and a tool-execution crate, at a pinnable commit, MIT-licensed. | Repo docs show `crates/mcp`, `crates/tools`, MIT license. | Crates may not compile standalone or may pull the cloud-first runtime. | In EP-002, clone at a pinned commit, `cargo build -p` each in isolation. | Partial — EP-002 has a STOP if they can't be isolated. |
| A3 | `llama-server` (llama.cpp) exposes an OpenAI-compatible `/v1/chat/completions` endpoint. | Standard llama.cpp server behavior. | Client shape wrong; local path fails. | `llama-server --help`; hit the endpoint against a tiny GGUF in EP-004. | No. |
| A4 | Venice API is OpenAI-compatible at `https://api.venice.ai/api/v1` with Bearer auth and its own model IDs; "private" vs "anonymized" model classes exist. | Confirmed via Venice docs during design. | Fallback client or privacy default wrong. | Re-read Venice docs at implementation; mock in EP-004 tests. | No. |
| A5 | Target hosts are x86_64 with AVX; 16–64 GB RAM; USB 3.2 Gen2 or NVMe-in-USB. | Stated constraints. | Perf targets unmet on weaker HW. | Document in ENVIRONMENT.md; perf smoke in EP-010. | No. |
| A6 | Static musl builds are achievable for all core crates (no crate forces glibc/dynamic). | Constraint requires it. | A needed crate breaks static build. | `scripts/build.sh` asserts static; per-crate check in EP-001/EP-002. | Partial — a hard-blocking crate is a STOP. |
| A7 | Monero access is via remote nodes through `monero-wallet-rpc` (no local full node). | Stated non-goal: no local node. | Wallet ops fail if RPC shape assumed wrong. | Confirm `monero-wallet-rpc` flags/JSON-RPC in EP-004 with a mock. | No. |
| A8 | VPS providers (1984.is/Njalla/BuyVM) accept SSH provisioning over Tor and XMR payment out-of-band. | Stated integrations. | Provider flow differs; automation stalls. | EP-004 uses a mock SSH target; real provisioning is a STOP (spends money). | No. |
| A9 | Git identity requirement = single stable pseudonym + stripped real metadata (NOT per-push rotation), and MAC handling = randomization (NOT impersonation). | User reframed goal to leak-prevention, not evasion. | Building rotation/impersonation would be out of scope and wrong. | SPEC-005; EP-006 asserts stable identity + randomized MAC. | No — settled. |
| A10 | No traditional DB; persistence is the LUKS2 vault + blockchain state. No schema migrations. | Stated stack. | Someone adds a DB/migration by habit. | ARCHITECTURE.md forbids it; review checklist. | No. |
| A11 | Tests never touch real devices/remotes/wallets; QEMU + loopback images + mocks only. | Safety + reproducibility. | A test could wipe a real disk or spend XMR. | AGENTS.md §13; scripts forbid device writes. | No — enforced. |
