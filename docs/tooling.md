# Build and verification tooling

`rust-toolchain.toml` is the source of truth for the Rust toolchain and compilation targets.
`mise.toml` pins the additional tooling and defines verification commands used both locally and in
CI, so the same checkout executes the same gates. Each task's comment in `mise.toml` explains its
own command line; this file holds what spans tasks and workflows.

Open this file to learn which gate covers a path, what a task or workflow proves, and why a job is
shaped the way it is. Sections: [Tiers](#tiers), [Markdown-only pull requests](#markdown-only-pull-requests),
[Web console and site](#web-console-and-site), [Features](#features), [Windows](#windows),
[Inspection](#inspection), [Dependencies](#dependencies), [Workflows and CI](#workflows-and-ci),
[Caches](#caches), [Advisory gates](#advisory-gates), [Benchmarks](#benchmarks), [Miri](#miri),
[Release captures](#release-captures), [Tools](#tools), [Property tests](#property-tests),
[Version pins](#version-pins), [MIDI loopback](#midi-loopback),
[Network loopback](#network-loopback),
[Worktrees](#worktrees-each-need-their-own-target-directory) and
[Flux](#flux-refinement-checking-opt-in-local-only).

## Tiers

Verification has two trigger tiers:

- `mise run check_pull_request` runs `check_docs` (the tooling contract, plus the roadmap planner
  and the requirements check, each with its test suite), the two workflow linters, the dependency
  audit, formatting, clippy with and without the default features, the `inspection` compile, a
  Windows-target lint of the shipped crates, the native tests and doctests under both feature sets,
  and rustdoc with default features and with `--features persistence`, the only place `orcvs`'s
  persistence-gated docs are generated. The tooling contract's own fixture suite is not part of it;
  see [The tooling contract](#the-tooling-contract).
  `mise run check_wasm` compiles every crate's test targets for `wasm32-unknown-unknown` and builds
  the application twice, once under each feature set. Pull requests run the first on Linux and the
  second on the WASM job. macOS runs `check_pull_request` and the MIDI loopback only during merge
  queue verification, pushes to `main`, and manual dispatch, and Windows runs the network loopback
  on the same events.
- `mise run check_merge` runs the native merge tier (`check_merge_native`: the persistence tier at
  proptest's full case count, a compile of the release captures, and `cargo deny`) and the browser
  regression suite; run locally, it also runs `check_wasm`. CI runs the native component on the
  Linux job and the browser suite on the WASM job, on `merge_group`, after a push to `main`, or on
  manual dispatch.

`mise run check` runs both tiers locally. The merge queue runs each tier once. In the queue the
Linux job runs only the native merge tier, the WASM job only the browser suite, and macOS
`check_pull_request` and the MIDI loopback. The Linux pull-request tier and `check_wasm` therefore
ran on the pull request's own merge with `main` as it stood then, not on the queued candidate. A
conflict between two queued pull requests that only those runs would catch reaches `main`, and the
push run, which runs everything, is where it fails. The queue's Linux merge tier still compiles and
tests the default feature set on Linux, and its browser suite compiles the console for `wasm32`, so
what slips through is narrower: a Linux-only break under `--no-default-features`, a `wasm32` clippy
lint, or a break in the `--no-default-features` Trunk build. Development happens on macOS, so its clean-runner verification waits
for the queue. Push runs retain full verification and warm the caches that pull-request and queue
runs restore, and a failed push run still requires repair before normal development continues.

What the queue defers is behaviour rather than compilation: a browser test that no longer compiles
fails the pull request that wrote it. The same holds one feature over — the persistence tests live
in a test-only module behind a dev-dependency, so no library build can reach them, and the
pull-request tier reaches them by building all targets. Doctests follow the same rule: a doctest
that one feature set compiles and the other does not is compiled by only one of them, so the tier
runs `cargo test --doc` under both. Rustdoc is gated too, because doctests execute examples and do
not generate the docs, so a public item that links to a private one is invisible until `cargo doc`
runs with warnings denied. Its pair is not the feature-set pair: it is default features and
`--features persistence`, so no rustdoc pass runs `--no-default-features`. Each rustdoc pass costs about a second on a warm runner after clippy,
which is why it sits in the pull-request tier.

## Markdown-only pull requests

A pull request whose diff is Markdown and `site/` alone does not compile. The `changes` job asks
`scripts/ci-compile-needed.sh`, which answers `false` only for a pull request whose every changed
path ends in `.md` or lies under `site/`, counting both sides of a rename; the Linux job then runs
`mise run check_docs` — the tooling contract, the roadmap planner and the requirements check, the
only gates that read Markdown — and the WASM job is skipped. `check_pull_request` runs `check_docs`
as its first step, so a Markdown-reading gate is added to that one task and both paths run it.
`scripts/check-ci-results.sh` accepts that skip only on a pull request. No crate compiles or reads a
`.md` file, which is the whole of the rule: `.scratch/` is not exempt as a directory, because a test
includes a `.toml` from it. The merge queue, pushes to `main` and manual dispatch always compile.

## Web console and site

The web console and the site build separately, and GitHub Actions deploys neither.
`.github/workflows/console-web.yml` runs on a push to any branch except `gh-pages` and the merge
queue's that changes what the console is built from: the console, the crates under it, the Built-in
Synth, the shipped examples and the toolchain and tool pins. Its first job takes the console key
from `scripts/console-key.sh`, one hash over the paths in the workflow's own path filter, so the
filter covers every workspace member, every file a crate embeds from outside itself and every script
a Trunk hook runs, or two different consoles share a key; `site/scripts/tests/console.sh` fails when
one is missing. When an
archive of that key is already on the public `console-web` release the run builds nothing. Otherwise it runs `mise run console_web_build`: one
release Trunk build with relative URLs into `target/console-web/`, kept as the `console-web`
artifact. Trunk's post-build hook, `scripts/stage-synth.sh`, builds the `synth` crate's WebAssembly
module and stages it beside the console with `synth/web/`'s AudioWorklet and page scripts (ADR 0087).
The build is published on the release as `console-web-<key>.tar.gz` from `main` and as
`console-preview-<key>.tar.gz` from any other branch, except Dependabot's read-only ones, and an
archive is never replaced. On `main` the run then copies `main`'s keyed archive to the rolling
`console-web.tar.gz` and keeps the newest 100 of `main`'s archives and 50 of the branches'
(ADR 0101). A branch that changes nothing in the console takes `main`'s archive of its key.
Builds of one archive share a concurrency group, and GitHub keeps only the newest pending run in a
group, so a run can be replaced before it starts, against ADR 0101's "no push cancels another's run";
the run that replaced it publishes the same archive
([orcvs-site/30](../.scratch/orcvs-site/issues/30-let-every-pushed-console-run-finish.md)).

Cloudflare builds and deploys the site through its GitHub app (Workers Builds), with `site/` as
the root directory and its default build command, `npm run build`. The build's `prebuild` step,
`site/scripts/console.sh`, downloads the published console into `site/public/play/`, or copies
the directory `CONSOLE_WEB` names, relative to `site/`, and Astro builds the site around it into
`site/dist/`; nothing in the site build compiles the console. Every Cloudflare build waits up to 15
minutes for the console keyed to its own commit and fails without it (ADR 0093, ADR 0101):
production for `main`'s archive only, a preview for either. A local build and a `dependabot/`
branch's build take the rolling console. `site/scripts/tests/console.sh` tests the script against a stubbed
release, and the site workflow runs it. A push to `main` deploys, and a pull
request gets Cloudflare's preview URL. No deploy hook exists and GitHub holds no Cloudflare
credential; a site build that failed because its console was late is retried from Cloudflare's
dashboard. The dashboard's build watch paths must include the console's paths, or be unset, so a
console push starts a site build; they cannot be checked from the repository
([orcvs-site/26](../.scratch/orcvs-site/issues/26-build-the-site-on-console-pushes.md)).
`site/.node-version` is the Node pin Cloudflare's build
reads, and a site test holds it equal to `mise.toml`'s.

`.github/workflows/site.yml`, path-filtered to the site, the tool pins and the docs it renders,
runs `mise run site_build`, which is the same `npm ci && npm run build` in `site/`, then the
Playwright suite, and deploys nothing. It builds a pull request's head, as Cloudflare's preview
does, with the console that commit builds: the `console-web` run's build for the same commit, or
otherwise the archive under its key. `mise run site_test` serves `site/dist/` and runs the
Playwright tests in Chromium. None of these is part of `check_pull_request`, and neither workflow
is a required check.

## Features

### `persistence`

`persistence` is a default feature of `console`, so "both feature sets" means the default one and
`--no-default-features`, and every command in this file resolves the shipped application unless a
line names `--no-default-features` beside it. The shipped binary saves the current Source revision
and restores it on the next start, which is what `product-persistence/01` requires of the shipped
application: a feature that shipped off would be proved only in a configuration nobody launches,
since `mise run run`, `cargo run`, the `.vscode` cargo tasks, and a plain `trunk build` all take the
default set. `console` pulls `orcvs/persistence` in through its own feature, so `orcvs` keeps
`default = []` and the whole workspace still resolves with storage on whenever `console` is in the
build.

That makes `--no-default-features` the only arm that proves the feature-off build: nothing else in
either tier compiles the application without the storage path. Turning persistence off gives up
save and restore of the current Source revision, and `product-persistence/01`'s criterion that "a
build without the feature keeps today's behaviour and compiles" is what those runs are for. The
pull-request tier pairs `cargo clippy`, `cargo nextest run` and `cargo test --doc` over the default
set with the same three over `--no-default-features`, and `check_wasm` pairs its two `trunk build`
invocations the same way. `scripts/check-tooling-contract.sh` pins both halves, and pins
`default = ["persistence"]` in `console/Cargo.toml` beside them, because with the default flipped
the pair collapses into two feature-off runs and nothing compiles the storage path at all.

`check_pull_request` also lints `orcvs` with `--no-default-features --features persistence` and runs
its tests with `--no-default-features`; between them and the two workspace passes beside them, all
four cells of the two features are compiled, and the tests that state what disabling persistence
gives up — compiled only with it off — are executed rather than merely type-checked.

The browser suite and `mise run test_persistence` take no feature-flag decision from this; each
task's comment says why. `test_persistence` runs in the merge tier, and its overlap with the
pull-request tier is deliberate: `check_pull_request` sets `PROPTEST_CASES` to 32, so the merge tier
is the only place the properties run at proptest's 256-case default. What is genuinely merge-only
is the browser run and that full-case run. Its `nextest` line's `-E 'not kind(bench)'` is
load-bearing; the comment above that line in `mise.toml` records why.

### Off-by-default features

Two `console` features are off by default and enabled by nothing else, so no ordinary gate compiles
them and each has a task that compiles it by name: `inspection`, through `mise run check_inspection`
in the pull-request tier (see [Inspection](#inspection)), and `release-capture`, through
`mise run check_release_capture` in the merge tier (see [Release captures](#release-captures)). The
third off-by-default feature, `test-source-shapes`, needs no task: the crate's own dev-dependency
enables it in every `--all-targets` run. `scripts/check-tooling-contract.sh`
pins each compile line and its caller, because deleting one is invisible — no test fails when a
feature stops being compiled.

## Windows

No CI job builds the console for Windows, so `mise run check_windows` is the one place that
compiles it there, from the runners the tier already has: Linux on pull requests, and macOS in the
merge queue and on pushes. The task's comment gives the command, its scope and why clippy needs no
MSVC linker. `rust-toolchain.toml` declares the target so rustup installs it with the pinned
channel, the way the WASM job gets `wasm32-unknown-unknown`. It proves that Windows compiles, not
that it runs; the merge tier's `windows` job runs only the network loopback below, and
`docs/research/windows-support.md` covers what running the rest would take.
`scripts/check-tooling-contract.sh` pins the task, its call in `check_pull_request`, and the
toolchain target.

## Inspection

`inspection` forwards to `eframe/inspection`, which pulls in `egui_inspection` and attaches its
plugin at start (`eframe-0.37.0/src/lib.rs:214-222`). It is development tooling rather than product:
what it offers is an external process reading the console's AccessKit tree, injecting input,
resizing the window and taking screenshots, which is full control of the running application with no
authentication in the protocol at all. That is why it ships off, and why the feature alone is not
enough to turn it on — `egui_inspection` binds nothing unless `EGUI_INSPECTION` is set, so a build
carrying the feature and launched without the variable behaves exactly like a build without it
(`egui_inspection-0.37.0/src/lib.rs:20-51`). `mise run check_inspection` compiles it; the task's
comment says why one line on one crate is enough.

`mise run inspect` is the launcher; its comment explains the literal loopback address, the
disposable `HOME` and `XDG_DATA_HOME`, and why it launches the binary the build reports. The
contract counts each `EGUI_INSPECTION` assignment in `mise.toml` and requires every one of them to
name the loopback host, so a bare `1` fails it as repository policy rather than because that
spelling is unsafe.

**The storage redirect works on macOS and Linux only.** Those are the two platforms eframe resolves
its storage directory from the environment on. On Windows it calls
`SHGetKnownFolderPath(FOLDERID_RoamingAppData)` instead
(`eframe-0.37.0/src/native/file_storage.rs:38,46-90`), which is a shell known-folder lookup and not
an environment variable at all, so neither `HOME`, `XDG_DATA_HOME` nor `%APPDATA%` moves it. A
`mise run inspect` on Windows writes the developer's real `app.ron` under
`%APPDATA%\Orcvs\data`, and the contract assertion above cannot tell — it reads the assignment, not
the platform. Development and both CI tiers run on macOS and Linux, so this is a stated limitation
rather than a guard: on Windows, close the console before inspecting, or expect the Source to be
overwritten.

`mise run install_egui_mcp` installs the bridge the agent side attaches through, at the version its
comment names. It is the one pinned tool that `[tools]` does not hold, because it is a cargo binary
rather than a mise-managed one, so the version lives in the task and the contract pins its shape
there — including that it is not installed from a git branch. `.mcp.json` and `.codex/config.toml`
register the installed `egui-mcp` as a stdio server for repository-scoped agent configuration
(Claude and Codex); no user-global or machine-global file is written by anything in this
repository. Codex loads the project file only when the project is trusted.
`.agents/skills/egui/references/guide.md` is where the attach, verify and troubleshooting procedure
lives.

## Dependencies

Dependency auditing runs in the pull-request tier through `mise run audit_deps`, which prints the
feature-resolved dependency tree, runs `cargo deny` over advisories, bans, licences and sources, and
checks the toolkit-free tree. Dependabot's weekly grouped bumps are exactly the pull requests it
exists for, so it gates them rather than reporting on them once they are already on `main`.
`check_merge_native` runs `cargo deny` again in the merge tier; the audit is cheap, and each tier
reading correctly on its own is worth more than removing the overlap.

Platform MIDI lives in the console, not the toolkit-free crate. The console declares `midir` for
macOS, Windows, and Linux only; the browser build never sees it and reaches Web MIDI through
`web-sys` bindings instead (ADR 0059). `orcvs` carries the MIDI vocabulary and output adapter but no
platform binding — no `midir`, no `dispatch`, and no target table that could pull either in.
`audit_deps` resolves `orcvs` with default features off and fails if `midir`, `dispatch`, or a
`-sys` crate beneath them is in it, and `scripts/check-tooling-contract.sh` pins `midir` to the
console's supported-native table and keeps those bindings out of `orcvs`.

The audit denies a second version of any crate. The `[bans] skip` list in `deny.toml` is the
complete set of duplicates the graph carries on purpose, and each entry names the upstream crate
whose latest release holds the older copy. An entry names a release line (`syn@2`, `rustix@0.38`),
not an exact version, because a full version after `@` matches only that version, and a routine
`cargo update` to a new patch release would then fail the audit. When a change fails it with
`error[duplicate]`, the duplicate is new and needs a decision: unify it by changing a manifest
requirement, if the dependents allow one shared version, or add a skip entry that names the upstream
crate holding the older copy. Leave `Cargo.lock` to the resolver rather than pinning a transitive
version with `cargo update --precise`: Cargo resolves each requirement to its greatest compatible
version and does not deduplicate across incompatible ones, so the next `cargo update` undoes the pin
and the audit fails on a duplicate that is already known. When an upstream release lifts a
constraint, cargo-deny reports the entry as an unmatched skip, which is the signal to remove it.

A `[dependencies]` entry nothing uses fails the build. `[workspace.lints.cargo]` in the root
`Cargo.toml` denies `cargo::unused_dependencies`; `-D warnings` cannot, because it reaches rustc and
clippy but not Cargo's own lints. Cargo reports an entry only on a build that compiles every target
of the package that could use it — the library and its binaries together, as the `--all-targets`
and `--lib --bins` clippy runs and `trunk build` do — and never reports a dev-dependency. An entry
used on one platform or feature set fails on the others; the console's `serde` sits in the
non-wasm32 table because `theme_document` compiles only on native targets and in tests. When a
change fails with `unused dependency`, either remove the entry or move it under the
`[target.'cfg(...)'.dependencies]` table, or behind the feature, that its use is gated by.

A third trigger runs the same audit on a schedule. `cargo deny` sees the dependency graph whenever a
commit changes the graph, and an advisory is published against code nobody changed, so between two
quiet weeks a new RUSTSEC entry can land against a locked dependency with nothing in the repository
reporting it until the next pull request happens to open. `.github/workflows/advisories.yml` runs
`mise run audit_deps` weekly and on manual dispatch, so the trigger is time rather than change. It
runs the repository's own task rather than a bespoke `cargo deny` line, so the scheduled answer and
the pull-request answer are the same answer. Scheduled workflows run on the default branch only,
which is the branch the question is about.

## Workflows and CI

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
on a browser that never answers. The contract derives the expected number of bounds from the number
of jobs each file declares rather than from a literal, so a job added without one fails the contract
instead of quietly inheriting the default.

Every commit that reaches `main` gets its own merge-tier run. The test workflow's concurrency group
is keyed on the pull request number for a pull request and on the commit for anything else, so a
later merge cannot cancel an earlier commit's run, and `cancel-in-progress` is confined to pull
requests. The merge-tier steps are guarded on the event not being a pull request rather than on its
being a push, so a manual dispatch — the obvious way to re-verify a commit — runs the merge tier
instead of reporting green having run only the pull-request tier.

The `ci` aggregate is the required check, and the merge queue for `main` waits on it. It runs with
`always()` and requires Linux success on every supported event, and WASM success on every event
except a Markdown-only pull request, where WASM must be skipped. It requires macOS and Windows to be
skipped on pull requests and successful on merge groups, pushes, and manual dispatches. Failure,
cancellation, and unexpected skipping fail the aggregate. `scripts/tests/check-ci-results.sh`
exercises every combination of these job results.

`.github/workflows/code-review.yml` asks Claude's code-review routine to review a pull request when
it is opened, reopened or marked ready for review; drafts, Dependabot and fork pull requests are
skipped. It reads the `CLAUDE_ROUTINE_TOKEN` repository Actions secret and fails, naming the secret,
when the secret is empty. The review arrives as comments from the routine's session, so the job is
not a required status context and gates nothing. Pushes to an open pull request do not re-request
a review, so each review is asked for once rather than once per commit. Because the job reads a
secret, the contract holds whichever workflow names it to the `pull_request` trigger alone and to
an empty token permission set.

### The tooling contract

`scripts/check-tooling-contract.sh` and its fixture suite are gated differently. The contract runs
in `check_pull_request`: it costs under a second, and it is what fails when someone edits a pinned
line, so every pull request owes it whatever that pull request touched. Its fixture suite,
`scripts/tests/check-tooling-contract.sh`, copies a tree, breaks one line and re-runs the contract,
over a hundred times — a minute or more. What the suite can answer depends only on the files it
copies into a fixture, so `.github/workflows/tooling.yml` runs it path-filtered on exactly those,
plus the two scripts themselves, so a pull request that changes one of them meets the suite there
and one that touches none of them does not pay for it.

That filter is derived rather than trusted. The paths and the files the suite reads are the same set
stated twice, and a file added to the suite and not to the workflow would leave the gate blind to
exactly the file it had just started reading. So the contract reads both lists out of the files and
fails if any input is uncovered, and the absence of the suite from `check_pull_request` is pinned
alongside its presence in the workflow, because putting the line back is a one-word edit nothing
else notices.

## Caches

Caches are written only from `main`. GitHub scopes a cache to the ref that saved it, so an entry
written on `refs/pull/N/merge` is readable by that pull request and by nothing else, while the
repository's 10 GB quota is shared across all of them; ungated pull-request entries evict the `main`
entries every run restores from. Every `Swatinem/rust-cache` step carries `save-if` and every
`jdx/mise-action` step carries `cache_save`, both gated on the default branch. Restoring is
deliberately not gated: a pull request still reads `main`'s cache through the key prefix, so the
gate costs a pull request nothing and it saves nothing worth keeping. The two benchmark jobs
additionally share one key, because they build the same tree under the same profile and only ever
one of them runs. `scripts/check-tooling-contract.sh` counts the gates against the number of caching
steps in each workflow, so a job added with an ungated cache fails the pull-request tier.

## Advisory gates

Two gates advise rather than block. The benchmark comparison in `.github/workflows/bench.yml` fails
on a threefold regression but is not a required status context, and `main`'s branch protection does
not enforce against administrators, so a direct push skips every required context. Both stay as
they are for the first release, as
`.scratch/v1-release/issues/01-prove-the-first-release-candidate.md` records; its benchmark series
and floor review stand in for enforcement. Closing either after v1 needs repository administration
rather than a change to a checkout, so
`.scratch/verification-gaps/issues/09-require-the-benchmark-and-close-the-protection-bypasses.md`
records the intended settings and holds that decision.

## Benchmarks

Benchmarks measure locally and check only in CI. `mise run bench` runs the criterion benchmarks
in `lang`, `orcvs`, `console`, and `synth` and prints them in the bencher output format. `lang` covers language
execution: parsing an Expression and interpreting it. `orcvs` covers a populated Source: reading an
unchanged revision, deriving its Render Frame, and applying an edit with the Language Map rebuild it
forces. `console` covers `Paint::derive` and `Paint::background_runs` at a fitted range and a culled
window, so work that follows the viewport reads as a flat series. Each Source path is measured over
several sizes, so whole-map work shows as growth across the series rather than hiding inside one
fixed size. `synth` covers `Synth::render` filling one render quantum with every channel idle, with
one tone at rest, and with 1, 4 and 16 channels running their effects. `.github/workflows/bench.yml` runs the same command
in two jobs and fails either when a benchmark is more than three times slower than the previous
stored result.

The measurement is reproducible from a checkout; the comparison is not, because it lives in the
action rather than in `mise.toml`, so the benchmarks are the one gate a local run cannot reproduce.
`.scratch/benchmarks/spec.md` records what the gate can and cannot detect.

That ratio gate answers one question — did this change make it worse — and has no way to answer a
second one: is it still as good as we agreed it should be. `benchmark-action/github-action-benchmark`
only ever compares a run against the previous point stored for `main`, so a deliberately accepted
regression re-baselines the series in silence, and the next pull request is judged against the
worse number with nothing left to say so. `benches/floors.toml` closes that gap beside the ratio
gate rather than inside it: it names each guarded benchmark, the figure it must not exceed, and the
runner the figure was measured on, and `scripts/check-bench-floors.ts` — run once per job in
`.github/workflows/bench.yml`, as the last step of each job — checks the same `output.txt`
against it without benchmarking anything a second time. It runs last so that no step depends on
surviving a floor breach: a step after it that ran on the job's status would be skipped, dropping
that commit's point from the memory series on `main` or the allocation comparison on a pull
request. A benchmark
absent from the file is unguarded and the check passes it without comment; raising a guarded
figure is a plain, reviewed edit to that file rather than something the check can do on its own.
`execute_function` carries the Interpreter's floor, over the one call a Turn makes; the file
records how its starting figure was chosen. Both bench jobs run
`mise-action` with `install: false` to stay off mise's slower cargo tools, so each installs `node`
by name — `mise install node` — immediately before the one step that needs it, rather than flipping
that job-wide setting.

The `bench` task names criterion's measurement budget rather than taking its default, and never
passes `--quick`; its comment gives the figures and the reasons, and
`scripts/check-tooling-contract.sh` pins both bench commands whole and refuses the flag in either
file. Each job runs `mise run bench_warmup` first and discards its output, for the reason that
task's comment records. Both jobs run the same warm-up: the contamination it removes moves the mean
rather than widening the spread, so a job that skipped it would report slower numbers than the job
it is compared against, and the gate would read a regression off the difference between two jobs
rather than between two commits. A differing measurement budget between the jobs would be safe,
since that only widens variance; a differing warm-up is not.

The publishing job runs after a push to `main` and appends the result to the series on the
`gh-pages` branch. The pull-request job compares against that series and stores nothing.
Permissions are declared per job, so only the publishing job can write repository contents. Both
triggers are filtered to the paths that can move a measurement, so a change that cannot touch
`lang`, `orcvs`, `console`, or `synth` performance runs no benchmark. `mise run check` does not run
either.

Both jobs publish a second series beside the timings, and it measures allocation rather than wall
clock. `lang/tests/allocation.rs` and `orcvs/tests/allocation.rs` count the blocks and bytes a Tick
(in `orcvs`, both the Playback Engine's Tick and one planned under the Source lock), a Render Frame
re-read, a Cell write, and a Language Map rebuild ask the allocator for, and assert
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

The series still reports on a run the timing gate fails, which is the run most likely to have
moved it. Both actions read and write the one `gh-pages` branch, and the timing step's outcome
cannot say whether it failed on the alert, after fetching that branch, or before fetching it. So the
allocation steps never consult it: the measurement runs on the criterion run's outcome, each later
step on the outcome of the one before it, and directly before the memory action a step of its own
detaches HEAD and runs `git fetch origin +gh-pages:gh-pages`, so the action skips its own fetch
only once that fetch has succeeded. The fetch is forced because the pull-request job's timing
comparison commits its point onto the local branch without pushing it, and a plain fetch refuses to
rewind that. The job still fails on the ratio gate. `scripts/check-tooling-contract.sh` pins the
fetch, the order of the chain, and that no step after a ratio gate runs on the job's status.

## Miri

Miri runs nothing at all unless someone asks for it. `mise run miri` interprets the
Source model tests under Miri, and `.github/workflows/miri.yml` runs that task on
`workflow_dispatch` alone — no pull request, no push, no required status context, and no other mise
task calls it. Miri ships on nightly only while `rust-toolchain.toml` pins stable, and a gate the
toolchain cannot run would make the contract unfollowable, so the contract keeps Miri as the tool
the unsafe gate would prefer, run deliberately
([verification-gaps/12](../.scratch/verification-gaps/issues/12-name-an-unsafe-review-gate-that-runs.md)).
It stays off both tiers so that reaching for it remains a decision rather than a cost every change
pays.

The workspace's shipped code holds no `unsafe` block. What the task covers is the Source's
copy-on-write Cell write — `Source::set_source` in `orcvs/src/source/model.rs`, through
`SourceBuffer::write` in `orcvs/src/source/buffer.rs` — and the standard-library `unsafe` beneath it. The workspace's own `unsafe` lives only in the counting allocators its test builds
install — `lang/tests/allocation.rs`, `orcvs/tests/allocation.rs` and the `allocation` module of
`orcvs/src/lib.rs` — each an `unsafe impl GlobalAlloc` forwarding to `System`, and no shipped target
links any of them. The filter leaves the two integration-test allocators out; the one in `orcvs`'s
lib unit-test binary shares a binary with the selected tests, so Miri runs it, with its own allocator
standing in for `System`. The `undocumented_unsafe_blocks` and `unsafe_op_in_unsafe_fn` denials in
`[workspace.lints]` check on every clippy run that an `unsafe` block states its invariant; they
cannot check that the invariant holds. Miri can, and the moment to spend it is when an `unsafe`
block enters shipped code, or when that Cell write, or the Grid indexing that mints the index it
takes, changes.

The task's comment explains the nightly install, the test filter rather than a crate scope, and
`nextest`'s per-test interpreter context. Miri's isolation also rejects a wall-clock read, so the
crates' test `trace()` helpers log without timestamps; a subscriber that stamps the time aborts any
selected test that logs. The job's `timeout-minutes` is longer than any other job in this
repository carries, because interpretation costs roughly two orders of magnitude over native
execution and `cargo miri setup` builds a MIR standard library before any of it starts.
`scripts/check-tooling-contract.sh` pins the filter, pins that no task calls `mise run miri`, and
requires any workflow that does run it to carry `workflow_dispatch` and neither of the other two
triggers.

## Release captures

The release-capture workflow is also manual, and produces evidence rather than a verdict. The release needs to
see the exact candidate rendered — both built-in Themes, wide and tall — and a test can prove every
colour as a value but not what reaches the screen. `.github/workflows/release-captures.yml` runs
`mise run capture_native` on `workflow_dispatch` alone, given a full commit SHA: it renders the
console through the kittest harness the interaction tests use, with `egui_kittest`'s `wgpu` renderer
and Mesa's lavapipe as the GPU, and uploads four images and a `manifest.json` as one artifact. The
test behind it, `console::kittest_tests::capture`, asserts before each image that the seeded Source
(`console/tests/fixtures/release-capture.orcvs`) loaded, that the pinned mode and Theme are the ones
presented, that egui's zoom is 1.0 and that every checklist state is among the drawn Cells, so an
incomplete capture fails instead of uploading. Everything it pins enters the way it enters the
shipped console — the Source and egui memory through a native `app.ron`, the Theme selections
through `config.toml` — so no shipped code carries a capture branch. Nothing compares the images
against a baseline; a reviewer reads them against the visual checklist.

The capture is gated by a feature rather than a test filter, because what it must keep out of the
tiers is a dependency tree and not only a run. `release-capture` enables `egui_kittest/snapshot`
and `egui_kittest/wgpu` on a dev-dependency, so no shipped build can carry it. Two `mise.toml`
lines name it: `capture_native` runs it, and `check_release_capture` compiles it from
`check_merge_native` alone; the two tasks' comments say why. `scripts/check-tooling-contract.sh`
pins that count of two over every spelling cargo accepts — a feature list, `=`, `-F`,
`console/release-capture` — pins the compile task's line and its one caller, allows
`--all-features` only on the dependency audit's `cargo tree`, pins that no task calls the capture
task, and requires any workflow that runs the task, names the feature or passes `--all-features`
to carry `workflow_dispatch` and no other trigger. `cargo deny` still reads the feature's
tree, because `deny.toml` resolves with every feature on: the wgpu graph brings a second `pollster`
and a second `rustc-hash`, and `snapshot`'s image differ brings `colored` 2 under MPL-2.0, allowed
for that one crate. `mise run audit_deps` answers for all three on every pull request whether or not
anything compiles them.

## Tools

- `criterion` measures the four benchmarked paths — language execution in `lang`, populated Source
  reading, rendering, and editing in `orcvs`, Paint derivation in `console`, and the Built-in Synth's
  render block in `synth`; `benchmark-action/github-action-benchmark` stores and
  compares the results, and the same pinned action stores the allocation series beside them. The
  allocation counting itself takes no dependency at all: it is a `GlobalAlloc` forwarding to
  `System` inside each crate's `tests/allocation.rs`, which those files explain in place.
- `egui_kittest` runs the console's UI interaction tests. It enables AccessKit on the `Context` and
  drives the shipped `eframe::App` through `Harness::build_eframe`, which is what lets a test find a
  control by the label a viewer reads instead of by a position the test computed. Feature `eframe`
  and nothing else in every gate: `snapshot` has no renderer without `wgpu`
  (`egui_kittest-0.37.0/src/renderer.rs:37-46`), and pulling the wgpu tree into the dev graph — and
  a working GPU into every job that runs the console's tests — buys coverage the `Shape`-level
  assertions in `console::tests` already hold at a finer grain. No snapshot test compares against a
  baseline. The two features are on only under `console`'s `release-capture` feature, for the
  release captures described above. The dependency is confined to `console`'s non-WASM `dev-dependencies` for the reason
  `proptest` is confined to `lang` and `orcvs`': the invariants are platform-independent, and
  `check_wasm` compiles this crate's test targets for `wasm32-unknown-unknown`. Its `eframe` feature
  does activate `eframe/accesskit`, so a `cargo test` build of `eframe` carries `accesskit_winit`
  that a `cargo build` does not; the shipped graph is unchanged, which `cargo tree --package console
  --edges normal` is what answers.
- `egui_mcp` is not a dependency of this workspace at all. It is a separately installed binary that
  speaks the `egui_inspection` protocol to a console launched by `mise run inspect`, so an agent can
  query the widget tree and drive the UI. Neither verification tier uses it: a gate that needs a
  desktop, a window and a live server is not a gate.
- `proptest` generates the property tests that encode the invariants `CONTEXT.md` and the ADRs
  already state.
- `cargo-nextest` runs the native and feature-specific test suites with the repository's CI
  profile, including non-fail-fast reporting.
- `cargo-deny` audits the locked dependency graph for advisories, bans, licences, and sources.
- `actionlint` checks the workflow files for syntax and expression errors. Actions accepts an
  unrecognised key by ignoring it, so a misspelled `if:` does not fail a run — it silently changes
  which jobs execute. It is the cheapest check that reads the workflows as a language rather than as
  text.
- `zizmor` audits the same files for injection, permission, and credential findings. It overlaps
  `actionlint` deliberately: the two answer different questions about one file, and the file decides
  whether every other gate here runs.
- `trunk` builds the browser application and performs its WASM asset pipeline.
- `wasm-pack` executes the browser regression suite through `wasm-bindgen-test`.
- `node` runs `scripts/roadmap.ts` and its test suite through its own type stripping and test
  runner. The pull-request tier runs both, and it needs both: the suite drives `buildRoadmap` and
  `planRelease` over temporary fixtures and never reads `.scratch/`, so the throws that catch tracker
  drift — a dangling `Blocked by:`, a dependency cycle, an untagged release blocker — are reached
  only by running the planner against the real tree. `package.json` declares the floor the type
  stripping needs and `mise.toml` pins the version the gate runs.
- `node` also runs `scripts/check-requirements.ts` and its suite. The script holds
  `docs/requirements/` to its snapshot contract: a marker on every topic, unique rule IDs, every
  cited rule defined, relative links and anchors resolving, every ADR a rule cites listed in its
  marker, every ADR amendment heading dated, no ADR numbered or dated past a marker, and every
  Evidence file present and holding the symbols named after it. With `--since HEAD^1`, the test
  workflow's pull-request run also checks that a change to `docs/adr/` moved every marker, which only
  a diff can show. Because Evidence
  reads source files, a rename in code can fail it; such a change is not Markdown-only, so it meets
  the check through `check_pull_request`.

## Property tests

`AGENTS.md` obliges a change at the parser boundary to bring "boundary or property tests;
fuzz when exposure warrants it", and the parser is the widest input surface in the workspace because
every keystroke reaches it. That obligation is the rationale for the dependency. `proptest` is
declared once in `[workspace.dependencies]` and consumed by `lang` and `orcvs` under
`[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]` only. Every invariant these
properties encode is platform-independent logic, so running them under `wasm-pack` would add a
dependency graph to maintain and no signal; the property modules carry the same `cfg`, so a WASM
build never sees the dependency at all. Default features are off and only `std` is enabled, because
`std` is what the counterexample files are written through and the `fork`, `timeout`, and `bit-set`
features buy nothing for properties over pure logic. `scripts/check-tooling-contract.sh` pins that
confinement, so `proptest` stays out of every shipped `[dependencies]` table.

The case count follows the verification tier: `check_pull_request` sets `PROPTEST_CASES` to 32 and
the merge tier leaves proptest's 256-case default in place. The task's comment says why the setting
is task-level env.

Counterexample files are committed like source. proptest writes them to a `proptest-regressions`
directory beside each crate's `src`, one file per module, and no ignore rule excludes them. The
contract script asks `git check-ignore` about a fixed sample of counterexample paths rather than
reading `.gitignore`, which catches a broad glob or a nested ignore file as well as a literal rule;
the sample is not one path per property or per tracked file. A
counterexample that CI can see and a developer cannot reproduce is worse than no property at all, so
the shrunk input travels with the repository and the next run replays it before generating anything
new. `git ls-files '*proptest-regressions*'` lists the counterexamples tracked today.

That pin keeps git from hiding a counterexample, so one proptest writes surfaces as an untracked
file rather than being silently swallowed. It does not assert the file exists, and existence is not
assertable: a counterexample exists only for a property that has failed, so requiring these paths
would fail every green checkout. Nor can CI catch one generated locally and deleted before a
commit — the file was never tracked, so there is no deletion to see, and the branch arrives looking
exactly like one whose properties all passed. Committing a counterexample is a discipline this
repository states, not a gate it enforces.

Whether a counterexample earns its place depends on how hard its input is to rediscover, and that
varies by an order of magnitude across these properties. The `language_map` rebuild property draws
two 78-element vectors over the alphabet, a grid size and six booleans, and the `parser` properties
draw generated Source to a depth of 128; a shrunk input there may be the only way anyone reproduces
a CI failure, so commit it. A property over a space every run re-covers within a handful of cases,
such as a `u8` and a pair of small column indices, records an input proptest would find unaided,
and its counterexample may be discarded.

The same `AGENTS.md` sentence defers fuzzing to "when exposure warrants it", so no fuzzing harness is
installed.

## Version pins

Every version this repository pins is bumped by something that watches the file it lives in, except
the `[tools]` table. `.github/dependabot.yml` covers three ecosystems weekly: `cargo` for the
workspace manifests, `github-actions` for the SHA pins in the workflows, and `rust-toolchain` for
the channel in `rust-toolchain.toml`. The third is a separate ecosystem rather than a setting on the
first, because `cargo` never reads the toolchain file.

All three entries hold back any release younger than seven days with Dependabot's `cooldown`, the
floor zizmor's `dependabot-cooldown` audit accepts. A compromised or broken release is usually
yanked inside that window, and on a weekly schedule the delay costs at most one run. When an advisory
fix needs a release younger than the window, bump it by hand; `cargo deny` still checks it.

`mise.toml`'s `[tools]` table is the exception, because Dependabot has no mise ecosystem. Its pins
move only when a human edits the line.
`.scratch/verification-gaps/issues/13-bump-the-mise-tool-pins-automatically.md` records what that
costs and the options for closing it; the decision is about which bots run against the repository
and what write permission they hold, so it is not one a checkout can take.

Every pin in `[tools]` installs a release binary. CI's mise cache is keyed on this file and saved
only on `main`, so a pull request that edits `mise.toml` installs every tool afresh, and a download
keeps that to seconds. The contract rejects a `cargo:` entry, because that backend compiles the tool
from source.

Upgrade each version deliberately in its source-of-truth file, then run the gates `AGENTS.md` names
for a dependency or tooling change, including `mise run audit_deps` and the affected platform or
feature gates; the full `mise run check` belongs to CI.

## MIDI loopback

`console/src/console_midi/loopback_tests.rs` plays a Source through a running Orcvs and
cross-checks two ledgers: the messages the destination accepted, recorded at the `MidiConnection`
the Playback Engine writes to, and the messages a listener received. The listener decodes the wire
format itself rather than through the output adapter's encoder, so a wrong byte shows up as a
difference.

`midi_loopback_in_process` hands the messages to a listener in the same process. It runs in the
ordinary suite and checks the program's bytes and the cross-check, not a transport.
`the_tracker_loopback_in_process` does the same for `examples/tracker.orcvs` with a pattern of
chords and rests, and asserts each Tick's Note Ons, Note Offs and their order within the Tick.

`virtual_port` holds the same two scenarios over a virtual input port. Each test opens one, finds
it through the console's own `ConsoleMidiBackend`, and sends through the platform's MIDI service:
the ALSA sequencer on Linux, CoreMIDI on macOS. Windows has no virtual ports, so that module is not
compiled there. A host without the service cannot run those tests, so the ordinary suite ignores
them and `mise run test_midi_loopback` runs them. CI runs the task in the merge tier's `macos` job, where
CoreMIDI needs no setup; the hosted Ubuntu runner's Azure kernel ships no ALSA sequencer module, and
a container shares that kernel.

## Network loopback

`orcvs/tests/network_loopback.rs` plays a Source of `!u` and `!o` rows through a running Orcvs to
one listener that both ports name. A Clock writes the Tick into each Payload, so every datagram
names the Tick that sent it, and the test asserts each Tick's UDP datagram and OSC message arrive
once, in Tick Plan order, with nothing after the stop and no diagnostic. The listener reads OSC 1.0 with its own
decoder rather than the output adapter's encoder, so a wrong byte shows up as a difference.

A second test sends one Tick where nothing listens and the next to the listeners. Windows reports
the first datagram as `ConnectionReset` on the socket's next send, which ADR 0085 ignores; the
test asserts that send is still made and reports nothing.

Loopback sockets need no platform service, so both run in the ordinary suite. `mise run
test_network_loopback` runs them alone, which is how the merge tier's `windows` job runs them:
it is the only job that tests on Windows, and it runs on the same events as `macos`. Its mise
step sets `install: false`, since the task calls no tool `mise.toml` pins.

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

## Flux refinement checking (opt-in, local only)

[Flux](https://github.com/flux-rs/flux) checks refinement-type annotations on the Grid's index
arithmetic in `orcvs/src/grid.rs`. It proves that `Grid::index` stays below the Cell count, that the
`expect` in `Grid::position_at` cannot fire, and that Sector stepping cannot divide by zero. Flux
pins its own nightly with `rustc-dev`, builds only from source, and still crashes on other modules
of this crate. No CI tier runs it, and no gate owes it. The rest of `orcvs` and `lang` are not
annotated: their remaining Flux diagnostics are invariants the code already guards at runtime.

The annotations are spelled `#[cfg_attr(flux, flux::...)]`. That form needs no `flux-rs`
dependency, which is git-only and refused by `deny.toml`. Every other build strips them, and
`[workspace.lints.rust]` declares `cfg(flux)` so `unexpected_cfgs` stays quiet. Under Flux,
`bool::then_some` and closures passed to `Option::map` lose refinements, so annotated functions use
`if` and `?` instead.

To run it, install Flux with its `install.sh`, which provides `flux`, `cargo-flux`, `fixpoint` and
`z3`, then:

```sh
RUSTFLAGS="-L $HOME/.flux" FLUXFLAGS="-Fstd-extern-specs" cargo flux -p orcvs
```

`-Fstd-extern-specs` supplies the specs for `Option` and slices that the annotations rely on. Under
`cargo flux`, it also needs the `-L` search path to find `flux_core`. `[package.metadata.flux]` in
`orcvs/Cargo.toml` limits checking to `src/grid.rs`.
