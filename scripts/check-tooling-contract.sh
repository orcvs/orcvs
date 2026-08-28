#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"

assert_contains() {
  local file="$1"
  local pattern="$2"
  if ! grep -Eq "$pattern" "$file"; then
    echo "expected $file to match: $pattern" >&2
    exit 1
  fi
}

assert_not_contains() {
  local file="$1"
  local pattern="$2"
  if grep -Eq "$pattern" "$file"; then
    echo "expected $file not to match: $pattern" >&2
    exit 1
  fi
}

assert_contains "$root_dir/mise.toml" '^\[tools\]$'
assert_contains "$root_dir/mise.toml" '^"cargo:cargo-nextest" = "0\.9\.137"$'
assert_contains "$root_dir/mise.toml" '^"cargo:cargo-deny" = "0\.20\.2"$'
assert_contains "$root_dir/mise.toml" '^"cargo:trunk" = "0\.21\.14"$'
assert_contains "$root_dir/mise.toml" 'cargo deny check'
assert_contains "$root_dir/console/check.sh" 'mise run check_wasm'
assert_contains "$root_dir/console/check.sh" 'mise run test_persistence'

assert_not_contains "$root_dir/Cargo.toml" '^criterion = '
assert_not_contains "$root_dir/console/Cargo.toml" '^criterion\.workspace = true$'
assert_not_contains "$root_dir/lang/Cargo.toml" '^criterion\.workspace = true$'
assert_not_contains "$root_dir/Cargo.toml" '^\[profile\.ci\]$'
