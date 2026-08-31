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
assert_toml_task_contains "$root_dir/mise.toml" 'check' '^mise run check_pull_request$'
assert_toml_task_contains "$root_dir/mise.toml" 'check' '^mise run check_merge$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo fmt --all -- --check$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo clippy --workspace --all-targets --locked -- -D warnings$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo nextest run --workspace --profile ci --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo test --workspace --doc --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_merge' '^[[:space:]]*mise run check_merge_native$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_merge' '^[[:space:]]*mise run check_wasm$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_merge' '^[[:space:]]*mise run test_wasm$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_merge_native' '^mise run test_persistence$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_merge_native' '^cargo deny --locked check$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_merge_native' '^RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'audit_deps' '^cargo deny --locked check$'
assert_toml_task_contains "$root_dir/mise.toml" 'audit_deps' '^cargo tree --workspace --all-features -e features --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'test_persistence' '^cargo check --package orcvs --lib --features persistence --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'test_persistence' '^cargo clippy --workspace --all-targets --features persistence --locked -- -D warnings$'
assert_toml_task_contains "$root_dir/mise.toml" 'test_persistence' '^cargo nextest run --workspace --all-targets --features persistence --profile ci --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'test_persistence' '^cargo test --workspace --doc --features persistence --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'test_persistence' '^RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --features persistence --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_wasm' '^env -u NO_COLOR trunk build --features persistence --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'test_wasm' '^run = .wasm-pack test --headless --firefox shell --test wasm --features persistence --locked.$'
assert_contains "$root_dir/shell/check.sh" 'mise run check_wasm'
assert_contains "$root_dir/shell/check.sh" 'mise run test_persistence'
assert_contains "$root_dir/shell/Trunk.toml" '^filehash[[:space:]]*=[[:space:]]*false$'
assert_contains "$root_dir/shell/assets/sw.js" "'./shell.js'"
assert_contains "$root_dir/shell/assets/sw.js" "'./shell_bg.wasm'"
assert_contains "$root_dir/.vscode/launch.json" '"--package=orcvs",'
assert_contains "$root_dir/.vscode/launch.json" '"--package=shell"'
assert_not_contains "$root_dir/.vscode/launch.json" '(package|bin)=console'
assert_not_contains "$root_dir/.vscode/launch.json" '(package|bin)=vtha|parser_benchmark'
assert_contains "$root_dir/.github/workflows/test.yml" 'run: mise run check_pull_request$'
assert_contains "$root_dir/.github/workflows/test.yml" 'ORCVS_MERGE_COMPONENT: native$'
assert_contains "$root_dir/.github/workflows/test.yml" 'ORCVS_MERGE_COMPONENT: wasm$'
assert_contains "$root_dir/.github/workflows/test.yml" 'run: mise run check_merge$'
assert_contains "$root_dir/.github/workflows/test.yml" "if: github.event_name == 'push'$"
assert_not_contains "$root_dir/.github/workflows/test.yml" '(cargo-nextest|cargo-deny|nextest|trunk|wasm-pack)@[0-9]'
assert_not_contains "$root_dir/.github/workflows/test.yml" 'taiki-e/install-action'
assert_contains "$root_dir/.github/workflows/test.yml" 'uses: actions/checkout@[0-9a-f]{40}[[:space:]]+# v4$'
assert_contains "$root_dir/.github/workflows/test.yml" 'uses: dtolnay/rust-toolchain@[0-9a-f]{40}[[:space:]]+# 1[.]98[.]0$'
assert_contains "$root_dir/.github/workflows/test.yml" 'uses: Swatinem/rust-cache@[0-9a-f]{40}[[:space:]]+# v2$'
assert_contains "$root_dir/.github/workflows/test.yml" 'uses: jdx/mise-action@[0-9a-f]{40}[[:space:]]+# v3$'

assert_not_contains "$root_dir/Cargo.toml" '^[[:space:]]*criterion[[:space:]]*='
assert_not_contains "$root_dir/shell/Cargo.toml" '^[[:space:]]*criterion([.]workspace)?[[:space:]]*='
assert_not_contains "$root_dir/orcvs/Cargo.toml" '^[[:space:]]*criterion([.]workspace)?[[:space:]]*='
assert_not_contains "$root_dir/lang/Cargo.toml" '^[[:space:]]*criterion([.]workspace)?[[:space:]]*='
assert_not_contains "$root_dir/Cargo.toml" '^\[profile\.ci\]$'
assert_contains "$root_dir/Cargo.toml" '^tokio[[:space:]]*=[[:space:]]*\{[^}]*version[[:space:]]*='
for manifest in "$root_dir/orcvs/Cargo.toml" "$root_dir/shell/Cargo.toml"; do
  assert_not_contains "$manifest" '^tokio[[:space:]]*=[[:space:]]*\{[^}]*version[[:space:]]*='
  assert_not_contains "$manifest" '^[[:space:]]*(dependencies[.])?tokio[.]version[[:space:]]*='
  assert_not_contains "$manifest" '^[[:space:]]*dependencies[.]tokio[[:space:]]*=[[:space:]]*\{[^}]*version[[:space:]]*='
  assert_toml_table_not_contains "$manifest" '^[[:space:]]*[[]([^]]+[.])?dependencies[.]tokio[]][[:space:]]*$' '^[[:space:]]*version[[:space:]]*='
  assert_not_contains "$manifest" '^tokio[[:space:]]*=[[:space:]]*\{[^}]*features[[:space:]]*=[[:space:]]*\[[^]]*"full"'
done
