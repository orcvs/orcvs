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

assert_occurs_exactly() {
  local file="$1"
  local pattern="$2"
  local expected="$3"
  local actual
  actual="$(grep -Ev '^[[:space:]]*#' "$file" | grep -Ec "$pattern" || true)"
  if [ "$actual" -ne "$expected" ]; then
    echo "expected $file to match $expected times, matched $actual: $pattern" >&2
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

# Both table assertions share one scan: skip comment lines, track whether the
# current table header matches, and report whether any line inside it matched.
# Exit status carries the answer so each assertion only supplies its condition.
toml_table_matches() {
  local file="$1"
  local table_pattern="$2"
  local field_pattern="$3"
  awk -v table_pattern="$table_pattern" -v field_pattern="$field_pattern" '
    /^[[:space:]]*#/ { next }
    /^[[:space:]]*\[/ {
      line = $0
      sub(/[[:space:]]*#.*/, "", line)
      in_table = (line ~ table_pattern)
      next
    }
    in_table && $0 ~ field_pattern { found = 1 }
    END { exit !found }
  ' "$file"
}

assert_toml_table_not_contains() {
  local file="$1"
  local table_pattern="$2"
  local field_pattern="$3"
  if toml_table_matches "$file" "$table_pattern" "$field_pattern"; then
    echo "expected $file table $table_pattern not to match: $field_pattern" >&2
    exit 1
  fi
}

assert_toml_table_contains() {
  local file="$1"
  local table_pattern="$2"
  local field_pattern="$3"
  if ! toml_table_matches "$file" "$table_pattern" "$field_pattern"; then
    echo "expected $file table $table_pattern to match: $field_pattern" >&2
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
assert_toml_task_contains "$root_dir/mise.toml" 'check_wasm' '^env -u NO_COLOR trunk build --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'test_wasm' '^run = .wasm-pack test --headless --firefox shell --test wasm --features persistence --locked.$'
assert_toml_task_contains "$root_dir/mise.toml" 'bench' '^run = .cargo bench --package lang --package orcvs --benches --locked -- --output-format bencher.$'
# The measurement is only compared when the workflow runs, so every path that can
# move a number has to trigger it: the two benchmarked crates included.
assert_contains "$root_dir/.github/workflows/bench.yml" "^      - 'lang/[*][*]'$"
assert_contains "$root_dir/.github/workflows/bench.yml" "^      - 'orcvs/[*][*]'$"
assert_contains "$root_dir/.github/workflows/bench.yml" '^        run: mise run bench [|] tee output[.]txt$'
# `orcvs` links ALSA through `midir` on Linux, so every bench job needs the same
# native dependency the test workflow installs. The count is derived from the jobs
# the workflow actually declares: a single install step satisfies no more than one
# of them, and a job added without one fails this check rather than failing in CI.
bench_job_count="$(awk '
  /^[[:space:]]*#/ { next }
  /^jobs:[[:space:]]*$/ { in_jobs = 1; next }
  in_jobs && /^[^[:space:]]/ { in_jobs = 0 }
  in_jobs && /^  [A-Za-z0-9_-]+:[[:space:]]*$/ { count++ }
  END { print count + 0 }
' "$root_dir/.github/workflows/bench.yml")"
if [ "$bench_job_count" -lt 1 ]; then
  echo "expected $root_dir/.github/workflows/bench.yml to declare at least one job" >&2
  exit 1
fi
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" '^        run: sudo apt-get update && sudo apt-get install --yes libasound2-dev$' "$bench_job_count"
assert_contains "$root_dir/shell/check.sh" 'mise run check_wasm'
assert_contains "$root_dir/shell/check.sh" 'mise run test_persistence'
assert_contains "$root_dir/shell/Trunk.toml" '^filehash[[:space:]]*=[[:space:]]*false$'
assert_contains "$root_dir/shell/assets/sw.js" "'./shell.js'"
assert_contains "$root_dir/shell/assets/sw.js" "'./shell_bg.wasm'"
assert_contains "$root_dir/shell/assets/sw.js" "^var cacheName = 'orcvs-pwa-v[0-9]+';$"
assert_contains "$root_dir/shell/assets/sw.js" "self[.]addEventListener[(]'activate'"
assert_contains "$root_dir/shell/assets/sw.js" 'caches[.]keys[(][)]'
assert_contains "$root_dir/shell/assets/sw.js" 'caches[.]delete[(]name[)]'
assert_contains "$root_dir/shell/assets/sw.js" "name === 'orcvs-pwa'"
assert_contains "$root_dir/shell/assets/sw.js" "name === 'egui-template-pwa'"
assert_contains "$root_dir/shell/assets/sw.js" "name[.]startsWith[(]'orcvs-pwa-'[)]"
assert_contains "$root_dir/shell/assets/sw.js" 'return isOrcvsCache && name !== cacheName;'
assert_contains "$root_dir/shell/assets/sw.js" 'caches[.]open[(]cacheName[)]'
assert_contains "$root_dir/shell/assets/sw.js" 'cache[.]match[(]e[.]request[)]'
assert_not_contains "$root_dir/shell/assets/sw.js" 'caches[.]match[(]e[.]request[)]'
assert_contains "$root_dir/shell/assets/sw.js" 'self[.]skipWaiting[(][)]'
assert_contains "$root_dir/shell/assets/sw.js" 'self[.]clients[.]claim[(][)]'
assert_contains "$root_dir/shell/assets/sw.js" "e[.]request[.]mode === 'navigate'"
assert_contains "$root_dir/shell/assets/sw.js" "e[.]request[.]url[.]endsWith[(]'/shell[.]js'[)]"
assert_contains "$root_dir/shell/assets/sw.js" "e[.]request[.]url[.]endsWith[(]'/shell_bg[.]wasm'[)]"
assert_contains "$root_dir/shell/assets/sw.js" "fetch[(]e[.]request, \{ cache: 'no-cache' \}[)]"
assert_contains "$root_dir/shell/assets/sw.js" 'response[.]ok'
assert_contains "$root_dir/shell/assets/sw.js" 'cache[.]put[(]e[.]request, response[.]clone[(][)][)][.]catch'
assert_contains "$root_dir/shell/assets/sw.js" 'response[[:space:]]*[|][|][[:space:]]*Response[.]error[(][)]'
assert_contains "$root_dir/.vscode/launch.json" '"--package=orcvs",'
assert_contains "$root_dir/.vscode/launch.json" '"--package=shell"'
assert_not_contains "$root_dir/.vscode/launch.json" '(package|bin)=console'
assert_not_contains "$root_dir/.vscode/launch.json" '(package|bin)=(vtha|parser_benchmark)'
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

# Criterion covers both benchmarked paths: language execution in `lang`, and
# populated Source rendering and editing in `orcvs`. It stays a plain versioned
# dev-dependency of exactly those two crates, so no shipped target and no other
# crate pulls its tree in.
assert_contains "$root_dir/lang/Cargo.toml" '^criterion[[:space:]]*=[[:space:]]*\{[^}]*cargo_bench_support'
assert_contains "$root_dir/orcvs/Cargo.toml" '^criterion[[:space:]]*=[[:space:]]*\{[^}]*cargo_bench_support'
assert_toml_table_not_contains "$root_dir/lang/Cargo.toml" '^[[:space:]]*[[]([^]]+[.])?dependencies[]][[:space:]]*$' '^[[:space:]]*criterion[[:space:]]*='
assert_toml_table_not_contains "$root_dir/orcvs/Cargo.toml" '^[[:space:]]*[[]([^]]+[.])?dependencies[]][[:space:]]*$' '^[[:space:]]*criterion[[:space:]]*='
assert_not_contains "$root_dir/Cargo.toml" '^[[:space:]]*criterion[[:space:]]*='
assert_not_contains "$root_dir/shell/Cargo.toml" '^[[:space:]]*criterion([.]workspace)?[[:space:]]*='
assert_not_contains "$root_dir/lang/Cargo.toml" '^[[:space:]]*criterion[.]workspace[[:space:]]*='
assert_not_contains "$root_dir/orcvs/Cargo.toml" '^[[:space:]]*criterion[.]workspace[[:space:]]*='
assert_not_contains "$root_dir/Cargo.toml" '^\[profile\.ci\]$'
assert_contains "$root_dir/Cargo.toml" '^tokio[[:space:]]*=[[:space:]]*\{[^}]*version[[:space:]]*='
for manifest in "$root_dir/orcvs/Cargo.toml" "$root_dir/shell/Cargo.toml"; do
  assert_not_contains "$manifest" '^tokio[[:space:]]*=[[:space:]]*\{[^}]*version[[:space:]]*='
  assert_not_contains "$manifest" '^[[:space:]]*(dependencies[.])?tokio[.]version[[:space:]]*='
  assert_not_contains "$manifest" '^[[:space:]]*dependencies[.]tokio[[:space:]]*=[[:space:]]*\{[^}]*version[[:space:]]*='
  assert_toml_table_not_contains "$manifest" '^[[:space:]]*[[]([^]]+[.])?dependencies[.]tokio[]][[:space:]]*$' '^[[:space:]]*version[[:space:]]*='
  assert_not_contains "$manifest" '^tokio[[:space:]]*=[[:space:]]*\{[^}]*features[[:space:]]*=[[:space:]]*\[[^]]*"full"'
done

# Proptest answers the obligation this contract already carries at the parser
# boundary: "boundary or property tests". Every invariant it encodes is
# platform-independent logic, so it stays a dev-dependency of the two crates that
# hold those invariants, confined to the non-WASM target table. `wasm-pack test`
# then compiles with no proptest in the graph, and no shipped binary can pull its
# tree in.
proptest_native_dev_table='^[[]target[.].cfg[(]not[(]target_arch = "wasm32"[)][)].[.]dev-dependencies[]]$'
assert_toml_table_contains "$root_dir/lang/Cargo.toml" "$proptest_native_dev_table" '^[[:space:]]*proptest[.]workspace[[:space:]]*='
assert_toml_table_contains "$root_dir/orcvs/Cargo.toml" "$proptest_native_dev_table" '^[[:space:]]*proptest[.]workspace[[:space:]]*='
assert_contains "$root_dir/Cargo.toml" '^proptest[[:space:]]*=[[:space:]]*\{[^}]*version[[:space:]]*='
assert_toml_table_not_contains "$root_dir/lang/Cargo.toml" '^[[:space:]]*[[]([^]]+[.])?dependencies[]][[:space:]]*$' '^[[:space:]]*proptest([.]workspace)?[[:space:]]*='
assert_toml_table_not_contains "$root_dir/orcvs/Cargo.toml" '^[[:space:]]*[[]([^]]+[.])?dependencies[]][[:space:]]*$' '^[[:space:]]*proptest([.]workspace)?[[:space:]]*='
assert_not_contains "$root_dir/shell/Cargo.toml" '^[[:space:]]*proptest([.]workspace)?[[:space:]]*='
# The plain `[dev-dependencies]` table is the one that also compiles for WASM, so
# it needs its own guard: the shipped-dependency assertions above deliberately do
# not match a `dev-` table, and without this a move from the target table into the
# plain one would leave every other assertion green while putting proptest back
# into the `wasm-pack test` graph.
assert_toml_table_not_contains "$root_dir/lang/Cargo.toml" '^[[:space:]]*[[]dev-dependencies[]][[:space:]]*$' '^[[:space:]]*proptest([.]workspace)?[[:space:]]*='
assert_toml_table_not_contains "$root_dir/orcvs/Cargo.toml" '^[[:space:]]*[[]dev-dependencies[]][[:space:]]*$' '^[[:space:]]*proptest([.]workspace)?[[:space:]]*='
# The pull-request tier trades case count for latency; the merge tier keeps
# proptest's 256-case default. Task-level env, so every run line above stays
# byte-identical to the text this script pins. Asserting the setting appears
# exactly once pins the merge tier's default without naming the merge tasks: a
# renamed task, a new tier, or a global `[env]` table would each break it, where
# a per-task negative assertion would silently pass.
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^env = [{] PROPTEST_CASES = .32. [}]$'
proptest_cases_settings="$(grep -Ev '^[[:space:]]*#' "$root_dir/mise.toml" | grep -Ec 'PROPTEST_CASES' || true)"
if [ "$proptest_cases_settings" -ne 1 ]; then
  echo "expected mise.toml to set PROPTEST_CASES exactly once, found $proptest_cases_settings" >&2
  exit 1
fi
# A counterexample CI can see and a developer cannot reproduce is worse than no
# property, so the `proptest-regressions` files are source and are never ignored.
# Asking git rather than reading `.gitignore` catches a broad glob or a nested
# ignore file that a substring match would miss.
# check-ignore answers 0 for ignored and 1 for not ignored, but 128 for its own
# failures. Collapsing 128 into "not ignored" would make this check pass silently
# wherever git cannot answer, so only 1 is accepted as the clean result.
for regressions_path in lang/proptest-regressions/parser.txt orcvs/proptest-regressions/grid.txt; do
  ignore_status=0
  git -C "$root_dir" check-ignore -q "$regressions_path" || ignore_status=$?
  case "$ignore_status" in
    0)
      echo "expected $regressions_path not to be ignored by git" >&2
      exit 1
      ;;
    1) ;;
    *)
      echo "git check-ignore failed with status $ignore_status for $regressions_path" >&2
      exit 1
      ;;
  esac
done
