#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"

make_fixture() {
  fixture_dir="$(mktemp -d)"
  mkdir -p "$fixture_dir/scripts" "$fixture_dir/.github/workflows" "$fixture_dir/console" "$fixture_dir/lang"
  cp "${CHECKER_SOURCE:-$repo_root/scripts/check-tooling-contract.sh}" "$fixture_dir/scripts/check-tooling-contract.sh"
  cp "$repo_root/mise.toml" "$repo_root/Cargo.toml" "$fixture_dir/"
  cp "$repo_root/console/Cargo.toml" "$fixture_dir/console/"
  cp "$repo_root/console/check.sh" "$fixture_dir/console/"
  cp "$repo_root/lang/Cargo.toml" "$fixture_dir/lang/"
  cp "$repo_root/.github/workflows/test.yml" "$fixture_dir/.github/workflows/"
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

test_prohibited_action_main_ref_is_rejected() {
  make_fixture
  printf '\n      - uses: taiki-e/install-action@main\n' >> "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "a prohibited action using an unpinned ref"
}

case "${1:-all}" in
  comments) test_commented_requirement_is_rejected ;;
  dotted-dependency) test_dotted_dependency_version_is_rejected ;;
  dependency-table) test_dependency_table_version_is_rejected ;;
  prohibited-action) test_prohibited_action_main_ref_is_rejected ;;
  all)
    test_commented_requirement_is_rejected
    test_dotted_dependency_version_is_rejected
    test_dependency_table_version_is_rejected
    test_prohibited_action_main_ref_is_rejected
    ;;
  *) echo "unknown test: $1" >&2; exit 2 ;;
esac
