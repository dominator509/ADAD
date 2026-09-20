# ADAD

ADAD (Amnesic Decentralized AI Development Environment) is an experimental
Debian-Live and Rust project for a local-first coding environment with an
explicit privacy and fail-closed networking design.

## Current status

ADAD is not production-ready and must not be described as a completed secure
operating system. The repository currently contains tested library contracts,
bounded production adapters, pure security/state models, a live-build recipe,
and static Rust tool builds. The shipped agent `chat` and interactive TUI now
drive the bounded loop with read-only workspace tools. The Linux MetaFUSE
adapter exposes a read-only scrubbed FUSE view; live `/dev/fuse` image
validation and several other Linux/external adapters still require production
evidence.

In particular, the following are release gates rather than claims established
by the unit suite:

- a real local `llama-server` runtime and end-to-end agent session;
- real WireGuard lifecycle and packet-level leak validation;
- live `/dev/fuse` FUSE mounting, production DMS/device destruction, panic/kexec,
  and packet-level netlink behavior;
- real wallet RPC and SSH transports behind human confirmation;
- zero-host-write, representative hardware, reproducibility, and performance
  evidence.

Do not use this project with a real device, persistent vault, wallet, or remote
host from an automated session.

## Development

Read [AGENTS.md](AGENTS.md) and [COMMANDS.md](COMMANDS.md) first. The supported
source verification command is:

```sh
scripts/verify.sh
```

It validates the Rust workspace and model-level tests. A release-shaped image
must be built separately with `scripts/build-image.sh`; its provenance and
on-image leak result are then checked by:

```sh
scripts/production-readiness-check.sh
```

Readiness requires a clean checkout, an image bound to the current Git source
tree, a matching SHA-256 digest in the on-image leak marker, and the external
validation listed in [PRODUCTION_READINESS.md](PRODUCTION_READINESS.md).

## Architecture

The active MCP protocol foundation is the official Rust MCP SDK (`rmcp`), with
ADAD-owned execution and policy logic in `agent-coding`. The local provider is
the default design; OpenAI-compatible and Venice fallbacks require validated
HTTPS and authorized WireGuard egress.

See [ARCHITECTURE.md](ARCHITECTURE.md), [SECURITY.md](SECURITY.md),
[HOW_TO_USE.md](HOW_TO_USE.md), and [RELEASE.md](RELEASE.md) for the current
boundaries and release process.

## License

MIT. See [LICENSE](LICENSE).
