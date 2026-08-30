#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
fixture_dirs=()

cleanup() {
  local dir
  for dir in "${fixture_dirs[@]}"; do
    rm -rf "$dir"
  done
}
trap cleanup EXIT

make_fixture() {
  fixture_dir="$(mktemp -d)"
  fixture_dirs+=("$fixture_dir")
  mkdir -p "$fixture_dir/scripts" "$fixture_dir/.github/workflows" "$fixture_dir/console" "$fixture_dir/lang"
  cp "${CHECKER_SOURCE:-$repo_root/scripts/check-tooling-contract.sh}" "$fixture_dir/scripts/check-tooling-contract.sh"
  cp "$repo_root/mise.toml" "$repo_root/Cargo.toml" "$fixture_dir/"
  cp "$repo_root/console/Cargo.toml" "$fixture_dir/console/"
  cp "$repo_root/console/check.sh" "$fixture_dir/console/"
  cp "$repo_root/lang/Cargo.toml" "$fixture_dir/lang/"
  cp "$repo_root/.github/workflows/test.yml" "$fixture_dir/.github/workflows/"
  if ! bash "$fixture_dir/scripts/check-tooling-contract.sh" >/dev/null; then
    echo "fresh tooling-contract fixture does not satisfy the contract" >&2
    return 1
  fi
}

assert_rejected() {
  local scenario="$1"
  local output
  if output="$(bash "$fixture_dir/scripts/check-tooling-contract.sh" 2>&1)"; then
    echo "expected tooling contract to reject $scenario" >&2
    return 1
  fi
  printf '%s\n' "$output"
}

test_commented_requirement_is_rejected() {
  make_fixture
  perl -pi -e 's/^(RUSTDOCFLAGS="-D warnings" cargo doc --package console --no-deps --features persistence --locked)$/# $1/' "$fixture_dir/mise.toml"
  assert_rejected "a commented-out required setting"
}

test_unlocked_check_deny_is_rejected() {
  make_fixture
  perl -pi -e 'if (!$done && s/cargo deny --locked check/cargo deny check/) { $done = 1 }' "$fixture_dir/mise.toml"
  assert_rejected "an unlocked cargo-deny invocation in the repository check task"
}

test_unlocked_audit_deny_is_rejected() {
  make_fixture
  perl -pi -e 'if (/cargo deny --locked check/ && ++$seen == 2) { s/cargo deny --locked check/cargo deny check/ }' "$fixture_dir/mise.toml"
  assert_rejected "an unlocked cargo-deny invocation in the dependency audit task"
}

test_unlocked_wasm_pack_is_rejected() {
  make_fixture
  perl -pi -e 's/wasm-pack test --headless --firefox console --test wasm --locked/wasm-pack test --headless --firefox console --test wasm/' "$fixture_dir/mise.toml"
  assert_rejected "an unlocked wasm-pack test invocation"
}

test_persistence_command_in_wrong_task_is_rejected() {
  make_fixture
  perl -0pi -e 's/(\[tasks\.test_persistence\].*?)cargo clippy --package console --all-targets --features persistence --locked -- -D warnings\n/$1/s' "$fixture_dir/mise.toml"
  printf '\ncargo clippy --package console --all-targets --features persistence --locked -- -D warnings\n' >> "$fixture_dir/mise.toml"
  assert_rejected "a persistence command outside test_persistence"
}

test_dotted_dependency_version_is_rejected() {
  make_fixture
  perl -pi -e 'if (!$done && s/^tokio = \{ workspace = true, features = \["rt", "macros", "time"\] \}$/tokio.workspace = true\ntokio.version = "9.0.0"/) { $done = 1 }' "$fixture_dir/console/Cargo.toml"
  assert_rejected "a crate-local dependency version in dotted TOML syntax"
}

test_dependency_table_version_is_rejected() {
  make_fixture
  perl -pi -e 'if (!$done && s/^tokio = \{ workspace = true, features = \["rt", "macros", "time"\] \}$/[dependencies.tokio]\nworkspace = true\nversion = "9.0.0"/) { $done = 1 }' "$fixture_dir/console/Cargo.toml"
  assert_rejected "a crate-local dependency version in TOML table syntax"
}

test_commented_dependency_table_version_is_rejected() {
  make_fixture
  perl -pi -e 'if (!$done && s/^tokio = \{ workspace = true, features = \["rt", "macros", "time"\] \}$/[dependencies.tokio] # local override\nworkspace = true\nversion = "9.0.0"/) { $done = 1 }' "$fixture_dir/console/Cargo.toml"
  assert_rejected "a crate-local dependency version in a TOML table with a trailing comment"
}

test_prohibited_action_main_ref_is_rejected() {
  make_fixture
  printf '\n      - uses: taiki-e/install-action@main\n' >> "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "a prohibited action using an unpinned ref"
}

test_invalid_fresh_fixture_is_rejected() {
  if CHECKER_SOURCE="$repo_root/Cargo.toml" make_fixture 2>/dev/null; then
    echo "expected fixture setup to reject an invalid fresh fixture" >&2
    return 1
  fi
}

case "${1:-all}" in
  comments) test_commented_requirement_is_rejected ;;
  unlocked-check-deny) test_unlocked_check_deny_is_rejected ;;
  unlocked-audit-deny) test_unlocked_audit_deny_is_rejected ;;
  unlocked-wasm-pack) test_unlocked_wasm_pack_is_rejected ;;
  dotted-dependency) test_dotted_dependency_version_is_rejected ;;
  dependency-table) test_dependency_table_version_is_rejected ;;
  commented-dependency-table) test_commented_dependency_table_version_is_rejected ;;
  prohibited-action) test_prohibited_action_main_ref_is_rejected ;;
  invalid-fixture) test_invalid_fresh_fixture_is_rejected ;;
  misplaced-persistence) test_persistence_command_in_wrong_task_is_rejected ;;
  all)
    test_invalid_fresh_fixture_is_rejected
    test_commented_requirement_is_rejected
    test_unlocked_check_deny_is_rejected
    test_unlocked_audit_deny_is_rejected
    test_unlocked_wasm_pack_is_rejected
    test_persistence_command_in_wrong_task_is_rejected
    test_dotted_dependency_version_is_rejected
    test_dependency_table_version_is_rejected
    test_commented_dependency_table_version_is_rejected
    test_prohibited_action_main_ref_is_rejected
    ;;
  *) echo "unknown test: $1" >&2; exit 2 ;;
esac
