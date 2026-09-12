#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "$0")/.." && pwd)"

# Every assertion below pipes the comment-stripped file into a second grep, and
# that second grep must read to end of input. `grep -q` does not: it exits on its
# first match, the upstream grep dies of SIGPIPE, and `set -o pipefail` above
# then reports 141 for a pipeline whose pattern *did* match. It is a race on how
# much the upstream has managed to write, so it stays invisible on small files
# and on macOS, and fires on Linux once a file is long enough — at 350 lines of
# `bench.yml` it failed 160 of 200 identical assertions. Redirecting to
# /dev/null instead of asking for `-q` costs nothing here and cannot race.
assert_contains() {
  local file="$1"
  local pattern="$2"
  if ! grep -Ev '^[[:space:]]*#' "$file" | grep -E "$pattern" >/dev/null; then
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
  if grep -Ev '^[[:space:]]*#' "$file" | grep -E "$pattern" >/dev/null; then
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
# The contract runs in the pull-request tier: nothing else executes it, so a gate
# that only a local run reaches is a gate that drifts. It costs under a second
# and it is what fails when someone edits a pinned line, so it is owed by every
# pull request whatever that pull request touched.
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^bash scripts/check-tooling-contract.sh$'
# Its fixture suite is not, and must not be. The suite copies a tree, breaks one
# line and re-runs the contract, ninety times over: two minutes, and the tier
# runs on both the Linux and macOS legs, so every pull request paid it twice —
# including the ones that touched no tooling. It runs path-filtered instead, in
# the workflow asserted below. Pinned as an absence as well as a presence,
# because putting the line back is a one-word edit that no other check notices.
assert_not_contains "$root_dir/mise.toml" '^bash scripts/tests/check-tooling-contract.sh$'
assert_contains "$root_dir/.github/workflows/tooling.yml" '^      - run: bash scripts/tests/check-tooling-contract.sh$'
# Path-filtering the suite is only sound while the filter names every file the
# suite reads. The two lists are the same set stated twice — the `$repo_root`
# paths the suite copies into a fixture, and the `paths:` entries that decide
# whether it runs — so a file added to the suite and not to the workflow leaves
# the gate blind to exactly the file it had just started reading. That is the
# failure this contract exists to catch, one layer up, so it is derived here
# rather than trusted: both sides are read out of the files and compared.
#
# Subset, not equality: the workflow also lists itself and the suite script,
# neither of which the suite reads from `$repo_root`. What must not happen is a
# read that no path covers.
fixture_inputs="$(grep -oE '[$]repo_root/[a-zA-Z0-9./_-]+' "$root_dir/scripts/tests/check-tooling-contract.sh" | sed 's|[$]repo_root/||' | grep '[.]' | sort -u)"
workflow_paths="$(grep -oE "^      - '[^']+'$" "$root_dir/.github/workflows/tooling.yml" | sed "s|^      - '||; s|'$||" | sort -u)"
uncovered="$(comm -23 <(printf '%s
' "$fixture_inputs") <(printf '%s
' "$workflow_paths"))"
if [ -n "$uncovered" ]; then
  echo "expected .github/workflows/tooling.yml to path-filter every file the fixture suite reads; uncovered:" >&2
  printf '%s
' "$uncovered" >&2
  exit 1
fi
# And the suite's own two scripts, which are inputs by being the code that runs.
assert_contains "$root_dir/.github/workflows/tooling.yml" "^      - 'scripts/tests/check-tooling-contract.sh'$"
assert_contains "$root_dir/.github/workflows/tooling.yml" "^      - '.github/workflows/tooling.yml'$"
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
# them. Only an all-targets build compiles them, and that ran behind the push
# guard: they were neither run nor type-checked before a merge. `console` now
# enables `persistence` by default and pulls `orcvs/persistence` with it, so the
# plain lines below are that build; the `--no-default-features` line beside each
# is what still compiles and runs the feature-off configuration, which is
# `product-persistence/01`'s own acceptance criterion. Pinning both halves is
# what stops the pair collapsing back into one configuration named twice.
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo clippy --workspace --all-targets --no-default-features --locked -- -D warnings$'
# `native-midi` is on by default, so every workspace compilation in this tier
# builds `orcvs` with a native MIDI backend — the `--no-default-features` one
# included, because `console` names the feature for its native targets. These
# two are the ones that build it without: the clippy pass crosses the feature off
# against `persistence`, so the four feature cells are all compiled, and the
# nextest pass runs the tests that state what turning the feature off gives up,
# which are compiled only with it off. Losing either leaves a shipped feature
# state that no tier reaches.
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo clippy --package orcvs --all-targets --no-default-features --features persistence --locked -- -D warnings$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo nextest run --package orcvs --no-default-features --profile ci --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo nextest run --workspace --profile ci --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo nextest run --workspace --tests --no-default-features --profile ci --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo test --workspace --doc --locked$'
# A doctest on a `persistence`-gated item is compiled by no feature-off run and a
# doctest on an item the feature removes is compiled by no default run, so the
# tier compiles the doctests under both.
assert_toml_task_contains "$root_dir/mise.toml" 'check_pull_request' '^cargo test --workspace --doc --no-default-features --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_merge' '^[[:space:]]*mise run check_merge_native$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_merge' '^[[:space:]]*mise run check_wasm$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_merge' '^[[:space:]]*mise run test_wasm$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_merge_native' '^mise run test_persistence$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_merge_native' '^cargo deny --locked check$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_merge_native' '^RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'audit_deps' '^cargo deny --locked check$'
assert_toml_task_contains "$root_dir/mise.toml" 'audit_deps' '^cargo tree --workspace --all-features -e features --locked$'
# The reason `native-midi` exists is a claim about the dependency tree, and a
# printed tree is read by a human or by nobody. This pins the check that fails
# instead: the tree `orcvs` resolves with the feature off, and the grep over it
# that rejects `midir` and the ALSA and CoreMIDI crates beneath it. Pinned as
# three lines because each carries part of the answer — the resolution, the
# rejection, and the non-zero exit. The rejecting pattern is held by the
# grep-backed assertion beneath, for the reason the Miri filter is: the
# task-scoped check reads through awk, which rejects the escaped `^` the pattern
# needs.
#
# The middle assertion names the variable because the resolution and the grep
# are two lines and nothing else joins them. Anchored at `^if printf` alone it
# passed against a half-finished rename: the pinned assignment still there,
# unused, and the grep reading an unset name — an empty string, which matches
# nothing, so `audit_deps` could no longer fail on a `midir` regression. It
# stops before the `-E` pattern, which is the part awk cannot carry.
assert_toml_task_contains "$root_dir/mise.toml" 'audit_deps' '^native_midi_tree="[$][(]cargo tree --package orcvs --no-default-features --edges normal --prefix none --locked[)]"$'
assert_toml_task_contains "$root_dir/mise.toml" 'audit_deps' "^if printf '%s.n' \"[\$]native_midi_tree\" [|] grep -E"
assert_toml_task_contains "$root_dir/mise.toml" 'audit_deps' '^  exit 1$'
assert_contains "$root_dir/mise.toml" "grep -E '\^\(midir\|alsa\|alsa-sys\|coremidi\|coremidi-sys\) '"
assert_toml_task_contains "$root_dir/mise.toml" 'test_persistence' '^cargo check --package orcvs --lib --features persistence --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'test_persistence' '^cargo clippy --workspace --all-targets --features persistence --locked -- -D warnings$'
# The `-E` is the point of the assertion, not decoration on it. Without the
# filter this line runs the criterion benchmarks as tests, unbudgeted, which is
# what held four merge-queue runs until `timeout-minutes` killed them. Anchored
# without it the pin passed against exactly that line. The parentheses are
# bracketed because the pattern reaches awk as an ERE.
assert_toml_task_contains "$root_dir/mise.toml" 'test_persistence' "^cargo nextest run --workspace --all-targets --features persistence --profile ci --locked -E 'not kind[(]bench[)]'\$"
assert_toml_task_contains "$root_dir/mise.toml" 'test_persistence' '^cargo test --workspace --doc --features persistence --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'test_persistence' '^RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --features persistence --locked$'
# `--lib` type-checks no test target, so the browser regressions compiled only
# under `wasm-pack test` in the merge tier. Compiling the test targets here is
# what keeps a break in them off main. The scope is the workspace rather than one
# package: it was console alone only while orcvs built an unguarded Tokio
# runtime in a test, which no longer holds.
assert_toml_task_contains "$root_dir/mise.toml" 'check_wasm' '^cargo clippy --workspace --all-targets --target wasm32-unknown-unknown --locked -- -D warnings$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_wasm' '^cd console$'
# Two builds, and they have to be two configurations. The default one is the
# persisting application the browser actually loads; the `--no-default-features`
# one is the only place the WASM build without the storage path is compiled.
assert_toml_task_contains "$root_dir/mise.toml" 'check_wasm' '^env -u NO_COLOR trunk build --no-default-features --locked$'
assert_toml_task_contains "$root_dir/mise.toml" 'check_wasm' '^env -u NO_COLOR trunk build --locked$'
# The browser suite runs the shipped configuration, which carries `persistence`
# through the console's default features. Naming the feature here would pin a
# flag that changes nothing; pinning the line without one is what catches a
# `--no-default-features` browser run that no longer exercises storage.
assert_toml_task_contains "$root_dir/mise.toml" 'test_wasm' '^run = .wasm-pack test --headless --firefox console --test wasm --locked.$'
# Both bench tasks are pinned whole, flags included. The criterion budget is not a
# tuning detail that may drift: the gate reading these numbers alerts at 150% and
# fails at 300% across two different hosted runners, so it cannot resolve better
# than tens of per cent, and criterion's defaults spend a 3s warm-up, a 5s
# measurement and a 100,000-resample bootstrap per benchmark chasing about 2%. On
# run 34321680905 that was ten seconds for each of 29 benchmarks, twice per job:
# 10m35s of a 12m32s job, buying precision three orders of magnitude finer than
# anything downstream reads. `--nresamples` carries its own share — roughly 2.7s of
# each benchmark's ten seconds fell outside the configured time budget, and the
# default bootstrap was most of it.
assert_toml_task_contains "$root_dir/mise.toml" 'bench' '^run = .cargo bench --package lang --package orcvs --benches --locked -- --output-format bencher --warm-up-time 0[.]5 --measurement-time 1 --sample-size 10 --nresamples 1000.$'
# The discard run differs from the measured one in exactly one figure, and that is
# the point of it having a task of its own: its output goes to /dev/null, so it
# needs no measurement fidelity, but it must still execute every benchmark. The
# contamination it exists for belongs to the binary rather than to a benchmark —
# criterion's own per-benchmark warm-up did not settle `parse_source`, which read
# 1,204 ns on the first run after a fresh compile against a settled 417 ns after.
assert_toml_task_contains "$root_dir/mise.toml" 'bench_warmup' '^run = .cargo bench --package lang --package orcvs --benches --locked -- --output-format bencher --warm-up-time 0[.]5 --measurement-time 0[.]1 --sample-size 10 --nresamples 1000.$'
# `--quick` looks like the flag this budget wants and it would disarm the gate in
# silence. It drops the name from each output line, and the action's `cargo` parser
# is one regex over `test <name> ... bench: <N> ns/iter`; a line that does not match
# is skipped without an error, so a `--quick` run stores zero benchmarks and reports
# green. It is not even faster than the flags above: 17.3s against 17.0s over the
# same nine benchmarks. The leading dash is bracketed in both patterns so `grep -E`
# reads them as patterns rather than as options of its own.
#
# The two shapes differ because the two files do. `mise.toml` holds every task in
# the repository, so the ban is scoped to lines that invoke `cargo bench` — the
# shape it guards is a third bench task written later, running `cargo bench` with
# the flag and then wired into the workflow. A file-wide ban there would fire on a
# trailing comment mentioning the flag, since the filter above strips only
# whole-line comments, and on any future unrelated task passing `--quick` to a tool
# that is not criterion. Scoping it to the two pinned tasks instead would guard
# nothing the whole-line pins do not already cover.
assert_not_contains "$root_dir/mise.toml" 'cargo bench.*[-]-quick'
# `bench.yml` is the benchmark workflow and nothing unrelated lives in it, so the
# ban stays file-wide there: the shape it guards is the flag appended to a step's
# `run` line, in an existing step or a new one.
assert_not_contains "$root_dir/.github/workflows/bench.yml" '[-]-quick'
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
# The timing series runs the benchmarks twice per job because a freshly compiled
# criterion binary's first pass is contaminated; an allocation count is
# deterministic for a fixed input, so a second run would only cost the job twice.
#
# Once per job is the second thing this count pins, and it is about the timing
# series rather than the memory one: the contamination moves the mean, so a job
# that skipped the warm-up would report slower numbers than the job it is compared
# against and read a regression off the difference between two jobs. Both jobs run
# it, and both run the same task.
assert_occurs_exactly "$root_dir/.github/workflows/bench.yml" '^        run: mise run bench_warmup > /dev/null$' "$bench_job_count"
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
assert_contains "$root_dir/console/Trunk.toml" '^filehash[[:space:]]*=[[:space:]]*false$'
# Persistence ships on, and every feature arm above is stated relative to that.
# With `default = ["persistence"]` the plain workspace runs are the persistence
# arm and `--no-default-features` is the feature-off arm. Flip this back to `[]`
# and both arms become feature-off — nothing left in either tier would compile
# the storage path — so the manifest and the tier lines are pinned together
# rather than one being left free to invalidate the other.
console_features_table='^[[:space:]]*[[]features[]][[:space:]]*$'
assert_toml_table_contains "$root_dir/console/Cargo.toml" "$console_features_table" '^[[:space:]]*default[[:space:]]*=[[:space:]]*[[]"persistence"[]]$'
# The feature has to stay a feature: inlining it would remove the build
# `--no-default-features` proves, which is the criterion the default-on decision
# was taken against rather than in place of.
assert_toml_table_contains "$root_dir/console/Cargo.toml" "$console_features_table" '^[[:space:]]*persistence[[:space:]]*=[[:space:]]*[[].*"eframe/persistence".*"orcvs/persistence".*[]]$'
assert_contains "$root_dir/console/assets/sw.js" "'./console.js'"
assert_contains "$root_dir/console/assets/sw.js" "'./console_bg.wasm'"
assert_contains "$root_dir/console/assets/sw.js" "^var cacheName = 'orcvs-pwa-v[0-9]+';$"
assert_contains "$root_dir/console/assets/sw.js" "self[.]addEventListener[(]'activate'"
assert_contains "$root_dir/console/assets/sw.js" 'caches[.]keys[(][)]'
assert_contains "$root_dir/console/assets/sw.js" 'caches[.]delete[(]name[)]'
assert_contains "$root_dir/console/assets/sw.js" "name === 'orcvs-pwa'"
assert_contains "$root_dir/console/assets/sw.js" "name === 'egui-template-pwa'"
assert_contains "$root_dir/console/assets/sw.js" "name[.]startsWith[(]'orcvs-pwa-'[)]"
assert_contains "$root_dir/console/assets/sw.js" 'return isOrcvsCache && name !== cacheName;'
assert_contains "$root_dir/console/assets/sw.js" 'caches[.]open[(]cacheName[)]'
assert_contains "$root_dir/console/assets/sw.js" 'cache[.]match[(]e[.]request[)]'
assert_not_contains "$root_dir/console/assets/sw.js" 'caches[.]match[(]e[.]request[)]'
assert_contains "$root_dir/console/assets/sw.js" 'self[.]skipWaiting[(][)]'
assert_contains "$root_dir/console/assets/sw.js" 'self[.]clients[.]claim[(][)]'
assert_contains "$root_dir/console/assets/sw.js" "e[.]request[.]mode === 'navigate'"
assert_contains "$root_dir/console/assets/sw.js" "e[.]request[.]url[.]endsWith[(]'/console[.]js'[)]"
assert_contains "$root_dir/console/assets/sw.js" "e[.]request[.]url[.]endsWith[(]'/console_bg[.]wasm'[)]"
assert_contains "$root_dir/console/assets/sw.js" "fetch[(]e[.]request, \{ cache: 'no-cache' \}[)]"
assert_contains "$root_dir/console/assets/sw.js" 'response[.]ok'
assert_contains "$root_dir/console/assets/sw.js" 'cache[.]put[(]e[.]request, response[.]clone[(][)][)][.]catch'
assert_contains "$root_dir/console/assets/sw.js" 'response[[:space:]]*[|][|][[:space:]]*Response[.]error[(][)]'
assert_contains "$root_dir/.vscode/launch.json" '"--package=orcvs",'
assert_contains "$root_dir/.vscode/launch.json" '"--package=console"'
assert_not_contains "$root_dir/.vscode/launch.json" '(package|bin)=shell'
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
# macOS and the two extended steps run at final verification. The aggregate
# always runs, so failed, cancelled, or unexpectedly skipped jobs cannot pass.
assert_contains "$root_dir/.github/workflows/test.yml" '^  merge_group:$'
assert_contains "$root_dir/.github/workflows/test.yml" '^    types: \[checks_requested\]$'
assert_occurs_exactly "$root_dir/.github/workflows/test.yml" "if: github.event_name != 'pull_request'$" 3
assert_occurs_exactly "$root_dir/.github/workflows/test.yml" '^[[:space:]]*(- )?if[[:space:]]*:' 4
assert_contains "$root_dir/.github/workflows/test.yml" '^    if: always[(][)]$'
assert_contains "$root_dir/.github/workflows/test.yml" '^    needs: \[full-gate, macos, wasm\]$'
assert_contains "$root_dir/.github/workflows/test.yml" '^        run: bash scripts/check-ci-results.sh$'
assert_not_contains "$root_dir/.github/workflows/test.yml" "if: github.event_name == 'push'$"
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
  if grep -Ev '^[[:space:]]*#' "$workflow" | grep -E '^[[:space:]]*-?[[:space:]]*run: mise (run|r) miri([^[:alnum:]_-]|$)' >/dev/null; then
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
assert_not_contains "$root_dir/console/Cargo.toml" '^[[:space:]]*criterion([.]workspace)?[[:space:]]*='
assert_not_contains "$root_dir/lang/Cargo.toml" '^[[:space:]]*criterion[.]workspace[[:space:]]*='
assert_not_contains "$root_dir/orcvs/Cargo.toml" '^[[:space:]]*criterion[.]workspace[[:space:]]*='
assert_not_contains "$root_dir/Cargo.toml" '^\[profile\.ci\]$'
assert_contains "$root_dir/Cargo.toml" '^tokio[[:space:]]*=[[:space:]]*\{[^}]*version[[:space:]]*='
for manifest in "$root_dir/orcvs/Cargo.toml" "$root_dir/console/Cargo.toml"; do
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
assert_not_contains "$root_dir/console/Cargo.toml" '^[[:space:]]*proptest([.]workspace)?[[:space:]]*='
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
# This pins non-ignoredness and nothing else. It is deliberately not an existence
# check: a counterexample exists only for a property that has failed, so requiring
# these paths would fail every green checkout, and four of the five name files
# that have never been committed. A counterexample generated locally and deleted
# before a commit is invisible here and to CI alike — it was never tracked, so
# there is no deletion to find. docs/tooling.md states the discipline this cannot
# enforce, and which counterexamples are worth keeping.
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

# `midir` is the one dependency in this workspace that links a system audio
# library — ALSA on Linux, CoreMIDI on macOS — and `native-midi` is what decides
# whether a build has it. The feature is on by default, so every build that
# exists today resolves as it did; what turning it off gives up is MIDI
# delivery, and what it buys is a tree with no `midir` and no audio library in
# it. Two conditions have to hold together for that, and each is pinned here.
#
# The feature has to be the only way in. `midir` declared without `optional`, or
# declared in a table the feature does not gate, is a dependency that arrives
# whatever the feature says — and the tree check in `audit_deps` is the only
# other thing that would notice.
midir_native_table='^[[]target[.].cfg[(]any[(]target_os = "macos", target_os = "windows", target_os = "linux"[)][)].[.]dependencies[]]$'
assert_contains "$root_dir/orcvs/Cargo.toml" '^default = \["native-midi"\]$'
assert_contains "$root_dir/orcvs/Cargo.toml" '^native-midi = \["dep:midir"\]$'
# Bracket forms rather than backslash escapes in every table pattern here and
# below: they reach the matcher through `awk -v`, which drops the backslash from
# `\{` and `\[` and turns what is left of `\["native-midi"\]` into a character
# class carrying an `e-m` range. That class matches the `f` of
# `default-features = false`, so the escaped spelling of the console assertion at
# the end of this file passed against a console that had stopped naming the
# feature at all.
assert_toml_table_contains "$root_dir/orcvs/Cargo.toml" "$midir_native_table" '^[[:space:]]*midir[[:space:]]*=[[:space:]]*[{][^}]*optional[[:space:]]*=[[:space:]]*true'
assert_toml_table_not_contains "$root_dir/orcvs/Cargo.toml" '^[[:space:]]*[[]dependencies[]][[:space:]]*$' '^[[:space:]]*midir[[:space:]]*='
# And the target table has to stay, because the feature alone does not say where
# a MIDI service could exist. Without it a default-featured WASM build would ask
# Cargo for `midir`, which is the one thing this arrangement must not change.
assert_toml_table_not_contains "$root_dir/orcvs/Cargo.toml" '^[[]target[.].cfg[(]target_arch = "wasm32"[)].[.]dependencies[]]$' '^[[:space:]]*midir[[:space:]]*='
#
# The console asks for the feature by name for its native targets rather than
# leaning on `orcvs`'s default, so the shipped application keeps its MIDI
# destination list even if that default changes, and the browser build keeps
# asking for no native backend at all.
#
# Three assertions rather than two, because the declaration is not two halves:
# Cargo unions a dependency's declarations and honours `default-features =
# false` only if every one of them says it, so the native table has to say it
# too. Pinning it in the plain table alone accepted a native declaration that
# had dropped it — a console back on the borrowed default, which is the one
# thing naming the feature exists to prevent, passing green.
assert_toml_table_contains "$root_dir/console/Cargo.toml" '^[[:space:]]*[[]dependencies[]][[:space:]]*$' '^[[:space:]]*orcvs[[:space:]]*=[[:space:]]*[{][^}]*default-features[[:space:]]*=[[:space:]]*false'
assert_toml_table_contains "$root_dir/console/Cargo.toml" '^[[]target[.].cfg[(]not[(]target_arch = "wasm32"[)][)].[.]dependencies[]]$' '^[[:space:]]*orcvs[[:space:]]*=[[:space:]]*[{][^}]*default-features[[:space:]]*=[[:space:]]*false'
# The feature list is pinned by what it contains, not by how long it is.
# `console` already has a `persistence` feature that maps onto
# `orcvs/persistence`, so a second entry beside `native-midi` is a manifest this
# contract should accept; requiring the list to be exactly `["native-midi"]`
# rejected it with a message that read as though the feature were missing.
assert_toml_table_contains "$root_dir/console/Cargo.toml" '^[[]target[.].cfg[(]not[(]target_arch = "wasm32"[)][)].[.]dependencies[]]$' '^[[:space:]]*orcvs[[:space:]]*=[[:space:]]*[{][^}]*[^-]features[[:space:]]*=[[:space:]]*[[]([^]]*,[[:space:]]*)?"native-midi"'

# Every ADR takes a number no other ADR takes. `0036` named two accepted
# decisions for a day — the pulse refusal and the Cell reservation — and every
# one of the 78 bare "ADR 0036" citations outside `docs/adr/` resolved to either
# of them, in comments that sat within thirty lines of each other. Nothing
# noticed, because nothing was looking: the number is chosen by whoever writes
# the file, and two branches open at once choose the same one. `docs/adr/README.md`
# states which of the two gets renumbered; this is what fails when a third pair
# is written. Gaps are not checked — `0027` is missing from this directory
# because an unlanded branch claims it, and a number nothing here answers to
# breaks no citation — and neither is order. A number appearing twice is the one
# thing that makes a citation ambiguous, so it is the one thing asserted.
#
# Guarded on the directory existing because the fixture suite in `scripts/tests/`
# builds a tree from the manifests and workflows alone, with no `docs/` in it.
adr_dir="$root_dir/docs/adr"
if [ -d "$adr_dir" ]; then
  duplicate_adr_numbers="$(find "$adr_dir" -maxdepth 1 -name '*.md' -exec basename {} \; | grep -Eo '^[0-9]{4}' | sort | uniq -d || true)"
  if [ -n "$duplicate_adr_numbers" ]; then
    echo "expected every ADR in docs/adr/ to take a number no other ADR takes; duplicated:" >&2
    printf '%s\n' "$duplicate_adr_numbers" >&2
    exit 1
  fi
fi
