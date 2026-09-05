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
  cp "$repo_root/.github/workflows/test.yml" "$repo_root/.github/workflows/bench.yml" "$repo_root/.github/workflows/advisories.yml" "$fixture_dir/.github/workflows/"
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
  perl -pi -e 's/wasm-pack test --headless --firefox shell --test wasm --features persistence --locked/wasm-pack test --headless --firefox shell --test wasm --features persistence/' "$fixture_dir/mise.toml"
  assert_rejected "an unlocked wasm-pack test invocation"
}

test_wasm_build_without_persistence_is_rejected() {
  make_fixture
  perl -pi -e 's/trunk build --features persistence --locked/trunk build --locked/' "$fixture_dir/mise.toml"
  assert_rejected "a WASM build without the persistence feature"
}

test_missing_default_wasm_build_is_rejected() {
  make_fixture
  perl -pi -e 's/^env -u NO_COLOR trunk build --locked\n$//' "$fixture_dir/mise.toml"
  assert_rejected "a missing default-feature WASM build"
}

test_wasm_test_without_persistence_is_rejected() {
  make_fixture
  perl -pi -e 's/wasm-pack test --headless --firefox shell --test wasm --features persistence --locked/wasm-pack test --headless --firefox shell --test wasm --locked/' "$fixture_dir/mise.toml"
  assert_rejected "browser tests without the persistence feature"
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

test_pull_request_tier_without_persistence_doctests_is_rejected() {
  make_fixture
  perl -0pi -e 's/cargo test --workspace --doc --locked\ncargo test --workspace --doc --features persistence --locked\n/cargo test --workspace --doc --locked\n/' "$fixture_dir/mise.toml"
  assert_rejected "a pull-request tier that never compiles the persistence doctests"
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

test_fixture_cleanup_removes_tmp_dirs_on_failure() {
  local tmp_root before after leaked_dirs leaked
  tmp_root="$(dirname "$(mktemp -u)")"
  before="$(find "$tmp_root" -mindepth 1 -maxdepth 1 -name 'tmp.*' 2>/dev/null | sort)"
  # invalid-fixture drives make_fixture down its early-failure path (an invalid
  # CHECKER_SOURCE), which is exactly the case where a leaked fixture dir would
  # otherwise survive the run.
  bash "$0" invalid-fixture >/dev/null 2>&1 || true
  after="$(find "$tmp_root" -mindepth 1 -maxdepth 1 -name 'tmp.*' 2>/dev/null | sort)"
  leaked_dirs="$(comm -13 <(printf '%s\n' "$before") <(printf '%s\n' "$after"))"
  if [ -n "$leaked_dirs" ]; then
    echo "expected no fixture directories to remain in $tmp_root after a failed scenario, found:" >&2
    printf '%s\n' "$leaked_dirs" >&2
    while IFS= read -r leaked; do
      [ -n "$leaked" ] && rm -rf "$leaked"
    done <<< "$leaked_dirs"
    return 1
  fi
}

case "${1:-all}" in
  comments) test_commented_requirement_is_rejected ;;
  unbenchmarked-orcvs) test_unbenchmarked_orcvs_is_rejected ;;
  lang-only-bench) test_lang_only_bench_task_is_rejected ;;
  missing-orcvs-criterion) test_missing_orcvs_criterion_is_rejected ;;
  shipped-orcvs-criterion) test_shipped_orcvs_criterion_is_rejected ;;
  bench-native-dependencies) test_bench_without_native_dependencies_is_rejected ;;
  unlocked-check-deny) test_unlocked_check_deny_is_rejected ;;
  unlocked-audit-deny) test_unlocked_audit_deny_is_rejected ;;
  unlocked-wasm-pack) test_unlocked_wasm_pack_is_rejected ;;
  wasm-build-persistence) test_wasm_build_without_persistence_is_rejected ;;
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
  unpinned-workflow-linter) test_unpinned_workflow_linter_is_rejected ;;
  workflow-linting) test_pull_request_tier_without_workflow_linting_is_rejected ;;
  unwatched-rust-toolchain) test_unwatched_rust_toolchain_is_rejected ;;
  persistence-doctests) test_pull_request_tier_without_persistence_doctests_is_rejected ;;
  prohibited-action) test_prohibited_action_main_ref_is_rejected ;;
  invalid-fixture) test_invalid_fresh_fixture_is_rejected ;;
  misplaced-persistence) test_persistence_command_in_wrong_task_is_rejected ;;
  fixture-cleanup) test_fixture_cleanup_removes_tmp_dirs_on_failure ;;
  all)
    test_invalid_fresh_fixture_is_rejected
    test_commented_requirement_is_rejected
    test_unlocked_check_deny_is_rejected
    test_unbenchmarked_orcvs_is_rejected
    test_lang_only_bench_task_is_rejected
    test_missing_orcvs_criterion_is_rejected
    test_shipped_orcvs_criterion_is_rejected
    test_bench_without_native_dependencies_is_rejected
    test_unlocked_audit_deny_is_rejected
    test_unlocked_wasm_pack_is_rejected
    test_wasm_build_without_persistence_is_rejected
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
    test_shared_push_concurrency_group_is_rejected
    test_unconditional_cancellation_is_rejected
    test_dispatch_skipping_the_merge_tier_is_rejected
    test_merge_only_wasm_job_is_rejected
    test_patch_bump_of_a_pinned_action_is_accepted
    test_major_bump_of_a_pinned_action_is_rejected
    test_mutable_major_tag_for_a_tool_action_is_rejected
    test_expression_form_merge_guard_is_rejected
    test_dropped_native_merge_component_is_rejected
    test_untimed_workflow_job_is_rejected
    test_advisory_audit_without_a_schedule_is_rejected
    test_advisory_workflow_without_the_audit_is_rejected
    test_unpinned_workflow_linter_is_rejected
    test_pull_request_tier_without_workflow_linting_is_rejected
    test_unwatched_rust_toolchain_is_rejected
    test_pull_request_tier_without_persistence_doctests_is_rejected
    test_fixture_cleanup_removes_tmp_dirs_on_failure
    ;;
  *) echo "unknown test: $1" >&2; exit 2 ;;
esac
