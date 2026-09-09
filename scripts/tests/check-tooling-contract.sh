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
  mkdir -p "$fixture_dir/scripts" "$fixture_dir/.github/workflows" "$fixture_dir/.vscode" "$fixture_dir/shell/assets" "$fixture_dir/orcvs" "$fixture_dir/lang"
  # The contract asks git whether the proptest regression files are ignored, so a
  # fixture has to be a work tree or that check cannot run against it at all.
  git -C "$fixture_dir" init --quiet
  cp "${CHECKER_SOURCE:-$repo_root/scripts/check-tooling-contract.sh}" "$fixture_dir/scripts/check-tooling-contract.sh"
  cp "$repo_root/mise.toml" "$repo_root/Cargo.toml" "$fixture_dir/"
  cp "$repo_root/shell/Cargo.toml" "$repo_root/shell/Trunk.toml" "$fixture_dir/shell/"
  cp "$repo_root/shell/assets/sw.js" "$fixture_dir/shell/assets/"
  cp "$repo_root/orcvs/Cargo.toml" "$fixture_dir/orcvs/"
  cp "$repo_root/lang/Cargo.toml" "$fixture_dir/lang/"
  # `miri.yml` is copied like the other three. The contract's Miri rules are
  # stated over whichever workflow runs the task rather than over a file name,
  # so without the file here nothing in the fixture matches `run: mise run miri`
  # and every one of those rules is dead code in this suite.
  cp "$repo_root/.github/workflows/test.yml" "$repo_root/.github/workflows/bench.yml" "$repo_root/.github/workflows/advisories.yml" "$repo_root/.github/workflows/miri.yml" "$fixture_dir/.github/workflows/"
  cp "$repo_root/.github/dependabot.yml" "$fixture_dir/.github/"
  cp "$repo_root/.vscode/launch.json" "$fixture_dir/.vscode/"
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

assert_accepted() {
  local scenario="$1"
  local output
  if ! output="$(bash "$fixture_dir/scripts/check-tooling-contract.sh" 2>&1)"; then
    echo "expected tooling contract to accept $scenario" >&2
    printf '%s\n' "$output" >&2
    return 1
  fi
}

test_commented_requirement_is_rejected() {
  make_fixture
  perl -pi -e 's/^(RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --features persistence --locked)$/# $1/' "$fixture_dir/mise.toml"
  assert_rejected "a commented-out required setting"
}

test_unbenchmarked_orcvs_is_rejected() {
  make_fixture
  perl -pi -e "s/^(      - 'orcvs\\/\\*\\*')\$/# \$1/" "$fixture_dir/.github/workflows/bench.yml"
  assert_rejected "a benchmark workflow that ignores changes to the benchmarked orcvs crate"
}

test_lang_only_bench_task_is_rejected() {
  make_fixture
  perl -pi -e 's/cargo bench --package lang --package orcvs --benches/cargo bench --package lang/' "$fixture_dir/mise.toml"
  assert_rejected "a benchmark task that measures only the lang crate"
}

test_missing_orcvs_criterion_is_rejected() {
  make_fixture
  perl -pi -e 's/^criterion = /# criterion = /' "$fixture_dir/orcvs/Cargo.toml"
  assert_rejected "an orcvs crate without the criterion dev-dependency"
}

test_shipped_orcvs_criterion_is_rejected() {
  make_fixture
  perl -pi -e 's/^\[dev-dependencies\]$/[dependencies]/' "$fixture_dir/orcvs/Cargo.toml"
  assert_rejected "a criterion dependency that ships in the orcvs library"
}

test_bench_without_native_dependencies_is_rejected() {
  make_fixture
  perl -pi -e 's/^(        run: sudo apt-get update .*)$/# $1/' "$fixture_dir/.github/workflows/bench.yml"
  assert_rejected "a benchmark workflow that never installs the orcvs native dependencies"

  # Both bench jobs build `orcvs`, so losing the install step from one of them is
  # just as broken as losing it from both.
  make_fixture
  perl -pi -e 'if (!$done && s/^(        run: sudo apt-get update .*)$/# $1/) { $done = 1 }' "$fixture_dir/.github/workflows/bench.yml"
  assert_rejected "a benchmark workflow whose first bench job never installs the orcvs native dependencies"
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
  perl -pi -e 's/wasm-pack test --headless --firefox shell --test wasm --locked/wasm-pack test --headless --firefox shell --test wasm/' "$fixture_dir/mise.toml"
  assert_rejected "an unlocked wasm-pack test invocation"
}

test_wasm_build_without_the_feature_off_arm_is_rejected() {
  make_fixture
  # `persistence` is a default feature, so the two trunk builds are only two
  # configurations while one of them says `--no-default-features`. Dropping that
  # leaves the task building the same application twice and compiling the
  # storage-free WASM build nowhere.
  perl -pi -e 's/trunk build --no-default-features --locked/trunk build --locked/' "$fixture_dir/mise.toml"
  assert_rejected "a WASM task that never builds the feature-off configuration"
}

test_missing_default_wasm_build_is_rejected() {
  make_fixture
  perl -pi -e 's/^env -u NO_COLOR trunk build --locked\n$//' "$fixture_dir/mise.toml"
  assert_rejected "a missing default-feature WASM build"
}

test_wasm_test_without_persistence_is_rejected() {
  make_fixture
  # The browser suite runs the shipped configuration, and storage is in it by
  # default. Opting out here would leave the one gate that drives a real browser
  # exercising a build the browser never loads.
  perl -pi -e 's/wasm-pack test --headless --firefox shell --test wasm --locked/wasm-pack test --headless --firefox shell --test wasm --no-default-features --locked/' "$fixture_dir/mise.toml"
  assert_rejected "browser tests that opt out of the persistence default"
}

test_stale_wasm_artifact_name_is_rejected() {
  make_fixture
  perl -pi -e 's/shell_bg[.]wasm/console_bg.wasm/' "$fixture_dir/shell/assets/sw.js"
  assert_rejected "a stale WASM artifact name in the service worker cache"
}

test_hashed_wasm_artifacts_are_rejected() {
  make_fixture
  perl -pi -e 's/filehash = false/filehash = true/' "$fixture_dir/shell/Trunk.toml"
  assert_rejected "hashed WASM artifacts with fixed service-worker cache names"
}

test_stale_script_artifact_name_is_rejected() {
  make_fixture
  perl -pi -e "s|'./shell[.]js'|'./console.js'|" "$fixture_dir/shell/assets/sw.js"
  assert_rejected "a stale script artifact name in the service worker cache"
}

test_service_worker_without_cache_invalidation_is_rejected() {
  make_fixture
  perl -0pi -e "s/self[.]addEventListener[(]'activate'.*?^}[)];\n//ms" "$fixture_dir/shell/assets/sw.js"
  assert_rejected "a service worker without versioned cache invalidation"
}

test_service_worker_deleting_unrelated_caches_is_rejected() {
  make_fixture
  perl -pi -e 's/return isOrcvsCache && name !== cacheName;/return name !== cacheName;/' "$fixture_dir/shell/assets/sw.js"
  assert_rejected "a service worker that deletes unrelated origin caches"
}

test_service_worker_without_legacy_cache_cleanup_is_rejected() {
  make_fixture
  perl -pi -e "s/egui-template-pwa/unrelated-pwa/" "$fixture_dir/shell/assets/sw.js"
  assert_rejected "a service worker that leaves its legacy cache behind"
}

test_service_worker_caching_error_responses_is_rejected() {
  make_fixture
  perl -pi -e 's/if [(]response[.]ok[)]/if (true)/' "$fixture_dir/shell/assets/sw.js"
  assert_rejected "a service worker that caches HTTP error responses"
}

test_service_worker_discarding_live_response_on_cache_failure_is_rejected() {
  make_fixture
  perl -pi -e 's/cache[.]put[(]e[.]request, response[.]clone[(][)][)][.]catch/cache.put(e.request, response.clone()).then/' "$fixture_dir/shell/assets/sw.js"
  assert_rejected "a service worker that discards a live response when cache storage fails"
}

test_service_worker_without_explicit_offline_error_is_rejected() {
  make_fixture
  perl -pi -e 's/response [|][|] Response[.]error[(][)]/response/' "$fixture_dir/shell/assets/sw.js"
  assert_rejected "a service worker whose offline cache miss resolves without a response"
}

test_service_worker_without_immediate_activation_is_rejected() {
  make_fixture
  perl -pi -e 's/self[.]skipWaiting[(][)]/self.waitForOldClients()/' "$fixture_dir/shell/assets/sw.js"
  assert_rejected "a service worker that waits for every old tab to close"
}

test_service_worker_without_immediate_control_is_rejected() {
  make_fixture
  perl -pi -e 's/self[.]clients[.]claim[(][)]/self.clients.waitForReload()/' "$fixture_dir/shell/assets/sw.js"
  assert_rejected "a service worker that does not control existing tabs after activation"
}

test_service_worker_with_cache_first_navigation_is_rejected() {
  make_fixture
  perl -pi -e "s/e[.]request[.]mode === 'navigate'/e.request.mode === 'cached-navigation'/" "$fixture_dir/shell/assets/sw.js"
  assert_rejected "cache-first navigation after a deploy"
}

test_service_worker_with_cache_first_stable_artifacts_is_rejected() {
  make_fixture
  perl -pi -e "s|e[.]request[.]url[.]endsWith[(]'/shell[.]js'[)]|false|" "$fixture_dir/shell/assets/sw.js"
  assert_rejected "cache-first stable JavaScript after an unchanged service-worker deploy"

  make_fixture
  perl -pi -e "s|e[.]request[.]url[.]endsWith[(]'/shell_bg[.]wasm'[)]|false|" "$fixture_dir/shell/assets/sw.js"
  assert_rejected "cache-first stable WASM after an unchanged service-worker deploy"
}

test_service_worker_using_the_default_http_cache_is_rejected() {
  make_fixture
  perl -pi -e "s/fetch[(]e[.]request, \{ cache: 'no-cache' \}[)]/fetch(e.request)/" "$fixture_dir/shell/assets/sw.js"
  assert_rejected "network-first stable artifacts that can use the HTTP cache"
}

test_stale_debug_package_is_rejected() {
  make_fixture
  perl -pi -e 's/("--package=shell")/$1,\n                    "--package=vtha"/' "$fixture_dir/.vscode/launch.json"
  assert_rejected "a stale package name in the debugger configuration"
}

test_stale_benchmark_is_rejected() {
  make_fixture
  perl -pi -e 's/("--package=shell")/$1,\n                    "--bin=parser_benchmark"/' "$fixture_dir/.vscode/launch.json"
  assert_rejected "a retired parser benchmark in the debugger configuration"
}

test_console_debug_package_is_rejected() {
  make_fixture
  perl -pi -e 's/("--package=shell")/$1,\n                    "--package=console"/' "$fixture_dir/.vscode/launch.json"
  assert_rejected "the retired console package in the debugger configuration"
}

test_missing_orcvs_persistence_check_is_rejected() {
  make_fixture
  perl -pi -e 's/^cargo check --package orcvs --lib --features persistence --locked\n$//' "$fixture_dir/mise.toml"
  assert_rejected "a missing per-package orcvs persistence check"
}

test_persistence_command_in_wrong_task_is_rejected() {
  make_fixture
  perl -0pi -e 's/(\[tasks\.test_persistence\].*?)cargo clippy --workspace --all-targets --features persistence --locked -- -D warnings\n/$1/s' "$fixture_dir/mise.toml"
  printf '\ncargo clippy --workspace --all-targets --features persistence --locked -- -D warnings\n' >> "$fixture_dir/mise.toml"
  assert_rejected "a persistence command outside test_persistence"
}

test_dotted_dependency_version_is_rejected() {
  make_fixture
  perl -pi -e 'if (!$done && s/^tokio = \{ workspace = true, features = \["rt", "macros", "time"\] \}$/tokio.workspace = true\ntokio.version = "9.0.0"/) { $done = 1 }' "$fixture_dir/orcvs/Cargo.toml"
  assert_rejected "a crate-local dependency version in dotted TOML syntax"
}

test_dependency_table_version_is_rejected() {
  make_fixture
  perl -pi -e 'if (!$done && s/^tokio = \{ workspace = true, features = \["rt", "macros", "time"\] \}$/[dependencies.tokio]\nworkspace = true\nversion = "9.0.0"/) { $done = 1 }' "$fixture_dir/orcvs/Cargo.toml"
  assert_rejected "a crate-local dependency version in TOML table syntax"
}

test_commented_dependency_table_version_is_rejected() {
  make_fixture
  perl -pi -e 'if (!$done && s/^tokio = \{ workspace = true, features = \["rt", "macros", "time"\] \}$/[dependencies.tokio] # local override\nworkspace = true\nversion = "9.0.0"/) { $done = 1 }' "$fixture_dir/orcvs/Cargo.toml"
  assert_rejected "a crate-local dependency version in a TOML table with a trailing comment"
}

test_shared_push_concurrency_group_is_rejected() {
  make_fixture
  perl -pi -e 's/github[.]event[.]pull_request[.]number [|][|] github[.]sha/github.event.pull_request.number || github.ref/' "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "a concurrency group that collapses every push to main into one run"
}

test_unconditional_cancellation_is_rejected() {
  make_fixture
  perl -pi -e "s/^(  cancel-in-progress:).*\$/\$1 true/" "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "cancellation that is not confined to pull requests"
}

test_dispatch_skipping_the_merge_tier_is_rejected() {
  make_fixture
  perl -pi -e "s/if: github[.]event_name != 'pull_request'/if: github.event_name == 'push'/" "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "merge-tier steps a manual dispatch would skip"
}

test_merge_only_wasm_job_is_rejected() {
  make_fixture
  perl -pi -e "s/^(  wasm:)\$/\$1\n    if: github.event_name != 'pull_request'/" "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "a WASM job that stopped running on pull requests"
}

test_patch_bump_of_a_pinned_action_is_accepted() {
  make_fixture
  perl -pi -e 's/# v7[.]0[.]1$/# v7.0.2/; s/# v4[.]3[.]0$/# v4.4.0/' "$fixture_dir/.github/workflows/test.yml"
  assert_accepted "a patch bump of a SHA-pinned action"
}

test_major_bump_of_a_pinned_action_is_rejected() {
  make_fixture
  perl -pi -e 's/# v7[.]0[.]1$/# v8.0.0/' "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "a major bump of a SHA-pinned action"
}

test_mutable_major_tag_for_a_tool_action_is_rejected() {
  make_fixture
  printf '\n      - uses: some-org/nextest@v1\n' >> "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "a tool-install action pinned to a mutable major tag"
}

test_expression_form_merge_guard_is_rejected() {
  make_fixture
  perl -pi -e "s/^(  wasm:)\$/\$1\n    if: \\\$\{\{ github.event_name != 'pull_request' \}\}/" "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "a merge guard written in expression syntax the count cannot see"
}

test_dropped_native_merge_component_is_rejected() {
  make_fixture
  perl -pi -e 's/ORCVS_MERGE_COMPONENT: native$/ORCVS_MERGE_COMPONENT: wasm/' "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "a workflow that no longer runs the native merge component"
}

test_untimed_workflow_job_is_rejected() {
  make_fixture
  printf '\n  extra:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo a job with no bound on its runtime\n' >> "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "a workflow job added without a timeout"

  # The count is per file, so a job that loses its timeout has to fail in the
  # file that lost it rather than being covered by another workflow's total.
  make_fixture
  perl -pi -e 'if (!$done && s/^(    timeout-minutes: [0-9]+)$/# $1/) { $done = 1 }' "$fixture_dir/.github/workflows/bench.yml"
  assert_rejected "a benchmark job whose timeout was commented out"
}

test_advisory_audit_without_a_schedule_is_rejected() {
  make_fixture
  perl -pi -e 's/^(    - cron: )/# $1/' "$fixture_dir/.github/workflows/advisories.yml"
  assert_rejected "an advisory audit with no schedule to run it"

  # The cron line surviving under a commented-out `schedule:` key is the same
  # silence with a plausible-looking file to point at, so both halves are pinned.
  make_fixture
  perl -pi -e 's/^  schedule:$/# schedule:/' "$fixture_dir/.github/workflows/advisories.yml"
  assert_rejected "an advisory workflow whose schedule trigger was commented out"
}

test_advisory_workflow_without_the_audit_is_rejected() {
  make_fixture
  perl -pi -e 's/^(      - run: )mise run audit_deps$/$1mise run check_pull_request/' "$fixture_dir/.github/workflows/advisories.yml"
  assert_rejected "a scheduled workflow that runs something other than the advisory audit"

  # The pinning rules are per file, so the newest workflow needs them asserted
  # against it rather than inherited from the two that came before.
  make_fixture
  perl -pi -e 's/^(      - uses: jdx\/mise-action@)[0-9a-f]{40}( +# v4.*)$/$1v4/' "$fixture_dir/.github/workflows/advisories.yml"
  assert_rejected "an advisory workflow whose mise-action is pinned to a mutable tag"
}

test_unpinned_workflow_linter_is_rejected() {
  make_fixture
  perl -pi -e 's/^("aqua:rhysd\/actionlint" = )"[0-9.]+"$/$1"latest"/' "$fixture_dir/mise.toml"
  assert_rejected "a workflow syntax linter tracking latest rather than a pinned version"

  make_fixture
  perl -pi -e 's/^("aqua:zizmorcore\/zizmor" = )"[0-9.]+"$/$1"1"/' "$fixture_dir/mise.toml"
  assert_rejected "a workflow security linter pinned only to a major version"
}

test_automatically_triggered_miri_workflow_is_rejected() {
  # `verification-gaps/12` kept Miri as a path taken on purpose. Every trigger
  # below turns it back into something a change pays for without anyone
  # deciding to, which is that decision reversed by a different route.
  make_fixture
  perl -pi -e "s/^on:\$/on:\n  schedule:\n    - cron: '0 3 * * *'/" "$fixture_dir/.github/workflows/miri.yml"
  assert_rejected "a Miri workflow running on a schedule"

  make_fixture
  perl -pi -e "s/^on:\$/on:\n  push:\n    branches: [main]/" "$fixture_dir/.github/workflows/miri.yml"
  assert_rejected "a Miri workflow running on a push"

  make_fixture
  perl -pi -e 's/^on:$/on:\n  workflow_call:/' "$fixture_dir/.github/workflows/miri.yml"
  assert_rejected "a Miri workflow another workflow can call"

  make_fixture
  perl -pi -e 's/^  workflow_dispatch:$//' "$fixture_dir/.github/workflows/miri.yml"
  assert_rejected "a Miri workflow nobody can dispatch"
}

test_miri_called_by_another_task_is_rejected() {
  # No tier calls it, in any spelling. `mise r` is the same call as `mise run`,
  # and a second command on one line is the same call again.
  make_fixture
  perl -pi -e 's/^(\[tasks\.bench\])$/[tasks.sneak]\nrun = "mise r miri"\n\n$1/' "$fixture_dir/mise.toml"
  assert_rejected "a task calling Miri through mise's run abbreviation"

  make_fixture
  perl -pi -e 's/^(cargo fmt --all -- --check)$/$1 \&\& mise run miri/' "$fixture_dir/mise.toml"
  assert_rejected "a tier calling Miri as the second half of a line"
}

test_memory_series_measured_by_a_second_binary_is_rejected() {
  # The series is fed by the very test functions that assert on the numbers. A
  # run that measures some other way lets the published number and the asserted
  # number drift apart, which is the one thing the series exists to prevent.
  make_fixture
  perl -pi -e 's/^(        run: ORCVS_MEMORY_SERIES=1 cargo test .*)$/# $1/' "$fixture_dir/.github/workflows/bench.yml"
  assert_rejected "a bench workflow publishing no allocation measurement"

  make_fixture
  perl -pi -e 's/^(        run: ORCVS_MEMORY_SERIES=1 )cargo test( --package lang)/$1cargo nextest run$2/' "$fixture_dir/.github/workflows/bench.yml"
  assert_rejected "an allocation measurement asking for a cargo tool the bench jobs do not install"
}

test_memory_series_assembled_with_jq_is_rejected() {
  # Shell builtins and coreutils only, so the step installs nothing and
  # `mise.toml` gains no tool for it.
  make_fixture
  perl -pi -e "s/^(          printf '\\[%s\\]).*\$/          jq -s '.' > memory.json/" "$fixture_dir/.github/workflows/bench.yml"
  assert_rejected "a memory series assembled with jq"
}

test_contract_assertions_read_their_whole_input() {
  # The contract script runs under `set -o pipefail`, so an assertion that pipes
  # into `grep -q` reports failure for a pattern that matched: `-q` exits on the
  # first match, the upstream grep dies of SIGPIPE, and pipefail surfaces its 141
  # as the pipeline's status. Whether that happens is a race on how much the
  # upstream has written, which is why it hid on macOS and on small files and
  # then failed 160 of 200 identical assertions against a 350-line `bench.yml` on
  # Linux — every one of them red for a line that was present.
  #
  # A race cannot be caught by running the suite once, so this asserts the shape
  # instead of the symptom: no assertion helper may pipe into a short-circuiting
  # grep. `grep -c` and a plain `grep` redirected to /dev/null both read to end
  # of input and are safe.
  local offenders
  offenders="$(grep -n '|[[:space:]]*grep -[A-Za-z]*q' "$repo_root/scripts/check-tooling-contract.sh" || true)"
  if [ -n "$offenders" ]; then
    echo "the contract script pipes into a short-circuiting grep -q, which races with pipefail:" >&2
    printf '%s\n' "$offenders" >&2
    return 1
  fi
}

test_pull_request_tier_without_workflow_linting_is_rejected() {
  make_fixture
  perl -pi -e 's/^actionlint\n$//' "$fixture_dir/mise.toml"
  assert_rejected "a pull-request tier that never checks the workflows for syntax"

  make_fixture
  perl -pi -e 's/^zizmor --offline [.]github\/workflows\n$//' "$fixture_dir/mise.toml"
  assert_rejected "a pull-request tier that never audits the workflows for injection and permission findings"
}

test_unwatched_rust_toolchain_is_rejected() {
  make_fixture
  perl -pi -e 's/^(  - package-ecosystem: rust-toolchain)$/# $1/' "$fixture_dir/.github/dependabot.yml"
  assert_rejected "a Dependabot config that never reads the pinned Rust channel"

  # `cargo` and `rust-toolchain` are separate ecosystems: the first never touches
  # the channel, so satisfying this by renaming the other entry is not satisfying it.
  make_fixture
  perl -pi -e 's/^(  - package-ecosystem: )cargo$/$1rust-toolchain/' "$fixture_dir/.github/dependabot.yml"
  assert_rejected "a Dependabot config that watches the channel instead of the manifests"
}

test_pull_request_tier_without_feature_off_doctests_is_rejected() {
  make_fixture
  perl -0pi -e 's/cargo test --workspace --doc --locked\ncargo test --workspace --doc --no-default-features --locked\n/cargo test --workspace --doc --locked\n/' "$fixture_dir/mise.toml"
  assert_rejected "a pull-request tier that never compiles the feature-off doctests"
}

test_optional_persistence_default_is_rejected() {
  make_fixture
  # Every feature arm in the tiers is stated relative to the shell default. With
  # `default = []` the plain workspace runs stop compiling the storage path and
  # the `--no-default-features` runs beside them test the same thing, so the pair
  # collapses into one configuration and persistence is verified nowhere.
  perl -pi -e 's/^default = \["persistence"\]$/default = []/' "$fixture_dir/shell/Cargo.toml"
  assert_rejected "a shell manifest that ships persistence off"

  # The other way to lose the pair is to stop the feature being a feature at all.
  # `--no-default-features` then proves nothing, and the acceptance criterion the
  # default-on decision was taken against — that the path still compiles out —
  # has no build behind it.
  make_fixture
  perl -pi -e 's|^persistence = \["eframe/persistence", "orcvs/persistence"\]$|# $&|' "$fixture_dir/shell/Cargo.toml"
  assert_rejected "a shell manifest with no persistence feature to switch off"
}

test_prohibited_action_main_ref_is_rejected() {
  make_fixture
  printf '\n      - uses: taiki-e/install-action@main\n' >> "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "a prohibited action using an unpinned ref"
}

test_ungated_rust_cache_save_is_rejected() {
  make_fixture
  # A caching step that writes on every ref, which is how the shared quota filled
  # with pull-request entries no run can read.
  perl -0pi -e "s/^          save-if: .*\n//m" "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "a rust-cache step that saves on every ref"

  # The count is against the number of caching steps, so a new job arriving with
  # an ungated cache has to fail too rather than only a gate being deleted.
  make_fixture
  printf '\n  extra:\n    timeout-minutes: 20\n    runs-on: ubuntu-latest\n    steps:\n      - uses: Swatinem/rust-cache@6323deb102c322ba6fcbdcafc7e3dddab59af2b6 # v2\n' >> "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "a job whose cache step arrived without the save gate"
}

test_ungated_mise_cache_save_is_rejected() {
  make_fixture
  perl -0pi -e "s/^          cache_save: .*\n//m" "$fixture_dir/.github/workflows/advisories.yml"
  assert_rejected "a mise-action step that saves on every ref"
}

test_unshared_bench_cache_key_is_rejected() {
  make_fixture
  perl -pi -e 's/^          shared-key: bench$/          shared-key: publish/' "$fixture_dir/.github/workflows/bench.yml"
  assert_rejected "benchmark jobs keyed apart from each other"
}

test_stale_bench_action_pin_is_rejected() {
  make_fixture
  perl -pi -e 's{^(      - uses: actions/checkout\@[0-9a-f]{40} )# v7([.][0-9]+)*$}{$1# v4}' "$fixture_dir/.github/workflows/bench.yml"
  assert_rejected "a benchmark checkout pinned a major behind the other workflows"
}

test_invalid_fresh_fixture_is_rejected() {
  if CHECKER_SOURCE="$repo_root/Cargo.toml" make_fixture 2>/dev/null; then
    echo "expected fixture setup to reject an invalid fresh fixture" >&2
    return 1
  fi
}

test_fixture_cleanup_removes_tmp_dirs_on_failure() {
  local tmp_root leaked_dirs status
  # A temporary root private to this scenario. Scanning the shared one instead
  # read every `tmp.*` directory in it as a candidate leak, so any concurrent
  # `mktemp -d` — another mise task, a parallel CI step, a second agent, cargo —
  # was reported as a leak and then deleted by the cleanup below.
  tmp_root="$(mktemp -d)"
  # invalid-fixture drives make_fixture down its early-failure path (an invalid
  # CHECKER_SOURCE), which is exactly the case where a leaked fixture dir would
  # otherwise survive the run.
  TMPDIR="$tmp_root" bash "$0" invalid-fixture >/dev/null 2>&1 || true
  # Everything under a root this scenario created is this scenario's own, so no
  # before-and-after comparison is needed to tell a leak from a bystander.
  leaked_dirs="$(find "$tmp_root" -mindepth 1 -maxdepth 1 -name 'tmp.*' 2>/dev/null | sort)"
  status=0
  if [ -n "$leaked_dirs" ]; then
    echo "expected no fixture directories to remain in $tmp_root after a failed scenario, found:" >&2
    printf '%s\n' "$leaked_dirs" >&2
    status=1
  fi
  rm -rf "$tmp_root"
  return "$status"
}

test_folded_merge_guard_is_rejected() {
  make_fixture
  # A folded scalar keeps the expression off the `if:` line entirely, which is
  # the spelling an assertion anchored to `if: ${{` cannot see.
  perl -pi -e "s/^(  wasm:)\$/\$1\n    if: >-\n      \\\$\{\{ github.event_name != 'pull_request' \}\}/" "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "a merge guard folded onto the line after the key"
}

test_spaced_key_merge_guard_is_rejected() {
  make_fixture
  # YAML permits whitespace between a key and its colon, so `if :` is the same key
  # to GitHub and a different string to every pattern anchored to `if:`. Spelled
  # this way a third guard left both counts reading two with three guards in place.
  perl -pi -e "s/^(  wasm:)\$/\$1\n    if : \\\$\{\{ github.event_name != 'pull_request' \}\}/" "$fixture_dir/.github/workflows/test.yml"
  assert_rejected "a merge guard whose key is spelled with a detached colon"
}

test_non_optional_midir_is_rejected() {
  make_fixture
  perl -pi -e 's/^midir = \{ version = "0.11", optional = true \}$/midir = "0.11"/' "$fixture_dir/orcvs/Cargo.toml"
  assert_rejected "a midir dependency that arrives whether native-midi is enabled or not"
}

test_shipped_midir_dependency_is_rejected() {
  make_fixture
  # The target table is what keeps `midir` out of a WASM build. Moved into the
  # plain table it is optional and feature-gated still, and every other
  # assertion about it holds, while a default-featured browser build now asks
  # Cargo for a crate that links CoreMIDI.
  perl -pi -e 's/^\[dependencies\]$/[dependencies]\nmidir = { version = "0.11", optional = true }/' "$fixture_dir/orcvs/Cargo.toml"
  assert_rejected "a midir dependency declared outside the native target table"
}

test_native_midi_off_by_default_is_rejected() {
  make_fixture
  perl -pi -e 's/^default = \["native-midi"\]$/default = []/' "$fixture_dir/orcvs/Cargo.toml"
  assert_rejected "an orcvs crate that no longer defaults native-midi on"
}

test_console_without_native_midi_is_rejected() {
  make_fixture
  perl -pi -e 's/, features = \["native-midi"\] \}$/ }/' "$fixture_dir/shell/Cargo.toml"
  assert_rejected "a console that asks for no native MIDI backend on its native targets"

  # Asking for it by name is the point: with `orcvs` defaulting the feature on,
  # a console that merely leaves the default alone still ships MIDI today and
  # loses it silently the day that default changes.
  make_fixture
  perl -pi -e 's/^orcvs = \{ path = "\.\.\/orcvs", version = "0\.1\.0", default-features = false \}$/orcvs = { path = "..\/orcvs", version = "0.1.0" }/' "$fixture_dir/shell/Cargo.toml"
  assert_rejected "a console that leans on the orcvs default instead of naming the feature"
}

test_pull_request_tier_without_the_disabled_feature_is_rejected() {
  make_fixture
  perl -pi -e 's/^(cargo clippy --package orcvs --all-targets --no-default-features --features persistence --locked -- -D warnings)$/# $1/' "$fixture_dir/mise.toml"
  assert_rejected "a pull-request tier that lints no build with native-midi disabled"

  make_fixture
  perl -pi -e 's/^(cargo nextest run --package orcvs --no-default-features --profile ci --locked)$/# $1/' "$fixture_dir/mise.toml"
  assert_rejected "a pull-request tier that runs no tests with native-midi disabled"
}

test_audit_without_the_disabled_tree_check_is_rejected() {
  make_fixture
  perl -pi -e 's/^(native_midi_tree=.*)$/# $1/' "$fixture_dir/mise.toml"
  assert_rejected "a dependency audit that never resolves the native-midi-disabled tree"

  # The tree is only evidence if something reads it. A printed tree that nothing
  # greps is the shape this check replaced.
  make_fixture
  perl -pi -e 's/^(if printf .*)$/# $1/' "$fixture_dir/mise.toml"
  assert_rejected "a dependency audit that resolves the tree and asserts nothing about it"
}

case "${1:-all}" in
  comments) test_commented_requirement_is_rejected ;;
  non-optional-midir) test_non_optional_midir_is_rejected ;;
  shipped-midir) test_shipped_midir_dependency_is_rejected ;;
  native-midi-default) test_native_midi_off_by_default_is_rejected ;;
  console-native-midi) test_console_without_native_midi_is_rejected ;;
  disabled-feature-tier) test_pull_request_tier_without_the_disabled_feature_is_rejected ;;
  disabled-feature-tree) test_audit_without_the_disabled_tree_check_is_rejected ;;
  unbenchmarked-orcvs) test_unbenchmarked_orcvs_is_rejected ;;
  lang-only-bench) test_lang_only_bench_task_is_rejected ;;
  missing-orcvs-criterion) test_missing_orcvs_criterion_is_rejected ;;
  shipped-orcvs-criterion) test_shipped_orcvs_criterion_is_rejected ;;
  bench-native-dependencies) test_bench_without_native_dependencies_is_rejected ;;
  unlocked-check-deny) test_unlocked_check_deny_is_rejected ;;
  unlocked-audit-deny) test_unlocked_audit_deny_is_rejected ;;
  unlocked-wasm-pack) test_unlocked_wasm_pack_is_rejected ;;
  wasm-build-feature-off) test_wasm_build_without_the_feature_off_arm_is_rejected ;;
  missing-default-wasm-build) test_missing_default_wasm_build_is_rejected ;;
  wasm-test-persistence) test_wasm_test_without_persistence_is_rejected ;;
  stale-wasm-artifact) test_stale_wasm_artifact_name_is_rejected ;;
  hashed-wasm-artifacts) test_hashed_wasm_artifacts_are_rejected ;;
  stale-script-artifact) test_stale_script_artifact_name_is_rejected ;;
  service-worker-cache-invalidation) test_service_worker_without_cache_invalidation_is_rejected ;;
  service-worker-cache-scope) test_service_worker_deleting_unrelated_caches_is_rejected ;;
  service-worker-legacy-cache) test_service_worker_without_legacy_cache_cleanup_is_rejected ;;
  service-worker-error-response) test_service_worker_caching_error_responses_is_rejected ;;
  service-worker-cache-write-failure) test_service_worker_discarding_live_response_on_cache_failure_is_rejected ;;
  service-worker-offline-miss) test_service_worker_without_explicit_offline_error_is_rejected ;;
  service-worker-immediate-activation) test_service_worker_without_immediate_activation_is_rejected ;;
  service-worker-immediate-control) test_service_worker_without_immediate_control_is_rejected ;;
  service-worker-navigation-strategy) test_service_worker_with_cache_first_navigation_is_rejected ;;
  service-worker-stable-artifacts) test_service_worker_with_cache_first_stable_artifacts_is_rejected ;;
  service-worker-http-cache) test_service_worker_using_the_default_http_cache_is_rejected ;;
  stale-debug-package) test_stale_debug_package_is_rejected ;;
  stale-debug-benchmark) test_stale_benchmark_is_rejected ;;
  console-debug-package) test_console_debug_package_is_rejected ;;
  missing-orcvs-persistence) test_missing_orcvs_persistence_check_is_rejected ;;
  dotted-dependency) test_dotted_dependency_version_is_rejected ;;
  dependency-table) test_dependency_table_version_is_rejected ;;
  commented-dependency-table) test_commented_dependency_table_version_is_rejected ;;
  shared-push-concurrency) test_shared_push_concurrency_group_is_rejected ;;
  unconditional-cancellation) test_unconditional_cancellation_is_rejected ;;
  dispatch-skips-merge-tier) test_dispatch_skipping_the_merge_tier_is_rejected ;;
  merge-only-wasm-job) test_merge_only_wasm_job_is_rejected ;;
  patch-bump-accepted) test_patch_bump_of_a_pinned_action_is_accepted ;;
  major-bump) test_major_bump_of_a_pinned_action_is_rejected ;;
  mutable-major-tag) test_mutable_major_tag_for_a_tool_action_is_rejected ;;
  expression-merge-guard) test_expression_form_merge_guard_is_rejected ;;
  dropped-native-component) test_dropped_native_merge_component_is_rejected ;;
  untimed-job) test_untimed_workflow_job_is_rejected ;;
  unscheduled-advisories) test_advisory_audit_without_a_schedule_is_rejected ;;
  advisories-without-audit) test_advisory_workflow_without_the_audit_is_rejected ;;
  automatically-triggered-miri) test_automatically_triggered_miri_workflow_is_rejected ;;
  miri-called-by-another-task) test_miri_called_by_another_task_is_rejected ;;
  memory-series-second-binary) test_memory_series_measured_by_a_second_binary_is_rejected ;;
  memory-series-jq) test_memory_series_assembled_with_jq_is_rejected ;;
  contract-assertions-read-whole-input) test_contract_assertions_read_their_whole_input ;;
  unpinned-workflow-linter) test_unpinned_workflow_linter_is_rejected ;;
  workflow-linting) test_pull_request_tier_without_workflow_linting_is_rejected ;;
  unwatched-rust-toolchain) test_unwatched_rust_toolchain_is_rejected ;;
  feature-off-doctests) test_pull_request_tier_without_feature_off_doctests_is_rejected ;;
  optional-persistence-default) test_optional_persistence_default_is_rejected ;;
  prohibited-action) test_prohibited_action_main_ref_is_rejected ;;
  ungated-rust-cache) test_ungated_rust_cache_save_is_rejected ;;
  ungated-mise-cache) test_ungated_mise_cache_save_is_rejected ;;
  unshared-bench-key) test_unshared_bench_cache_key_is_rejected ;;
  stale-bench-pin) test_stale_bench_action_pin_is_rejected ;;
  invalid-fixture) test_invalid_fresh_fixture_is_rejected ;;
  misplaced-persistence) test_persistence_command_in_wrong_task_is_rejected ;;
  fixture-cleanup) test_fixture_cleanup_removes_tmp_dirs_on_failure ;;
  folded-merge-guard) test_folded_merge_guard_is_rejected ;;
  spaced-merge-guard) test_spaced_key_merge_guard_is_rejected ;;
  all)
    test_invalid_fresh_fixture_is_rejected
    test_commented_requirement_is_rejected
    test_non_optional_midir_is_rejected
    test_shipped_midir_dependency_is_rejected
    test_native_midi_off_by_default_is_rejected
    test_console_without_native_midi_is_rejected
    test_pull_request_tier_without_the_disabled_feature_is_rejected
    test_audit_without_the_disabled_tree_check_is_rejected
    test_unlocked_check_deny_is_rejected
    test_unbenchmarked_orcvs_is_rejected
    test_lang_only_bench_task_is_rejected
    test_missing_orcvs_criterion_is_rejected
    test_shipped_orcvs_criterion_is_rejected
    test_bench_without_native_dependencies_is_rejected
    test_unlocked_audit_deny_is_rejected
    test_unlocked_wasm_pack_is_rejected
    test_wasm_build_without_the_feature_off_arm_is_rejected
    test_missing_default_wasm_build_is_rejected
    test_wasm_test_without_persistence_is_rejected
    test_stale_wasm_artifact_name_is_rejected
    test_hashed_wasm_artifacts_are_rejected
    test_stale_script_artifact_name_is_rejected
    test_service_worker_without_cache_invalidation_is_rejected
    test_service_worker_deleting_unrelated_caches_is_rejected
    test_service_worker_without_legacy_cache_cleanup_is_rejected
    test_service_worker_caching_error_responses_is_rejected
    test_service_worker_discarding_live_response_on_cache_failure_is_rejected
    test_service_worker_without_explicit_offline_error_is_rejected
    test_service_worker_without_immediate_activation_is_rejected
    test_service_worker_without_immediate_control_is_rejected
    test_service_worker_with_cache_first_navigation_is_rejected
    test_service_worker_with_cache_first_stable_artifacts_is_rejected
    test_service_worker_using_the_default_http_cache_is_rejected
    test_stale_debug_package_is_rejected
    test_stale_benchmark_is_rejected
    test_console_debug_package_is_rejected
    test_missing_orcvs_persistence_check_is_rejected
    test_persistence_command_in_wrong_task_is_rejected
    test_dotted_dependency_version_is_rejected
    test_dependency_table_version_is_rejected
    test_commented_dependency_table_version_is_rejected
    test_prohibited_action_main_ref_is_rejected
    test_ungated_rust_cache_save_is_rejected
    test_ungated_mise_cache_save_is_rejected
    test_unshared_bench_cache_key_is_rejected
    test_stale_bench_action_pin_is_rejected
    test_shared_push_concurrency_group_is_rejected
    test_unconditional_cancellation_is_rejected
    test_dispatch_skipping_the_merge_tier_is_rejected
    test_merge_only_wasm_job_is_rejected
    test_patch_bump_of_a_pinned_action_is_accepted
    test_major_bump_of_a_pinned_action_is_rejected
    test_mutable_major_tag_for_a_tool_action_is_rejected
    test_expression_form_merge_guard_is_rejected
    test_folded_merge_guard_is_rejected
    test_spaced_key_merge_guard_is_rejected
    test_dropped_native_merge_component_is_rejected
    test_untimed_workflow_job_is_rejected
    test_advisory_audit_without_a_schedule_is_rejected
    test_advisory_workflow_without_the_audit_is_rejected
    test_automatically_triggered_miri_workflow_is_rejected
    test_miri_called_by_another_task_is_rejected
    test_memory_series_measured_by_a_second_binary_is_rejected
    test_memory_series_assembled_with_jq_is_rejected
    test_contract_assertions_read_their_whole_input
    test_unpinned_workflow_linter_is_rejected
    test_pull_request_tier_without_workflow_linting_is_rejected
    test_unwatched_rust_toolchain_is_rejected
    test_pull_request_tier_without_feature_off_doctests_is_rejected
    test_optional_persistence_default_is_rejected
    test_fixture_cleanup_removes_tmp_dirs_on_failure
    ;;
  *) echo "unknown test: $1" >&2; exit 2 ;;
esac
