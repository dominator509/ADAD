#!/usr/bin/env sh
# Agent client assertion: unsafe fallback is blocked before socket write.
set -eu
cd "$(dirname "$0")/../.."

cargo test -p agent-coding --test egress_guard

echo "leak battery: agent egress guard assertions ok"
