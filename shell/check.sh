#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."
mise run check
mise run test_persistence
mise run check_wasm
