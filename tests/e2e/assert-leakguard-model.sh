#!/usr/bin/env sh
# Model-level assertions for leakguard posture before the booted-image battery exists.
set -eu
cd "$(dirname "$0")/../.."

cargo test -p leakguard --test netlink_drop
cargo test -p leakguard --test routing
cargo test -p leakguard --test mac
cargo test -p leakguard --test dms

echo "leak battery: leakguard model assertions ok"
