# Tooling, dependency and release issue audit — 2026-09-29

Implementation baseline `cad296df6e0fbcb3a8b325dddc557bf52cda1404`, verified against
`git ls-remote origin refs/heads/main`. Working HEAD `c34bccb6` adds the earlier audit only.
The locked graph, actual workflows, source and assertion bodies are evidence; issue comments
are claims to check. No dependency update, repository-setting change or workflow dispatch performed.

## All 14 open issues in this partition

| Issue | Verdict | Independently checked evidence |
| --- | --- | --- |
| dependency-duplicates/02 | Keep needs-triage; repair blocker metadata | Cargo.lock:4346 resolves winit 0.30.13; deny.toml:36-51 retains the old-stack skips. Version prose in Blocked by generated nonexistent issue 30 through scripts/roadmap.ts:101. Reduced metadata to 01; upstream requirement stays in body. |
| dependency-duplicates/03 | Keep needs-triage; repair blocker metadata | Cargo.lock:61 locks accesskit_macos 0.26.3, whose dependencies include objc2 0.5.2, as does winit. Both objc2 versions remain (Cargo.lock:2103,2113); deny.toml:47-50 retains all four skips. Reduced Blocked by to 02 to remove false issues 30/31 parsed from version prose. |
| dependency-duplicates/06 | Keep needs-triage | Cargo.lock:31-60 holds accesskit_consumer 0.35, 0.36, 0.38. accesskit_winit is 0.32.2; kittest 0.4 uses consumer 0.35; hashbrown 0.16/0.17 remain. deny.toml:62-64 still skips the older copies. Dated upstream-release claims were not refreshed: this verdict is about the checked-in graph. |
| verification-gaps/09 | Keep ready-for-human; partially satisfied by recorded decision | Live protection API returns required full-gate/macos/wasm, strict=true, enforce_admins=false, approving reviews=0. Bench is absent. The first unchecked criterion permits a recorded advisory decision, and v1-release/01 already records one for v1; the remaining ticket is explicitly post-v1 policy. No settings changed or decision inferred beyond that record. |
| verification-gaps/13 | Keep ready-for-human | .github/dependabot.yml declares cargo, github-actions and rust-toolchain only; mise.toml tools still have no repository updater. docs/tooling.md:500-503 explicitly records that omission. External installed bots and current third-party product support were not inferred from repository absence. |
| verification-gaps/15 | Keep ready-for-agent | bench.yml:302 accepts manual branch dispatch; compare steps provide no ref. Actual dispatch job 108343219535 failed both comparisons with “No commit information is found in payload”; allocation collection/assembly/fetch succeeded. This verifies the failure against a real run, without accepting the issue's proposed action-input fix as tested. |
| release-captures/01 | Keep ready-for-agent | No capture workflow exists under .github/workflows. console/Cargo.toml:175-203 deliberately enables eframe only on egui_kittest, without snapshot/wgpu. No eight-image candidate manifest exists. Existing screenshots are not the requested capture pipeline. |
| release-captures/02 | Keep ready-for-agent, blocked by 01 | No capture workflow, Playwright procedure or browser-storage capture job exists. Existing console/tests/wasm.rs tests model/browser behavior, not the specified candidate images. |
| v1-release/01 | Keep ready-for-human, blocked | Candidate record is empty; candidate SHA/baseline nomination and named GO/NO-GO remain absent. Existing passing main CI is not a nominated evidence bundle. Release captures and physical observation remain explicit requirements. |
| v1-release/03 | Keep ready-for-agent, blocked | No candidate verification workflow/task exists. mise.toml and test.yml provide normal tiers; bench.yml push path filters can exclude docs-only SHAs. A fresh successful main CI run does not populate the candidate inventory-to-evidence matrix or nominate a baseline. |
| v1-release/04 | Keep ready-for-human, blocked by 03 | No candidate-bound physical MIDI record exists in the issue. Fake adapter tests cannot demonstrate hardware enumeration, audible continuity or reconnect. No physical test performed. |
| property-testing/06 | Keep ready-for-agent | AGENTS.md:7 names CONTEXT/ADRs as architecture authority, but does not state all requested property-versus-glossary adjudication rules. Existing source-of-truth wording alone does not satisfy the checklist. |
| source-comments/05 | Keep ready-for-agent | Actual lineage remains at console/Cargo.toml:85 (“once resolved”), orcvs/Cargo.toml:51 (“retired clock task”), scripts/check-tooling-contract.sh:258 (“used to live”). Scope is still real; no source comments changed. |
| v1-roadmap-wayfinding/08 | Keep ready-for-agent | Resolved inventory 01 still says Sequence/Tick/movement/Jump are missing. lang/src/atom.rs:800-846 declares their functions; corresponding behavior tests are identified in language.md. Refreshing every inventory row/test mapping remains work, not accomplished merely by confirming this issue is valid. |

## Six earlier closures independently rechecked

| Issue | Verdict | Evidence |
| --- | --- | --- |
| benchmarks/09 | Resolved supported | bench.yml:160,396 gate allocation collection on bench outcome, not timing comparison; independent fetch precedes memory comparison. Run 36135018866 job 108070907456 log explicitly reports seven alerts exceeded failure threshold 3, then allocation measure/assembly/fetch/comparison and floors succeed; job remains failure. This is the requested real ratio-failure case. |
| benchmarks/10 | Wontfix supported | No Interpreter::execute benchmark remains; benches/floors.toml:48 floors execute_function instead. Main push Benchmark run 36508316780 succeeds, including floor check. The historical variance question was not solved; its subject was removed. |
| native-midi/03 | Wontfix supported | orcvs has no native-midi feature or midir dependency. console/Cargo.toml:118 owns midir and mise.toml benchmarks console too. ALSA removal as written no longer follows from disabling an orcvs feature. |
| verification-gaps/14 | Wontfix supported for original scope | test.yml:71 excludes macos on pull_request and :79 sets 40-minute timeout. scripts/check-ci-results.sh handles non-success in the aggregate. This does not prove historical cost attribution or make ci a required context; the live protection query confirms it is not required. |
| memory-verification/04 | Resolved supported | console/tests/wasm.rs:76 actually warms 512 iterations, measures 2048, then asserts page-count equality. Freshly downloaded cad296df job 109214656628 log records the named test ok and 16 passed. This is evidence for that workload/run, not proof of absence of all leaks. |
| memory-verification/05 | Resolved supported | bench.yml measures the same allocation test binaries and publishes memory via customSmallerIsBetter, fail-on-alert=false. cad296df job 109214657253 shows measurement, assembly, fetch and publication success. Exact historical point count was not reverified or repeated as a finding. Making allocation trends fail is an explicit separate policy deferral. |

## Remote evidence and commands

All remote operations were read-only. API job summaries were cross-checked with raw logs where a
specific failure reason or individual test result mattered.

- `git ls-remote origin refs/heads/main` — passed after sandbox DNS failure and escalated retry.
- `gh api repos/orcvs/orcvs/branches/main/protection --jq '{contexts: .required_status_checks.contexts, strict: .required_status_checks.strict, admins: .enforce_admins.enabled, reviews: .required_pull_request_reviews.required_approving_review_count}'` — passed.
- `gh api repos/orcvs/orcvs/actions/runs/36508316642/jobs --paginate --jq '.jobs[] | {id,name,conclusion,steps: [.steps[] | {name,conclusion}]}'` — passed; all jobs success.
- `gh api repos/orcvs/orcvs/actions/runs/36508316780/jobs --paginate --jq '.jobs[] | {id,name,conclusion,steps: [.steps[] | {name,conclusion}]}'` — passed; publish success.
- `gh api repos/orcvs/orcvs/actions/runs/36219958326/jobs --paginate --jq '.jobs[] | {id,name,conclusion,steps: [.steps[] | {name,conclusion}]}'` — passed; dispatch comparison failures confirmed.
- `gh api repos/orcvs/orcvs/actions/runs/36135018866/jobs --jq '.jobs[] | {id,name,conclusion,steps:[.steps[] | {name,conclusion}]}'` — passed; ratio failure followed by successful allocations confirmed.
- `gh api repos/orcvs/orcvs/actions/jobs/109214656628/logs --allow-escape-sequences > /tmp/orcvs-audit-wasm.log` — passed.
- `gh api repos/orcvs/orcvs/actions/jobs/108343219535/logs --allow-escape-sequences > /tmp/orcvs-audit-dispatch.log` — passed.
- `gh api repos/orcvs/orcvs/actions/jobs/108070907456/logs --allow-escape-sequences > /tmp/orcvs-audit-ratio.log` — passed.
- Initial log retrieval failed on sandbox network access, then gh refused ANSI escape sequences; explicit escalated downloads with the formatting flag succeeded. Files were searched as text, not executed.

Raw log links: [browser](https://github.com/orcvs/orcvs/actions/runs/36508316642/job/109214656628),
[main benchmarks](https://github.com/orcvs/orcvs/actions/runs/36508316780/job/109214657253),
[dispatch failure](https://github.com/orcvs/orcvs/actions/runs/36219958326/job/108343219535),
[ratio failure](https://github.com/orcvs/orcvs/actions/runs/36135018866/job/108070907456).
