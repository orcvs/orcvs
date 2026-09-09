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

# Counts the jobs a workflow declares: the keys at exactly one indent level
# inside `jobs:`, which is where a job name lives and where nothing else does.
# Assertions that must hold once per job derive their expected count from this
# rather than from a literal, so a job added without them fails here.
workflow_job_count() {
  awk '
    /^[[:space:]]*#/ { next }
    /^jobs:[[:space:]]*$/ { in_jobs = 1; next }
    in_jobs && /^[^[:space:]]/ { in_jobs = 0 }
    in_jobs && /^  [A-Za-z0-9_-]+:[[:space:]]*$/ { count++ }
    END { print count + 0 }
  ' "$1"
}

# The count of a pattern outside comment lines, for assertions whose expected
# number is another property of the same file rather than a literal.
count_matches() {
  grep -Ev '^[[:space:]]*#' "$1" | grep -Ec "$2" || true
}

# The triggers a workflow declares, one per line: the keys at exactly one indent
# level inside `on:`. Read as a set rather than matched as forbidden names, so a
# rule about what may run a workflow cannot be stepped around by reaching for a
# trigger the rule's authors did not think to forbid.
workflow_triggers() {
  awk '
    /^[[:space:]]*#/ { next }
    /^on:[[:space:]]*$/ { in_on = 1; next }
    in_on && /^[^[:space:]]/ { in_on = 0 }
    in_on && /^  [A-Za-z_]+[[:space:]]*:/ {
      line = $0
      sub(/^  /, "", line)
      sub(/[[:space:]]*:.*/, "", line)
      print line
    }
  ' "$1"
}

assert_only_trigger() {
  local file="$1"
  local expected="$2"
  local actual
  actual="$(workflow_triggers "$file" | paste -sd, -)"
  if [ "$actual" != "$expected" ]; then
    echo "expected $file to declare $expected as its only trigger, found: ${actual:-none}" >&2
    exit 1
  fi
}

assert_contains "$root_dir/mise.toml" '^\[tools\]$'
assert_contains "$root_dir/mise.toml" '^"cargo:cargo-nextest"[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"$'
assert_contains "$root_dir/mise.toml" '^"cargo:cargo-deny"[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"$'
assert_contains "$root_dir/mise.toml" '^"cargo:trunk"[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"$'
assert_contains "$root_dir/mise.toml" '^"cargo:wasm-pack"[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"$'
# The roadmap suite runs through node's own test runner, so the tier that runs it
# needs the runtime pinned here rather than inherited from whatever the machine has.
assert_contains "$root_dir/mise.toml" '^node[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"$'
# The workflows are the only part of the verification surface no compiler reads,
# and they are the part that decides whether the rest runs at all. `actionlint`
# checks their syntax and expressions, `zizmor` audits them for injection and
# permission findings; both are pinned exactly like the cargo tools, because a
# linter that follows its own latest release turns an unrelated pull request red.
assert_contains "$root_dir/mise.toml" '^"aqua:rhysd/actionlint"[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"$'
assert_contains "$root_dir/mise.toml" '^"aqua:zizmorcore/zizmor"[[:space:]]*=[[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+"$'
assert_toml_task_contains "$root_dir/mise.toml" 'check' '^mise run check_pull_request$'
assert_toml_task_contains "$root_dir/mise.toml" 'check' '^mise run check_merge$'
# The contract and its own tests run in the pull-request tier: nothing else
# executes them, so a gate that only a local run reaches is a gate that drifts.
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^bash scripts/check-tooling-contract.sh$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^bash scripts/tests/check-tooling-contract.sh$'
# Both linters run beside the contract script, on the same reasoning: they check
# the repository's own configuration, they cost seconds, and they fail before the
# tier spends twenty minutes compiling. Expect little from them — the workflows
# already SHA-pin every action and declare permissions per job — which is the
# point. They hold that shape rather than discovering it.
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^actionlint$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^zizmor --offline [.]github/workflows$'
# The roadmap planner throws on tracker inconsistency, so its suite guards
# invariants agents edit constantly. It had never run automatically, and had
# already drifted by two tests before anything executed it.
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^node --test scripts/tests/roadmap.test.ts$'
# The suite runs the planner over temporary fixtures and never reads `.scratch/`,
# so the throws that catch tracker drift — a dangling `Blocked by:`, a dependency
# cycle, an untagged release blocker — need the planner run against the real tree.
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^node scripts/roadmap[.]ts > /dev/null$'
# `audit_deps` had no caller at all. Dependabot's weekly grouped bumps are exactly
# the pull requests an advisory, licence, and source audit exists for, and the
# feature-resolved tree it prints was inspected only when a human typed it.
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^mise run audit_deps$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo fmt --all -- --check$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo clippy --workspace --all-targets --locked -- -D warnings$'
# The three persistence tests live in a test-only module and depend on serde_json,
# a dev-dependency absent from the normal graph, so no library build can reach
# them. Only an all-targets build with the feature enabled compiles them, and that
# ran behind the push guard: they were neither run nor type-checked before a merge.
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo clippy --workspace --all-targets --features persistence --locked -- -D warnings$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo nextest run --workspace --profile ci --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo nextest run --workspace --tests --features persistence --profile ci --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo test --workspace --doc --locked$'
# A doctest on a `persistence`-gated item is compiled by no default-feature run,
# so leaving this in the merge tier alone kept one persistence path in the
# found-after-merge class the tier beside it had just left.
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo test --workspace --doc --features persistence --locked$'
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
# `--lib` type-checks no test target, so the browser regressions compiled only
# under `wasm-pack test` in the merge tier. Compiling the test targets here is
# what keeps a break in them off main. The scope is the workspace rather than one
# package: it was shell alone only while orcvs built an unguarded Tokio runtime in
# a test, which no longer holds.
assert_toml_task_contains "$root_dir/mise.toml" 'check_wasm' '^cargo clippy --workspace --all-targets --target wasm32-unknown-unknown --features persistence --locked -- -D warnings$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_wasm' '^cd shell$'
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
bench_job_count="$(workflow_job_count "$root_dir/.github/workflows/bench.yml")"
if [ "$bench_job_count" -lt 1 ]; then
  echo "expected $root_dir/.github/workflows/bench.yml to declare at least one job" >&2
  exit 1
fi
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" '^        run: sudo apt-get update && sudo apt-get install --yes libasound2-dev$' "$bench_job_count"
# The memory series is published from the same jobs and from the same test
# functions that assert on the numbers. A separate binary re-running the measured
# paths would let the published number and the asserted number drift apart, so
# the command is pinned rather than merely the fact that something is measured.
# `cargo test` and not `cargo nextest run`: both bench jobs run `mise-action` with
# `install: false`, so nextest is not installed and asking for it would either
# fail the job or put ten minutes of tool building back into it.
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" '^        run: ORCVS_MEMORY_SERIES=1 cargo test --package lang --package orcvs --test allocation --locked -- --nocapture [|] tee allocations[.]txt$' "$bench_job_count"
assert_not_contains "$root_dir/.github/workflows/bench.yml" 'cargo nextest run'
# Exactly once per job is also what pins "no warm-up run for the memory series".
# The timing series runs `mise run bench` twice per job because a freshly compiled
# criterion binary's first pass is contaminated; an allocation count is
# deterministic for a fixed input, so a second run would only cost the job twice.
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" '^        run: mise run bench > /dev/null$' "$bench_job_count"
# The series name the action keys the stored history by. A rename does not move
# the history, it starts an empty series beside it, which is why the existing
# `lang` name carries the same rule and is pinned the same way.
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" '^          name: memory$' "$bench_job_count"
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" '^          name: lang$' "$bench_job_count"
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" '^          tool: customSmallerIsBetter$' "$bench_job_count"
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" '^          tool: cargo$' "$bench_job_count"
# The memory step in each job runs after a `github-action-benchmark` step that has
# already fetched `gh-pages` — and, in the publishing job, pushed to it. Fetching
# again would discard the commit that step just made.
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" '^          skip-fetch-gh-pages: true$' "$bench_job_count"
# The memory series alerts and writes a job summary and does not fail the
# workflow. That is a decision rather than an omission: a deterministic metric at
# a threshold this tight fires on any real change, and the action offers no
# in-repo way to accept a deliberate increase, since the series lives on
# `gh-pages` rather than in a file a pull request can edit beside the change that
# moves it. Whether this workflow blocks a merge at all belongs to
# `.scratch/verification-gaps/issues/09`, which this series stays out of.
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" '^          fail-on-alert: false$' "$bench_job_count"
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" "^          alert-threshold: '110%'\$" "$bench_job_count"
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" "^          fail-threshold: '125%'\$" "$bench_job_count"
# The timing series' own thresholds, stated beside them, because "far tighter than
# the timing series" is only a property of the pair.
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" "^          alert-threshold: '150%'\$" "$bench_job_count"
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" "^          fail-threshold: '300%'\$" "$bench_job_count"
# The JSON is assembled with coreutils and shell builtins, so this step installs
# nothing and `mise.toml` gains no tool for it. `jq` is the obvious reach and it is
# the one thing this must not become.
assert_contains "$root_dir/.github/workflows/bench.yml" '^          printf .\[%s\].n. "[$][(]printf'
assert_not_contains "$root_dir/.github/workflows/bench.yml" '(^|[^[:alnum:]-])jq([^[:alnum:]-]|$)'
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
# Every version this repository pins is bumped by something that watches the file
# it lives in. `rust-toolchain.toml` was watched by nothing, which is why the
# channel sat at 1.98.0 while 1.98.1 was current. Dependabot reads it under its own
# ecosystem; the `cargo` entry does not, so the two are asserted separately.
assert_contains "$root_dir/.github/dependabot.yml" '^  - package-ecosystem: cargo$'
assert_contains "$root_dir/.github/dependabot.yml" '^  - package-ecosystem: github-actions$'
assert_contains "$root_dir/.github/dependabot.yml" '^  - package-ecosystem: rust-toolchain$'
assert_contains "$root_dir/.github/workflows/test.yml" 'run: mise run check_pull_request$'
# Counting each component pins which merge tier runs, not merely that some step
# carries a guard: dropping the native step while adding a guard elsewhere leaves
# the guard count at two, and rustdoc, `cargo deny` and the 256-case persistence
# run stop reaching `main` with the contract still green.
assert_occurs_exactly "$root_dir/.github/workflows/test.yml" 'ORCVS_MERGE_COMPONENT: native$' 1
assert_occurs_exactly "$root_dir/.github/workflows/test.yml" 'ORCVS_MERGE_COMPONENT: wasm$' 1
assert_occurs_exactly "$root_dir/.github/workflows/test.yml" 'run: mise run check_merge$' 2
assert_contains "$root_dir/.github/workflows/test.yml" 'run: mise run check_wasm$'
# A push to `main` must not cancel an earlier commit's run. The group interpolated
# the pull request number, which is empty on a push, so every push shared one
# group; `dd20cba6` landed with two of three jobs cancelled and was never re-run.
# A cancelled run reports `cancelled` rather than `failure`, so nothing alerted.
assert_contains "$root_dir/.github/workflows/test.yml" '^  group: [$][{][{] github[.]workflow [}][}]-[$][{][{] github[.]event[.]pull_request[.]number [|][|] github[.]sha [}][}]$'
assert_contains "$root_dir/.github/workflows/test.yml" "^  cancel-in-progress: [\$][{][{] github[.]event_name == 'pull_request' [}][}]$"
# Two steps are merge-only — the native and WASM `check_merge` runs — and
# nothing else is. A third occurrence is a job that stopped running on pull
# requests, which is how the WASM tier came to be skipped before a merge.
# The guard tests "not a pull request" rather than "is a push" so that a manual
# dispatch — the obvious way to re-verify a commit whose run was cancelled — runs
# the merge tier instead of reporting three green jobs that ran only the
# pull-request tier.
assert_occurs_exactly "$root_dir/.github/workflows/test.yml" "if: github.event_name != 'pull_request'$" 2
assert_not_contains "$root_dir/.github/workflows/test.yml" "if: github.event_name == 'push'$"
# The count above reads the bare literal. GitHub accepts the same guard written
# as an expression, and a block or folded scalar puts that expression on the line
# after the key, where nothing anchored to `if:` can see it. Requiring the file's
# total number of `if:` keys to equal the number spelled as the literal leaves no
# spelling that GitHub accepts and this check misses: a substituted guard either
# drops the literal count below two or lifts the total above it. YAML reads `if :`
# as that same key, so the total counts the colon detached as well as fused: it is
# the total, not the literal, that a third guard would otherwise slip past, and a
# backstop blind to a spelling GitHub honours is not a backstop.
assert_occurs_exactly "$root_dir/.github/workflows/test.yml" '^[[:space:]]*(- )?if[[:space:]]*:' 2
# `@v1` is how a mutable tag is usually written, so matching a bare digit after
# the `@` let the common spelling of the thing this forbids straight through.
assert_not_contains "$root_dir/.github/workflows/test.yml" '(cargo-nextest|cargo-deny|nextest|trunk|wasm-pack)@v?[0-9]'
assert_not_contains "$root_dir/.github/workflows/test.yml" 'taiki-e/install-action'
# Dependabot rewrites the version comment beside the SHA on every bump, and this
# script is the first line of `check_pull_request`, so pinning the patch would
# turn each of its own bump pull requests red on all three jobs. The major is
# what deserves a human: it is where an action's inputs and node runtime move.
assert_contains "$root_dir/.github/workflows/test.yml" 'uses: actions/checkout@[0-9a-f]{40}[[:space:]]+# v7([.][0-9]+)*$'
assert_contains "$root_dir/.github/workflows/test.yml" 'uses: dtolnay/rust-toolchain@[0-9a-f]{40}[[:space:]]+# 1[.]98[.]0$'
assert_contains "$root_dir/.github/workflows/test.yml" 'uses: Swatinem/rust-cache@[0-9a-f]{40}[[:space:]]+# v2$'
assert_contains "$root_dir/.github/workflows/test.yml" 'uses: jdx/mise-action@[0-9a-f]{40}[[:space:]]+# v4([.][0-9]+)*$'

# `cargo deny` sees the graph whenever a commit changes it, and an advisory is
# published against code nobody changed. Without a trigger that is time rather
# than change, a RUSTSEC entry landing against a locked dependency waits for the
# next pull request to be reported. The schedule is the whole gate, so it is
# pinned here alongside the command it runs — a workflow left with only its
# manual dispatch would be the same silence with a file to point at.
assert_contains "$root_dir/.github/workflows/advisories.yml" '^  schedule:$'
assert_contains "$root_dir/.github/workflows/advisories.yml" "^    - cron: '[-0-9*/,]+ [-0-9*/,]+ [-0-9*/,]+ [-0-9*/,]+ [-0-9*/,]+'\$"
assert_contains "$root_dir/.github/workflows/advisories.yml" '^      - run: mise run audit_deps$'
# The pinning rules are asserted per file, so the newest workflow needs them
# stated against it rather than inherited from the two written before it.
assert_contains "$root_dir/.github/workflows/advisories.yml" 'uses: actions/checkout@[0-9a-f]{40}[[:space:]]+# v7([.][0-9]+)*$'
assert_contains "$root_dir/.github/workflows/advisories.yml" 'uses: dtolnay/rust-toolchain@[0-9a-f]{40}[[:space:]]+# 1[.]98[.]0$'
assert_contains "$root_dir/.github/workflows/advisories.yml" 'uses: jdx/mise-action@[0-9a-f]{40}[[:space:]]+# v4([.][0-9]+)*$'
assert_not_contains "$root_dir/.github/workflows/advisories.yml" 'taiki-e/install-action'
assert_not_contains "$root_dir/.github/workflows/advisories.yml" '(cargo-nextest|cargo-deny|nextest|trunk|wasm-pack)@v?[0-9]'
# Stated against `bench.yml` for the same reason: unasserted, its `checkout` and
# `mise-action` pins sat a major behind the other two workflows, and the split
# mise-action major stored every mise cache under two key shapes at once.
assert_contains "$root_dir/.github/workflows/bench.yml" 'uses: actions/checkout@[0-9a-f]{40}[[:space:]]+# v7([.][0-9]+)*$'
assert_contains "$root_dir/.github/workflows/bench.yml" 'uses: dtolnay/rust-toolchain@[0-9a-f]{40}[[:space:]]+# 1[.]98[.]0$'
assert_contains "$root_dir/.github/workflows/bench.yml" 'uses: Swatinem/rust-cache@[0-9a-f]{40}[[:space:]]+# v2$'
assert_contains "$root_dir/.github/workflows/bench.yml" 'uses: jdx/mise-action@[0-9a-f]{40}[[:space:]]+# v4([.][0-9]+)*$'
assert_not_contains "$root_dir/.github/workflows/bench.yml" 'taiki-e/install-action'
# The memory series reuses the timing series' action rather than adding one, so
# every use of it carries the same pin. Two steps per job — the timing series and
# the memory series — derived from the job count rather than from a literal, so a
# job added with only one of them fails here.
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" 'uses: benchmark-action/github-action-benchmark@[0-9a-f]{40}[[:space:]]+# v1([.][0-9]+)*$' "$((bench_job_count * 2))"
# A short SHA or a missing pin is already caught by the count above, which only
# a full forty-character digest satisfies; this states the mutable refs by name.
assert_not_contains "$root_dir/.github/workflows/bench.yml" 'benchmark-action/github-action-benchmark@(v[0-9]|main|master)'

# Every job in every workflow carries a bound on its runtime. Without one a job
# inherits the six-hour runner limit, and the shape that would spend it is a
# `wasm-pack test --headless --firefox` waiting on a browser that never answers —
# jobs here otherwise finish in about ninety seconds. The expected count is the
# number of jobs the file declares rather than a literal, so a job added without a
# bound fails this check instead of quietly inheriting the default. Four spaces is
# the job-level indent; a step-level `timeout-minutes` sits deeper and is not
# counted, so a bound on one step cannot stand in for the job's.
# A cache saved on a pull request's ref can only ever be read back by that same
# pull request. Saving there fills the repository's shared quota with entries no
# run reads and evicts the ones on `main` that every run restores from, so each
# caching action's write is gated on the default branch. Restoring is deliberately
# not gated: a pull request still reads `main`'s cache through the key prefix, so
# the gate costs a pull request nothing. The expected count is the number of
# caching steps the file declares rather than a literal, so a job added with an
# ungated cache fails this check instead of quietly filling the quota.
for workflow in "$root_dir"/.github/workflows/*.yml; do
  assert_occurs_exactly "$workflow" \
    "^          save-if: [\$][{][{] github[.]ref == 'refs/heads/main' [}][}]\$" \
    "$(count_matches "$workflow" 'uses: Swatinem/rust-cache@')"
  assert_occurs_exactly "$workflow" \
    "^          cache_save: [\$][{][{] github[.]ref == 'refs/heads/main' [}][}]\$" \
    "$(count_matches "$workflow" 'uses: jdx/mise-action@')"
done

# Both benchmark jobs build the same tree under the same profile and only one of
# them ever runs, so they share one cache entry. Without the shared key each is
# keyed on its own job id and the same bytes are stored twice.
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" '^          shared-key: bench$' 2

for workflow in "$root_dir"/.github/workflows/*.yml; do
  workflow_jobs="$(workflow_job_count "$workflow")"
  if [ "$workflow_jobs" -lt 1 ]; then
    echo "expected $workflow to declare at least one job" >&2
    exit 1
  fi
  assert_occurs_exactly "$workflow" '^    timeout-minutes: [0-9]+$' "$workflow_jobs"
done

# Miri is deliberate and non-gating, and both halves of that are pinned here.
# `.scratch/verification-gaps/issues/12` decided the contract stops *requiring*
# Miri — it ships on nightly only, `rust-toolchain.toml` pins stable, and naming
# a gate the toolchain cannot run makes the contract unfollowable at the one
# place it matters most. What that decision kept is Miri as the tool the unsafe
# gate would prefer, run on purpose, so the task is that purpose written down and
# its two lines are pinned rather than left to drift. The first is the channel
# and component the task installs for itself: declaring them in
# `rust-toolchain.toml` instead would make every other gate nightly's problem.
assert_toml_task_contains "$root_dir/mise.toml" 'miri' '^rustup toolchain install nightly --component miri$'
# The second is the run itself, scoped by test filter rather than by crate.
# `orcvs` links ALSA through `midir` and builds a multi-threaded Tokio runtime,
# and Miri can execute neither; it interprets what actually runs rather than what
# the crate links, so the filter is the whole reason those never become a
# problem. Widening it to the package would put them back, which is why the
# selection is pinned and not merely the `cargo miri` prefix. The task-scoped
# check reads through awk, which rejects an escaped `^` inside a pattern, so it
# holds the shape and the grep-backed line beneath it holds the exact text.
assert_toml_task_contains "$root_dir/mise.toml" 'miri' '^cargo [+]nightly miri nextest run --package orcvs -E .test[(]/.source::model::test::/[)].$'
assert_contains "$root_dir/mise.toml" "^cargo [+]nightly miri nextest run --package orcvs -E 'test[(]/\^source::model::test::/[)]'\$"
# No tier calls it. A `mise run miri` line inside another task is the shape that
# turns the deliberate path back into a requirement without anyone deciding to,
# and it would arrive on every pull request as an interpreter roughly two orders
# of magnitude slower than the suite beside it. Matched unanchored and with
# mise's `r` abbreviation, because an anchored `^mise run miri$` reads only a
# line that is nothing else: `mise run check && mise run miri` and `mise r miri`
# are the same call and would both have walked past it.
assert_not_contains "$root_dir/mise.toml" 'mise (run|r) miri([^[:alnum:]_-]|$)'
# The same rule against the workflow that runs it, stated over whichever workflow
# runs the task rather than over a file name — so a renamed or copied job cannot
# step around it, and so the contract's own fixture has no missing file to
# dereference.
#
# Stated as a whole trigger set rather than as a list of forbidden names.
# `workflow_dispatch` has to be there, because a job with no trigger is not a
# path anyone can take. Naming `pull_request` and `push` as the two that must
# not be left every other automatic trigger through: a `schedule:` with a nightly
# cron reverses `verification-gaps/12` exactly as a `push:` would — a 90-minute
# interpreter running unasked — and forbidding the two spellings someone thought
# of is not a rule about what may run this workflow. `release`, `workflow_run`
# and `workflow_call` are the same hole. Requiring the set to be exactly
# `workflow_dispatch` leaves none of them.
for workflow in "$root_dir"/.github/workflows/*.yml; do
  if grep -Ev '^[[:space:]]*#' "$workflow" | grep -Eq '^[[:space:]]*-?[[:space:]]*run: mise (run|r) miri([^[:alnum:]_-]|$)'; then
    assert_only_trigger "$workflow" 'workflow_dispatch'
  fi
done

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
for regressions_path in \
  lang/proptest-regressions/parser.txt \
  lang/proptest-regressions/interpreter.txt \
  orcvs/proptest-regressions/grid.txt \
  orcvs/proptest-regressions/source/language_map.txt \
  orcvs/proptest-regressions/source/tick.txt; do
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
