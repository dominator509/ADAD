# EP-010 Performance Review

## Scope
This note records the EP-010 M3 performance evidence gathered from repository
commands and tests. Real provider, paid infrastructure, and physical-device
checks remain human-gated.

## Results
- Killswitch latency: passed.
  - Command: `cargo test -p leakguard --test netlink_drop`
  - Result: 2 passed, 0 failed.
  - Target: `DROP_ALL_TARGET_LATENCY` is 250 ms.
- VPS provisioning mock timing: passed.
  - Command: `cargo test -p vps-deploy --test vps_mock`
  - Result: 2 passed, 0 failed.
  - Target: mock provisioning elapsed time under 120 seconds.

## Unmeasured Item
- Local inference tok/s was not measured in this session.
  - Reason: the repository documents the `llama-server` endpoint and command
    shape, but it does not include a benchmark command, a pinned GGUF fixture,
    or a documented throughput band beyond the checklist requirement.
  - Risk: production readiness cannot claim a measured local inference tok/s
    value until a human supplies an approved model fixture or the repo adds a
    mockable benchmark harness.
  - Launch handling: keep this as an EP-010 residual risk unless a later
    milestone adds an allowed, reproducible perf smoke command.
