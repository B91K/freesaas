#!/usr/bin/env bash
# First-time setup after cloning: fetch the editors and build the server.
set -euo pipefail
cd "$(dirname "$0")/.."
git submodule update --init --depth 1
cargo build
echo "Ready. Run: cargo run --bin freesaas -- examples/local-dev.yaml"
