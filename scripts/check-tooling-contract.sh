#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"

assert_contains() {
  local file="$1"
  local pattern="$2"
  if ! grep -Ev '^[[:space:]]*#' "$file" | grep -Eq "$pattern"; then
    echo "expected $file to match: $pattern" >&2
    exit 1
  fi
}

assert_not_contains() {
  local file="$1"
  local pattern="$2"
  if grep -Ev '^[[:space:]]*#' "$file" | grep -Eq "$pattern"; then
    echo "expected $file not to match: $pattern" >&2
    exit 1
  fi
}

assert_toml_table_not_contains() {
  local file="$1"
  local table_pattern="$2"
  local field_pattern="$3"
  if awk -v table_pattern="$table_pattern" -v field_pattern="$field_pattern" '
    /^[[:space:]]*#/ { next }
    /^[[:space:]]*\[/ {
      line = $0
      sub(/[[:space:]]*#.*/, "", line)
      in_table = (line ~ table_pattern)
      next
    }
    in_table && $0 ~ field_pattern { found = 1 }
    END { exit !found }
  ' "$file"; then
    echo "expected $file table $table_pattern not to match: $field_pattern" >&2
    exit 1
  fi
}

assert_toml_task_contains() {
  local file="$1"
  local task="$2"
  local pattern="$3"
  if ! awk -v task="$task" -v pattern="$pattern" '
    /^[[:space:]]*#/ { next }
    $0 == "[tasks." task "]" { in_task = 1; next }
    in_task && /^\[.*\]$/ { in_task = 0 }
    in_task && $0 ~ pattern { found = 1 }
    END { exit !found }
  ' "$file"; then
    echo "expected $file task $task to match: $pattern" >&2
    exit 1
  fi
}

assert_contains "$root_dir/mise.toml" '^\[tools\]$'
assert_contains "$root_dir/mise.toml" '^"cargo:cargo-nextest"[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"$'
assert_contains "$root_dir/mise.toml" '^"cargo:cargo-deny"[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"$'
assert_contains "$root_dir/mise.toml" '^"cargo:trunk"[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"$'
assert_contains "$root_dir/mise.toml" '^"cargo:wasm-pack"[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"$'
assert_toml_task_contains "$root_dir/mise.toml" 'check' '^cargo deny --locked check$'
assert_toml_task_contains "$root_dir/mise.toml" 'audit_deps' '^cargo deny --locked check$'
assert_contains "$root_dir/mise.toml" 'cargo tree --workspace --all-features -e features --locked'
assert_contains "$root_dir/mise.toml" 'cargo clippy --package console --all-targets --features persistence --locked -- -D warnings'
assert_contains "$root_dir/mise.toml" 'cargo nextest run --package console --all-targets --features persistence --profile ci --locked'
assert_contains "$root_dir/mise.toml" 'cargo test --package console --doc --features persistence --locked'
assert_contains "$root_dir/mise.toml" 'cargo doc --package console --no-deps --features persistence --locked'
assert_contains "$root_dir/mise.toml" 'wasm-pack test --headless --firefox console --test wasm --locked'
assert_contains "$root_dir/console/check.sh" 'mise run check_wasm'
assert_contains "$root_dir/console/check.sh" 'mise run test_persistence'
assert_contains "$root_dir/.github/workflows/test.yml" 'run: mise run check$'
assert_contains "$root_dir/.github/workflows/test.yml" 'run: mise run test_persistence$'
assert_contains "$root_dir/.github/workflows/test.yml" 'run: mise run check_wasm$'
assert_contains "$root_dir/.github/workflows/test.yml" 'run: mise run test_wasm$'
assert_not_contains "$root_dir/.github/workflows/test.yml" '(cargo-nextest|cargo-deny|nextest|trunk|wasm-pack)@[0-9]'
assert_not_contains "$root_dir/.github/workflows/test.yml" 'taiki-e/install-action'
assert_contains "$root_dir/.github/workflows/test.yml" 'uses: actions/checkout@[0-9a-f]{40}[[:space:]]+# v4$'
assert_contains "$root_dir/.github/workflows/test.yml" 'uses: dtolnay/rust-toolchain@[0-9a-f]{40}[[:space:]]+# 1[.]98[.]0$'
assert_contains "$root_dir/.github/workflows/test.yml" 'uses: Swatinem/rust-cache@[0-9a-f]{40}[[:space:]]+# v2$'
assert_contains "$root_dir/.github/workflows/test.yml" 'uses: jdx/mise-action@[0-9a-f]{40}[[:space:]]+# v3$'

assert_not_contains "$root_dir/Cargo.toml" '^[[:space:]]*criterion[[:space:]]*='
assert_not_contains "$root_dir/console/Cargo.toml" '^[[:space:]]*criterion([.]workspace)?[[:space:]]*='
assert_not_contains "$root_dir/lang/Cargo.toml" '^[[:space:]]*criterion([.]workspace)?[[:space:]]*='
assert_not_contains "$root_dir/Cargo.toml" '^\[profile\.ci\]$'
assert_contains "$root_dir/Cargo.toml" '^tokio[[:space:]]*=[[:space:]]*\{[^}]*version[[:space:]]*='
assert_not_contains "$root_dir/console/Cargo.toml" '^tokio[[:space:]]*=[[:space:]]*\{[^}]*version[[:space:]]*='
assert_not_contains "$root_dir/console/Cargo.toml" '^[[:space:]]*(dependencies[.])?tokio[.]version[[:space:]]*='
assert_not_contains "$root_dir/console/Cargo.toml" '^[[:space:]]*dependencies[.]tokio[[:space:]]*=[[:space:]]*\{[^}]*version[[:space:]]*='
assert_toml_table_not_contains "$root_dir/console/Cargo.toml" '^[[:space:]]*[[]([^]]+[.])?dependencies[.]tokio[]][[:space:]]*$' '^[[:space:]]*version[[:space:]]*='
assert_not_contains "$root_dir/console/Cargo.toml" '^tokio[[:space:]]*=[[:space:]]*\{[^}]*features[[:space:]]*=[[:space:]]*\[[^]]*"full"'
