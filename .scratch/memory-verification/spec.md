# Verify how much memory Orcvs uses

**Goal:** Make allocation behaviour and retained-state growth checkable in the gates that already run, so a regression on a per-Tick path or a leak in a long-running Console fails a pull request instead of being noticed by a user.

## Why

`docs/research/memory-testing-in-ci.md` records the survey behind this effort. Its finding is that the phrase "memory testing" covers two different things and that only one of them pays off here.

The safety half — Miri, AddressSanitizer, LeakSanitizer — exists to find bugs in `unsafe`. This workspace holds exactly one `unsafe` block, an in-place ASCII byte write with a stated invariant, already covered by the `undocumented_unsafe_blocks` and `unsafe_op_in_unsafe_fn` denials that `verification-gaps/12` settled on. Every tool in that half is nightly-only, and LeakSanitizer does not support `aarch64-apple-darwin`, which is what the macOS runner is. The yield is close to nothing.

The behaviour half — allocation counts, retained-state growth — runs on stable and targets what this repository actually is. A Tick runs on a musical clock. A Render Frame re-reads the Source many times a second. The Console is an application a user leaves open for hours. Those are the three shapes where an accidental allocation and an unbounded map hide, and the criterion suite finds neither: wall clock hides an allocation behind a cache hit, and a leak does not slow anything down until it has been running far longer than a benchmark runs.

Nothing in the repository measures either today.

## Rules

**Assert shapes, not numbers.** An absolute allocation count rots on a compiler release or a dependency bump, and the fix is always to edit the number, which teaches everyone to edit the number. Assert zero, assert independence from input size, or assert that a delta over one span of frames matches the delta over the next. Where a number genuinely must be watched, watch it in the published series with a threshold, not in an assertion.

**A failing assertion is a finding, not a number to relax.** If a path that should not allocate does allocate, record what it does and why in the issue before changing the assertion. Weakening an assertion to make it pass is the one outcome this effort exists to prevent.

**The counter lives in a test binary.** An integration test is its own binary, so a `#[global_allocator]` declared there cannot reach the shipped Console or any other test target. No feature gate is needed, which keeps this out of the feature combinations the contract asks changes to exercise.

**Ordinary tests, existing gates.** `cargo nextest run --workspace --profile ci --locked` already builds and runs every integration test on both runners on every pull request. Issues 01 through 03 add no mise task, no workflow, and no line for `scripts/check-tooling-contract.sh` to pin.

## Tooling

The counting allocator is written here rather than taken from a crate. `dhat` is the standard choice and its testing mode is the right shape, but 0.3.3 dates from February 2024, it carries an explicit "experimental… maintenance is not a high priority of the author" warning, and it pulls eight crates. `allocation-counter` has no runtime dependencies and exactly the right API, and has had no release since September 2023. The mechanism both implement is a `GlobalAlloc` that forwards to `System` and increments a counter: around forty lines.

That trade is a dependency swapped for one `unsafe impl GlobalAlloc` whose whole invariant is "forwards to `System` unchanged". It is the reason the contract's dependency rationale is recorded as the absence of a dependency, and it is why `01` carries the SAFETY comment as an acceptance criterion rather than leaving it to review.

`dhat` remains the right tool for *diagnosing* a failed assertion, because its per-callsite attribution says which allocation appeared. Add it to a branch for that and do not commit it.

Counting is thread-local, following `allocation-counter`'s design, with `const`-initialised thread-local storage so the counter cannot allocate from inside the allocation it is counting. That makes these tests correct even under a bare `cargo test`, where tests share a process. Anything measured across a multi-threaded runtime needs atomics instead and is then correct only under nextest's process-per-test isolation; a test in that position says so.

`egui_kittest` is not a decision this effort makes. `console-testing` already chose it at 0.36 with the `eframe` feature and no others, recorded why `wgpu` and `snapshot` are refused against a `deny.toml` graph that runs `all-features` across five targets, and verified that the Console constructs under `build_eframe` with no refactor. Issue `03` here inherits that harness rather than reopening any of it.

## Order

`01` first: it writes the allocator and the assertion convention that `02` and `03` follow, and it is the smallest place to discover whether the SAFETY comment satisfies the workspace denials.

`04` and `06` are independent of everything and of each other. `04` needs no instrumentation at all, because wasm linear memory never shrinks. `06` is the safety half, kept deliberately non-gating.

`05` is last and is deliberately the smallest version of itself. It starts as alerts and a job summary with no failure, because a deterministic metric at a tight threshold fires on any real change and the action offers no in-repo way to accept a deliberate increase. Turn failing on once the series has enough points to show it is stable.

## Not in scope

Image comparison, which `console-testing` already ruled out and for reasons that have nothing to do with memory.

AddressSanitizer, LeakSanitizer, MemorySanitizer, `cargo-careful`, and Valgrind or `iai-callgrind`. The research doc records the reasoning per tool. In short: nightly plus `-Zbuild-std` plus Linux-only leak detection, to find classes of bug that one documented ASCII byte write does not plausibly contain, or instruction counts that duplicate a criterion series the repository already publishes. The condition that reopens this is a real FFI audio backend, or `unsafe` growing past a handful of blocks.

Making the memory series a required check. `verification-gaps/09` owns whether the benchmark workflow blocks a merge, and `05` deliberately does not interact with it.

## Issues

- `issues/01-count-allocations-on-the-tick-and-render-frame-paths.md`
- `issues/02-count-allocations-on-the-source-write-and-language-map-rebuild.md`
- `issues/03-assert-the-console-retained-state-settles.md`
- `issues/04-assert-wasm-linear-memory-settles-after-warm-up.md`
- `issues/05-publish-a-memory-series-beside-the-benchmark.md`
- `issues/06-run-miri-deliberately-against-the-source-model.md`
