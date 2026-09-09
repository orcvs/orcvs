# Build and verification tooling

`rust-toolchain.toml` is the source of truth for the Rust toolchain and compilation targets.
`mise.toml` pins the additional tooling and defines verification commands used both locally and in
CI, so the same checkout executes the same gates.

Verification has two trigger tiers:

- `mise run check_pull_request` runs the tooling contract, the contract's own test suite, the two
  workflow linters, the roadmap planner's test suite, the dependency audit, formatting, clippy with
  and without the default features, and the native tests and doctests under both.
  `mise run check_wasm` compiles every crate's test targets for `wasm32-unknown-unknown` and builds
  the application twice, once under each. Pull requests run the first on Linux and macOS and the
  second on the WASM job.
- `mise run check_merge` runs the browser regression suite, the rustdoc gates, and the persistence
  tier at proptest's full case count. CI distributes these gates across the existing Linux and WASM
  jobs after a push to `main` or a manual dispatch.

`mise run check` runs both tiers locally. A failure in the merge tier makes `main` red and must be
fixed before normal development continues. What the delayed tier holds is behaviour rather than
compilation: a browser regression can still be found after merge, but a browser test that no longer
compiles fails the pull request that wrote it. The same is true one feature over — the persistence
tests live in a test-only module behind a dev-dependency, so no library build can reach them, and
the pull-request tier reaches them by building all targets. The doctests follow the same rule: a
doctest that one feature set compiles and the other does not is compiled by only one of them, so
the tier runs `cargo test --doc` under both rather than only the shipped one.

`persistence` is a default feature of `shell`, so "both feature sets" now means the default one and
`--no-default-features`. The shipped binary saves the current Source revision and restores it on the
next start, which is what `product-persistence/01` requires of the shipped application: a feature
that shipped off would be proved only in a configuration nobody launches, since `mise run run`,
`cargo run`, the `.vscode` cargo tasks, and a plain `trunk build` all take the default set. `shell`
pulls `orcvs/persistence` in through its own feature, so `orcvs` keeps `default = []` and the whole
workspace still resolves with storage on whenever `shell` is in the build.

That makes `--no-default-features` the arm that proves the feature-off build, and it is the arm that
proves it *only* — nothing else in either tier compiles the application without the storage path.
`product-persistence/01`'s criterion that "a build without the feature keeps today's behaviour and
compiles" is what those runs are for, so the pull-request tier pairs `cargo clippy`, `cargo nextest
run` and `cargo test --doc` over the default set with the same three over `--no-default-features`,
and `check_wasm` pairs the two `trunk build` invocations the same way. Naming `--features
persistence` on the other half of any of those pairs would name the default twice and leave one
configuration tested; `scripts/check-tooling-contract.sh` pins both halves, and pins
`default = ["persistence"]` in `shell/Cargo.toml` beside them, because with the default flipped back
the pair collapses into two feature-off runs and nothing compiles the storage path at all.

The browser suite and `mise run test_persistence` take no feature flag decision from this.
`test_wasm` runs the default configuration, which is the one a browser loads, and spending a second
headless Firefox run on a build `check_wasm` already compiles buys nothing. `test_persistence` keeps
its explicit `--features persistence`: `persistence` is not in the `orcvs` default set, so its
`cargo check --package orcvs --lib` line has no other way to reach the feature, and the task keeps
its meaning if the `shell` default ever moves.

`mise run test_persistence` still runs in the merge tier, and its overlap with the pull-request tier
is deliberate rather than an oversight: `check_pull_request` sets `PROPTEST_CASES` to 32, so the
merge tier is the only place the properties run at proptest's 256-case default. What is genuinely
merge-only is the browser run, the rustdoc gates, and that full-case run.

The tooling contract and its fixture suite are gated differently, and the split is worth stating
because it looks like an inconsistency. `scripts/check-tooling-contract.sh` runs in
`check_pull_request`: it costs under a second, and it is what fails when someone edits a pinned
line, so every pull request owes it whatever that pull request touched. Its fixture suite,
`scripts/tests/check-tooling-contract.sh`, does not run there. The suite copies a tree, breaks one
line and re-runs the contract, ninety times over — about thirty seconds — and the tier runs on both
the Linux and macOS legs, so every pull request paid it twice, including the ones that touched no
tooling at all. What the suite can answer depends only on the files it copies into a fixture, so
`.github/workflows/tooling.yml` runs it path-filtered on exactly those, plus the two scripts
themselves.

That filter is derived rather than trusted. The paths and the files the suite reads are the same set
stated twice, and a file added to the suite and not to the workflow would leave the gate blind to
exactly the file it had just started reading — which is the failure the contract exists to catch,
one layer up. So the contract reads both lists out of the files and fails if any input is
uncovered, and the absence of the suite from `check_pull_request` is pinned alongside its presence
in the workflow, because putting the line back is a one-word edit nothing else notices.

`orcvs` carries a second feature, and it is the one a tier can pass without ever building.
`native-midi` gates the platform MIDI backend and the `midir` dependency that reaches it, and it is
on by default, so every command in this file resolves exactly as it did before the feature existed
— the `--no-default-features` halves included, because `shell` names `orcvs/native-midi` for its
non-WASM targets and workspace resolution hands it back.
What turning it off gives up is delivery. A running Orcvs still composes, executes, and renders
Source, and still has a MIDI output adapter; the adapter holds a backend that offers no destination
and refuses to connect, so it accepts every submission and sends nothing. What it buys is a
dependency tree with neither `midir` nor a system audio library — ALSA on Linux, CoreMIDI on macOS —
anywhere in it. `orcvs/src/native_midi.rs` is the one place that chooses between the two answers:
the feature says whether this build wants a native backend, and the manifest's target table says
where one could exist, which is why a WASM build never sees `midir` whichever way the feature is
set.

Being on by default is exactly why the tier has to ask for the other state by name.
`check_pull_request` lints `orcvs` with `--no-default-features --features persistence` and runs its
tests with `--no-default-features`; between them and the two workspace passes beside them, which
carry the backend either way, all four cells of the two features are compiled, and the tests that
state what disabling
`native-midi` gives up — compiled only with it off — are executed rather than merely type-checked.
The scope is `orcvs` because it is the only crate the feature reaches: `shell` asks for
`orcvs/native-midi` by name for its non-WASM targets and would keep it whatever the tier passed, and
the browser build asks for no native backend at all. `mise run audit_deps` holds the tree claim
itself rather than describing it — it resolves the feature-off tree and fails if `midir` or an audio
library is still in it, and because `cargo tree` resolves the host target, the Linux and macOS
runners between them check both libraries a native build could otherwise link.
`scripts/check-tooling-contract.sh` pins all of it, down to `midir` staying optional and staying in
its target table.

Dependency auditing runs in the pull-request tier through `mise run audit_deps`, which checks
advisories, licences, and sources and prints the feature-resolved dependency tree. Dependabot's
weekly grouped bumps are exactly the pull requests it exists for, so it gates them rather than
reporting on them once they are already on `main`. `mise run check_merge_native` runs `cargo deny`
again after a merge; the audit is cheap, and each tier reading correctly on its own is worth more
than removing the overlap.

A third trigger runs the same audit on a schedule. `cargo deny` sees the dependency graph whenever a
commit changes the graph, and an advisory is published against code nobody changed, so between two
quiet weeks a new RUSTSEC entry can land against a locked dependency with nothing in the repository
reporting it until the next pull request happens to open. `.github/workflows/advisories.yml` runs
`mise run audit_deps` weekly and on manual dispatch, so the trigger is time rather than change. It
runs the repository's own task rather than a bespoke `cargo deny` line, so the scheduled answer and
the pull-request answer are the same answer. Scheduled workflows run on the default branch only,
which is the branch the question is about.

The workflows themselves are the one part of the verification surface no compiler reads, and they
are the part that decides whether the rest of it runs. `actionlint` checks their syntax and
expressions; `zizmor` audits them for injection and permission findings. Both run in the
pull-request tier beside the tooling contract, on the same reasoning: they read the repository's own
configuration, they cost seconds, and they fail before the tier spends twenty minutes compiling.
Neither reports anything against the workflows as they stand — every action is SHA-pinned and every
job declares its permissions — and that is what they are for. They hold that shape rather than
discovering it, in a file format where a typo in an `if:` key is accepted silently and a job simply
stops running.

Every job in every workflow declares `timeout-minutes`. Without one a job inherits the six-hour
runner limit, and the shape that would spend it is a `wasm-pack test --headless --firefox` waiting
on a browser that never answers; the jobs otherwise finish in about ninety seconds. The contract
derives the expected number of bounds from the number of jobs each file declares rather than from a
literal, so a job added without one fails the contract instead of quietly inheriting the default.

Every commit that reaches `main` gets its own merge-tier run. The workflow's concurrency group is
keyed on the pull request number for a pull request and on the commit for anything else, so a later
merge cannot cancel an earlier commit's run, and `cancel-in-progress` is confined to pull requests.
The merge-tier steps are guarded on the event not being a pull request rather than on its being a
push, so a manual dispatch — the obvious way to re-verify a commit — runs the merge tier instead of
reporting green having run only the pull-request tier.

Caches are written only from `main`. GitHub scopes a cache to the ref that saved it, so an entry
written on `refs/pull/N/merge` is readable by that pull request and by nothing else, while the
repository's quota is shared across all of them. Left ungated, pull-request runs filled that quota
with entries no later run could read and evicted the `main` entries every run restores from — the
repository sat at 9.8 GB of a 10 GB limit, under continuous eviction. Every `Swatinem/rust-cache`
step carries `save-if` and every `jdx/mise-action` step carries `cache_save`, both gated on the
default branch. Restoring is deliberately not gated: a pull request still reads `main`'s cache
through the key prefix, so the gate costs a pull request nothing and it saves nothing worth keeping.
The two benchmark jobs additionally share one key, because they build the same tree under the same
profile and only ever one of them runs. `scripts/check-tooling-contract.sh` counts the gates against
the number of caching steps in each workflow, so a job added with an ungated cache fails the
pull-request tier rather than quietly filling the quota again.

Two gates advise rather than block, and both are recorded here rather than assumed. The benchmark
comparison in `.github/workflows/bench.yml` fails on a threefold regression but is not a required
status context, and `main`'s branch protection does not enforce against administrators, so a direct
push skips every required context. Closing either needs repository administration rather than a
change to a checkout, so
`.scratch/verification-gaps/issues/09-require-the-benchmark-and-close-the-protection-bypasses.md`
records the intended settings and holds the decision.

A third tier measures locally and checks only in CI. `mise run bench` runs the criterion benchmarks
in both `lang` and `orcvs` and prints them in the bencher output format. `lang` covers language
execution: parsing an Expression and interpreting it. `orcvs` covers a populated Source: reading an
unchanged revision, deriving its Render Frame, and applying an edit with the Language Map rebuild it
forces. Each is measured over several Source sizes, so whole-map work shows as growth across the
series rather than hiding inside one fixed size. `.github/workflows/bench.yml` runs the same command
in two jobs and fails either when a benchmark is more than three times slower than the previous
stored result. The publishing job runs after a push to `main` and appends the result to the series
on the `gh-pages` branch. The pull-request job compares against that series and stores nothing.
Permissions are declared per job, so only the publishing job can write repository contents. Both
triggers are filtered to the paths that can move a measurement, so a change that cannot touch `lang`
or `orcvs` performance runs no benchmark. `mise run check` does not run either.

Both jobs publish a second series beside the timings, and it measures allocation rather than wall
clock. `lang/tests/allocation.rs` and `orcvs/tests/allocation.rs` count the blocks and bytes a Tick,
a Render Frame re-read, a Cell write, and a Language Map rebuild ask the allocator for, and assert
shapes over them on every `cargo nextest run --workspace`. Setting `ORCVS_MEMORY_SERIES=1` makes the
same test functions print what they just measured, and the workflow turns those records into the
`customSmallerIsBetter` JSON the same pinned action stores under the series name `memory`. The
series is fed by the asserting tests rather than by a binary of its own, so the published number and
the asserted number cannot drift apart.

Three things about that series differ from the timings beside it, and each is a decision. It is
collected with a plain `cargo test`, because both bench jobs run `mise-action` with `install: false`
and hold no cargo tool beyond the toolchain; the thread-local counters in those files are
`const`-initialised precisely so they stay correct in the one shared process a bare `cargo test`
runs every test in. It takes no warm-up run, because an allocation count is deterministic for a
fixed input and a second run would only cost the job twice. And it alerts without failing, at
`110%`/`125%` against the timings' `150%`/`300%`: the metric has no runner noise for a loose
threshold to sit above, and the action offers no in-repo way to accept a deliberate increase, so
failing waits until the series has enough points to show it is stable.
`.scratch/memory-verification/spec.md` records the effort behind it.

This benchmark gate is the one exception to the equivalence above. The measurement is reproducible
from a checkout; the comparison is not, because it lives in the action rather than in `mise.toml`.
`.scratch/benchmarks/spec.md` records what the gate can and cannot detect.

A fourth trigger runs nothing at all unless someone asks for it. `mise run miri` interprets the
Source model tests under Miri, and `.github/workflows/miri.yml` runs that task on
`workflow_dispatch` alone — no pull request, no push, no required status context, and no other mise
task calls it. That is deliberate rather than an omission.
`.scratch/verification-gaps/issues/12-name-an-unsafe-review-gate-that-runs.md` decided the contract
would stop *requiring* Miri, because it ships on nightly only while `rust-toolchain.toml` pins
stable, and a gate the toolchain cannot run makes the contract unfollowable at the one place it
matters most. What that decision kept is Miri as the tool the unsafe gate would prefer, run
deliberately. This is the path it points at, and it stays off both tiers so that reaching for it
remains a decision rather than a cost every change pays.

What it covers is the one `unsafe` block in the workspace's shipped code: the in-place ASCII
byte write in `Source::set_source`, `orcvs/src/source/model.rs`. Two more live in the counting
allocators of `lang/tests/allocation.rs` and `orcvs/tests/allocation.rs`, and the filter leaves them
out on purpose — each is an `unsafe impl GlobalAlloc` forwarding to `System`, which is the one thing
Miri replaces with its own allocator rather than interpreting, and no shipped target links either. The `undocumented_unsafe_blocks` and
`unsafe_op_in_unsafe_fn` denials in `[workspace.lints]` already check on every clippy run that the
block states an invariant; they cannot check that the invariant holds. Miri can, and the moment to
spend it is when that byte write, or the Grid indexing that mints the index it takes, changes.

The task installs `nightly` and the `miri` component itself rather than moving the pinned channel,
so nothing else in the repository becomes nightly's problem for the length of a run. It is scoped by
test filter — `-E 'test(/^source::model::test::/)'`, the 68 tests in the module that holds the block
— and not by crate. That distinction is what makes the run possible at all: `orcvs` links ALSA
through `midir` in the default-featured build the task runs, and builds a multi-threaded Tokio
runtime, and Miri can execute neither, having no foreign functions and no real threads to hand them. But Miri interprets what actually runs rather
than what the crate links, so a dependency no selected test calls never becomes a problem, where
`cargo miri nextest run --package orcvs` would meet both. The run goes through `nextest` for the
reason every gate here does, with one addition: process-per-test gives each test its own interpreter
context, so Miri's leak check at termination reports per test rather than per binary. The job's
`timeout-minutes` is longer than any other job in this repository carries, because interpretation
costs roughly two orders of magnitude over native execution and `cargo miri setup` builds a MIR
standard library before any of it starts. `scripts/check-tooling-contract.sh` pins the filter, pins
that no task calls `mise run miri`, and requires any workflow that does run it to carry
`workflow_dispatch` and neither of the other two triggers.

- `criterion` measures both benchmarked paths — language execution in `lang`, and populated Source
  reading, rendering, and editing in `orcvs`; `benchmark-action/github-action-benchmark` stores and
  compares the results, and the same pinned action stores the allocation series beside them. The
  allocation counting itself takes no dependency at all: it is a `GlobalAlloc` forwarding to
  `System` inside each crate's `tests/allocation.rs`, which those files explain in place.
- `proptest` generates the property tests that encode the invariants `CONTEXT.md` and the ADRs
  already state.
- `cargo-nextest` runs the native and feature-specific test suites with the repository's CI
  profile, including non-fail-fast reporting.
- `cargo-deny` audits the locked dependency graph for advisories, bans, licences, and sources.
- `actionlint` checks the workflow files for syntax and expression errors. Actions accepts an
  unrecognised key by ignoring it, so a misspelled `if:` does not fail a run — it silently changes
  which jobs execute, which is the failure this branch exists to close, one file over from where it
  was found. It is the cheapest check that reads the workflows as a language rather than as text.
- `zizmor` audits the same files for injection, permission, and credential findings. It overlaps
  `actionlint` deliberately: the two answer different questions about one file, and the file decides
  whether every other gate here runs. Neither has anything to report against the workflows as they
  stand, so both are regression protection for a shape already reached rather than a repair.
- `trunk` builds the browser application and performs its WASM asset pipeline.
- `wasm-pack` executes the browser regression suite through `wasm-bindgen-test`.
- `node` runs `scripts/roadmap.ts` and its test suite through its own type stripping and test
  runner. The pull-request tier runs both, and it needs both: the suite drives `buildRoadmap` and
  `planRelease` over temporary fixtures and never reads `.scratch/`, so the throws that catch tracker
  drift — a dangling `Blocked by:`, a dependency cycle, an untagged release blocker — are reached
  only by running the planner against the real tree. `package.json` declares the floor the type
  stripping needs and `mise.toml` pins the version the gate runs.

`AGENTS.md` already obliges a change at the parser boundary to bring "boundary or property tests;
fuzz when exposure warrants it", and the parser is the widest input surface in the workspace because
every keystroke reaches it. That obligation is the rationale for the dependency. `proptest` is
declared once in `[workspace.dependencies]` and consumed by `lang` and `orcvs` under
`[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]` only. Every invariant these
properties encode is platform-independent logic, so running them under `wasm-pack` would add a
dependency graph to maintain and no signal; the property modules carry the same `cfg`, so a WASM
build never sees the dependency at all. Default features are off and only `std` is enabled, because
`std` is what the counterexample files are written through and the `fork`, `timeout`, and `bit-set`
features buy nothing for properties over pure logic. `scripts/check-tooling-contract.sh` pins that
confinement, so a later change cannot quietly move `proptest` into a shipped `[dependencies]` table.

The case count follows the verification tier. `mise run check_pull_request` sets `PROPTEST_CASES` to
32, so a pull request trades coverage for latency; the merge tier leaves proptest's 256-case default
in place. The setting is task-level env rather than an inline assignment on a run line, because the
contract script pins the exact text of those lines.

Counterexample files are committed like source. proptest writes them to a `proptest-regressions`
directory beside each crate's `src`, one file per module, and no ignore rule excludes them. The
contract script pins the path for each property — `lang/proptest-regressions/parser.txt`,
`lang/proptest-regressions/interpreter.txt`, `orcvs/proptest-regressions/grid.txt`,
`orcvs/proptest-regressions/source/language_map.txt`, and
`orcvs/proptest-regressions/source/tick.txt` — by asking
`git check-ignore` rather than reading
`.gitignore`, which catches a broad glob or a nested ignore file as well as a literal rule. A
counterexample that CI can see and a developer cannot reproduce is worse than no property at all, so
the shrunk input travels with the repository and the next run replays it before generating anything
new.

The same `AGENTS.md` sentence defers fuzzing to "when exposure warrants it", so no fuzzing harness is
installed. That is a separate decision with its own cost, and it is not taken here.

Every version this repository pins is bumped by something that watches the file it lives in.
`.github/dependabot.yml` covers three ecosystems weekly: `cargo` for the workspace manifests,
`github-actions` for the SHA pins in the workflows, and `rust-toolchain` for the channel in
`rust-toolchain.toml`. The third is a separate ecosystem rather than a setting on the first —
`cargo` never reads the toolchain file — and its absence is why the channel sat at 1.98.0 while
1.98.1 was current.

`mise.toml`'s `[tools]` table is the remaining exception, and it is an exception because Dependabot
has no mise ecosystem and no near prospect of one. The seven pins there move only when a human edits
the line. `.scratch/verification-gaps/issues/13-bump-the-mise-tool-pins-automatically.md` records
what that costs and the three options for closing it; the decision is about which bots run against
the repository and what write permission they hold, so it is not one a checkout can take.

Upgrade each version deliberately in its source-of-truth file, then run `mise run check`, the
affected platform or feature gates, and `mise run audit_deps`.

## Worktrees each need their own target directory

Everything above assumes a local gate answers a question about the checkout it ran in. A
`CARGO_TARGET_DIR` or `[build] target-dir` shared across several worktrees of this repository
breaks that assumption, and the failure is silent rather than loud.

Cargo records a unit's location workspace-relatively. The fingerprint's `path` field is a hash of
the target's source path relative to the workspace root, and the companion `dep-*` file lists
sources the same way — `src/lib.rs`, `src/parser.rs`. Neither carries the checkout that produced
them. Two worktrees of one repository therefore write fingerprints that each will happily validate
against the other's sources, and when the mtimes agree cargo calls the unit fresh and hands the
wrong artefact to the next crate as `--extern`.

That is not hypothetical here. With nine worktrees sharing one target directory, a clean checkout
of `main` failed to compile `orcvs` against a `lang` carrying enum variants that existed only in
the `04-send-control-change-and-pitch-bend` worktree, while `cargo nextest run --package orcvs`
in the same checkout passed and ran five tests that this tree does not contain. A gate that green
on a binary built from another branch's source is worse than no gate. The same directory is also
one build lock, so concurrent worktrees serialise on it rather than running in parallel.

Let each worktree use its own `target/`. Share the layers that cannot alias instead: the registry
under `~/.cargo/registry`, keyed by name, version and checksum, and `sccache`, keyed by the
compiler, its arguments, and the preprocessed source.

What that recovers is the dependency graph, not the whole build. sccache's key includes the
command line, and the command line carries absolute paths, so a crate compiled under two
different worktree roots hashes to two different keys. Registry crates are immune because they
compile out of one shared `~/.cargo/registry` path whichever worktree asked for them; the
workspace's own crates are not. Building `lang` from a second worktree holding identical sources,
after the first worktree had already compiled it, hit 16 of 18 Rust compilations — every registry
dependency — and missed on the workspace-local crates. Closing that last gap needs sccache's
path-normalisation (`SCCACHE_BASEDIRS`), which is not configured here, so treat cross-worktree
reuse of this repository's own crates as best effort and the dependency graph as the real saving.

None of it works while incremental compilation is on: sccache declines an incremental unit, and
the dev profile enables one by default. `incremental = false` is what lets sccache see the
compilation at all, and it drops `target/debug/incremental/` besides, which pays back part of
what per-worktree directories cost in disk. The price is paid on the other side — repeatedly
rebuilding one crate you are editing is slower without incremental state — so it is a trade that
suits many worktrees and parallel agents, not a default that suits every checkout.
