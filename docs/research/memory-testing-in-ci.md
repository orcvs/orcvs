# Memory tests and analysis in CI

Research date: 2026-09-09. This is a tooling investigation covering `lang`, `orcvs`, and the
`shell` egui application against the repository's existing two-tier CI. It proposes gates and
records why the rejected tools were rejected; nothing here is adopted yet.

## Recommendation

**Treat allocation behaviour as ordinary tests, not as a separate memory-analysis job.**

The canonical Rust answer splits along two axes that get conflated under the phrase "memory
testing":

- **Memory safety / undefined behaviour** — Miri, AddressSanitizer, LeakSanitizer. Every one of
  these is nightly-only, and every one of them exists to find bugs in `unsafe`.
- **Memory behaviour** — allocation counts, peak heap, steady-state growth. These run on stable,
  and they find bugs the type system cannot: an allocation on a per-Tick path, retained state
  that never stops growing, a cache with no eviction.

This workspace holds exactly one `unsafe` block: an in-place ASCII byte write in
`orcvs/src/source/model.rs` with a stated invariant, already covered by the `undocumented_unsafe_blocks`
and `unsafe_op_in_unsafe_fn` denials that `.scratch/verification-gaps/issues/12` settled on. The
safety axis has close to no yield here. The behaviour axis has a great deal: `lang` sits on a path a
Tick runs on a musical clock, `orcvs` re-reads the Source for every Render Frame, and `shell` is an
egui application a user leaves open for hours. Those are exactly the three shapes where allocation
regressions and unbounded retention hide, and the criterion suite measures them only indirectly —
wall clock hides an allocation behind a cache hit.

So: put allocation assertions in the pull-request tier as plain `#[test]` functions, put a growth
assertion on the egui frame loop, publish the numbers alongside the existing criterion series, and
keep Miri as a deliberate non-gating nightly run scoped to the one `unsafe` block.

**Write the counting allocator rather than depending on one.** Both candidate crates are stale, and
one of them ships a maintenance disclaimer. The mechanism is a `GlobalAlloc` that forwards to
`System` and increments a counter — about forty lines, installed in an integration-test binary
where nothing shipped can see it. See "Why not `dhat` or `allocation-counter`" below.

## Candidates and their fit

| Candidate | What the primary source establishes | Fit here |
| --- | --- | --- |
| **Counting `GlobalAlloc` in a test binary** | `GlobalAlloc` is stable and `System` is the default backing allocator. A `#[global_allocator]` declared in an integration test applies to that test binary alone, because each integration test is its own binary. | **Adopt.** No dependency, no `cargo deny` question, no wasm build problem, no version to keep current. Costs one `unsafe impl GlobalAlloc` with a SAFETY comment — see the trade below. |
| **`dhat` (testing mode)** | Provides `ProfilerBuilder::testing()`, `HeapStats::get()`, and `dhat::assert_eq!` over `total_blocks`, `total_bytes`, `max_bytes`, `curr_bytes`, writing a viewable profile when an assertion fails. Requires `dhat::Alloc` as the global allocator and panics if two profilers run at once. [Documentation](https://docs.rs/dhat/latest/dhat/) | **Escalation only.** Its per-callsite attribution is the right tool for *diagnosing* a failed assertion. Add it to a branch for that, do not commit it: 0.3.3 dates from February 2024, it carries an explicit "experimental… maintenance is not a high priority" warning, and it pulls eight crates including a `rustc-hash 1.1` the lockfile already carries twice. |
| **`allocation-counter`** | `measure(closure)` returns `AllocationInfo` with `count_total`, `count_current`, `count_max`, `bytes_total`, `bytes_current`, `bytes_max`, counted in a thread-local, with no runtime dependencies. [Documentation](https://docs.rs/allocation-counter) | **Reject, but copy the design.** Zero-dependency and exactly the right API shape, but 0.8.1 was released September 2023 and nothing has followed. Its thread-local counting is the detail worth reproducing (see "Thread-local versus atomic" below). |
| **`egui_kittest`** | `HarnessBuilder::build_eframe` constructs a harness over an `eframe::App` behind the `eframe` feature; `step()` advances one frame; GPU and rendering are required only for the `snapshot`/`wgpu` features. Version 0.36.2 tracks the `egui` 0.36 line this workspace pins. [Documentation](https://docs.rs/egui_kittest/latest/egui_kittest/) | **Adopt for `shell`.** It is the only way to drive `Console` frame by frame without a window, and non-snapshot mode needs no GPU, so it runs on the existing ubuntu and macOS runners unchanged. `accesskit 0.24.1` is already in the lockfile via eframe, so the marginal dependency cost is `kittest` itself. |
| **Miri** | Detects out-of-bounds access, use-after-free, invalid initialisation, alignment and aliasing violations, data races, and leaks at termination. Nightly-only via `rustup +nightly component add miri`. Cannot execute foreign functions or syscalls; networking, threading APIs, and file I/O are unsupported or limited. [Repository](https://github.com/rust-lang/miri) | **Adopt as non-gating.** `cargo miri nextest run` is supported and gives each test its own Miri context. Scope it by test filter to the Source model, not by crate: `orcvs` links `midir` (ALSA FFI) and a multi-threaded tokio runtime, neither of which Miri can execute, but a test that never calls them never makes Miri try. |
| **AddressSanitizer / LeakSanitizer** | Nightly-only `-Zsanitizer=`, strongly recommended with `-Zbuild-std --target`. LeakSanitizer supports `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, and `x86_64-apple-darwin` — not `aarch64-apple-darwin`. [Unstable book](https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/sanitizer.html) | **Reject for now.** Nightly plus `-Zbuild-std` plus Linux-only leak detection (the macOS runners are arm64), to find classes of bug that one documented ASCII byte write does not plausibly contain. The condition that changes this answer: a real FFI audio backend, or `unsafe` growing past a handful of blocks. |
| **`iai-callgrind` / Valgrind** | Wraps Callgrind, DHAT, Massif, Memcheck, Helgrind, and DRD; runs on stable; deterministic enough for CI; Linux i686/x86_64, with Valgrind installed. [Documentation](https://docs.rs/iai-callgrind/latest/iai_callgrind/) | **Reject.** Its strength is instruction counts, which duplicates the criterion series `bench.yml` already publishes, and its DHAT mode duplicates the in-process counter — while confining the gate to Linux, so the macOS job loses it. Reconsider only if the criterion series' variance ever becomes the problem. |
| **`cargo-careful`** | Runs the standard library with debug assertions enabled, catching some UB cheaply. | **Reject.** Nightly, and its coverage overlaps Miri's on a workspace with one `unsafe` block. |
| **`core::arch::wasm32::memory_size`** | Stable; returns the current linear-memory size in 64 KiB pages. Linear memory never shrinks under `wasm32-unknown-unknown`, because freed memory returns to the allocator's free list rather than to the host. | **Adopt for the wasm tier.** Neither Miri nor the sanitizers reach wasm at all. Monotonicity is what makes this a *good* leak signal rather than a weak one: growth after warm-up is real growth. |
| **`benchmark-action/github-action-benchmark`, custom formats** | `customSmallerIsBetter` accepts a JSON array of `{name, unit, value, range?, extra?}`, with `alert-threshold` and `fail-on-alert` applied against the stored series. [Repository](https://github.com/benchmark-action/github-action-benchmark) | **Adopt for tracking.** The workflow, the gh-pages series, the warm-up discipline, and the path filter already exist. A second series with its own `name:` is a step, not a pipeline. |

## Why the counter belongs in a test binary

`cargo nextest` runs each test in its own process. That is stated as a core feature and is why
nextest is recommended with Miri in the first place — each test gets an isolated context, which
"simplif[ies] operations like memory leak detection". The repository already pins nextest 0.9.137
and runs every gate through it, with `cargo test` reserved for doctests.

That process isolation is what makes a global counter usable. Put the allocator and the assertions
in `lang/tests/allocation.rs` and `orcvs/tests/allocation.rs`:

- An integration test is its own binary, so `#[global_allocator]` there cannot reach the shipped
  `shell` binary or any other test target.
- Under nextest each `#[test]` in that binary is a separate process, so counters cannot race
  between tests.
- No feature flag is needed, which is one fewer feature combination for the contract's "exercise
  explicit feature combinations" rule to cover.

Gate the file the way the property suites are already gated, since `System` and this counting are
native-only concerns:

```toml
[target.'cfg(not(target_arch = "wasm32"))'.dev-dependencies]
```

Nothing needs to change in `mise.toml`. `cargo nextest run --workspace --profile ci --locked`
already builds and runs every integration test in the workspace, on both Linux and macOS, on every
pull request. That is the whole point: a memory test that is just a test needs no new gate,
no new tool pin, and no new workflow.

## The allocator, and the two trades it carries

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    // `const` initialisation is what keeps this safe to touch from inside the
    // allocator: a lazily initialised thread-local would allocate on first
    // access, from within the allocation it is counting.
    static BLOCKS: Cell<usize> = const { Cell::new(0) };
    static BYTES: Cell<usize> = const { Cell::new(0) };
}

struct Counting;

// SAFETY: every method forwards to `System` with the layout it was handed and
// returns exactly what `System` returned, so this allocator's contract is the
// one `System` already upholds. The counters are separate state and touch
// neither the pointer nor the layout.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        BLOCKS.with(|n| n.set(n.get() + 1));
        BYTES.with(|n| n.set(n.get() + layout.size()));
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOC: Counting = Counting;
```

`realloc` and `alloc_zeroed` are deliberately not overridden. `GlobalAlloc`'s default `realloc`
is alloc-copy-dealloc through `Self`, so a grow is counted honestly; overriding it to reach
`System`'s `mremap` would be faster and would need the counting written twice. Test binaries do not
need the faster path.

**Trade one: unsafe, to avoid a dependency.** The contract prefers safe Rust and asks that
dependencies carry a recorded rationale. This swaps a dependency for one `unsafe impl` whose
invariant is "forwards to `System` unchanged" — about as small as an unsafe scope gets, and it
satisfies `undocumented_unsafe_blocks` with the comment above. The alternative is a crate that
was last released in 2023 or one that says its own maintenance is not a priority. Recording that
comparison in the change is the rationale.

**Trade two: thread-local versus atomic.** Thread-local counting is correct when the measured work
happens on the calling thread — `lang`'s parse and interpret paths, `orcvs`'s Source writes, and
egui frame stepping all qualify, and it makes the tests correct even under a bare `cargo test`,
where tests share a process and run in threads. Anything measured across a multi-threaded tokio
runtime needs `AtomicUsize` with `Ordering::Relaxed` instead, and is then correct only under
nextest's process isolation. Prefer thread-local; reach for atomics only where the measured work
is genuinely multi-threaded, and say so in the test.

## What to assert

Absolute allocation counts rot: a compiler release or a dependency bump moves them, and the fix is
always to update the number, which teaches everyone to update the number. Assert shapes instead.

- **Zero.** "A Tick over an already-parsed Source allocates nothing." "`Source::set` on one Cell
  allocates nothing." Zero never needs updating and never drifts.
- **Independent of size.** "Parsing a 16-row Source and a 64-row Source allocate within a constant
  of each other." This catches an accidental per-Cell allocation without pinning a number.
- **Flat in steady state.** Measure the delta over frames 200–400 and again over 400–600 and assert
  they match. This is the one that catches leaks, and it is immune to warm-up noise.

Where a number genuinely must be pinned, pin it in the tracked series (below) with an alert
threshold rather than in an assertion, so a change shows up as a reviewed movement instead of a
red gate somebody edits away.

## egui and `shell`

`shell` has no native test target today — `shell/tests/wasm.rs` is `#![cfg(target_arch = "wasm32")]`
and covers the web startup path. `egui_kittest` closes that gap for the native side, and the growth
assertion is the reason to want it:

```rust
let mut harness = egui_kittest::Harness::builder()
    .build_eframe(|cc| Console::new(cc));

for _ in 0..200 { harness.step(); }
let settled = snapshot();          // blocks, bytes, and ctx memory sizes
for _ in 0..200 { harness.step(); }
let after = snapshot();
assert_eq!(settled.retained, after.retained);
```

What this catches that nothing else does: egui retains per-`Id` state in `Context::memory`, and a
widget whose `Id` varies per frame grows that map forever without any visible symptom until the
application has been open for an hour. Same for a diagnostics buffer with no cap, or a texture
handle re-created per frame. `shell/src/console.rs` is a thousand lines and none of it is currently
exercised by a test.

Two integration risks to resolve before wiring this into the pull-request tier:

- `Console::new(cc)` may open MIDI through `midir` on the desktop targets. `test.yml` installs
  `libasound2-dev` so it will *build*, but a headless runner has no ALSA sequencer to open, so the
  constructor's failure path is what CI will exercise. Check whether that path is graceful; if it
  is not, the harness needs a constructor seam that skips MIDI, which is worth having regardless.
- `eframe` is pinned with `default-features = false` and `glow`. `egui_kittest`'s `eframe` feature
  drives `App::update` against a test context rather than creating a window, so no GL context should
  be required — confirm that against 0.36.2 before committing to it, because it is the assumption
  the whole approach rests on.

Add `egui_kittest = { version = "0.36", default-features = false, features = ["eframe"] }` as a
native-only dev-dependency. Do **not** enable `snapshot` or `wgpu`: those pull `image`, `dify`, and a
GPU requirement, and image-diff snapshot tests are a separate decision from memory testing.

## Wasm

Miri does not run wasm and the sanitizers do not target it, so the wasm tier gets the one signal
that is actually available: linear memory only ever grows.

```rust
fn pages() -> usize { core::arch::wasm32::memory_size(0) }
```

Sample it in `shell/tests/wasm.rs` after a warm-up run, drive the app through a long sequence of
writes and Ticks, and assert the page count has not moved. This belongs in `test_wasm`, which the
merge tier already runs under headless Firefox — the pull-request tier does not run the browser
suite, and this assertion needs a long run, so the merge tier is where it costs nothing extra.

## Tracking, not just gating

`bench.yml` already builds both benchmarked crates, warms up, runs, and publishes to a gh-pages
series with `alert-threshold: '150%'` and `fail-threshold: '300%'`. Those thresholds are calibrated
for wall clock, which is noisy on shared runners. Allocation counts are not noisy — they are
deterministic for a fixed input — so a memory series wants far tighter bounds.

Emit `[{name, unit: "blocks", value}, ...]` from a small harness, and add a second
`benchmark-action/github-action-benchmark` step with its own `name:` (the action keys the stored
series by it, and `bench.yml` already notes that the existing `lang` name must never change).
Thresholds around `110%`/`125%` are defensible where `150%`/`300%` are not.

The path filter, the warm-up rationale, the `shell: bash` pipefail note, and the publish
concurrency group all apply unchanged. This is a step added to an existing job, not a new workflow.

## Miri, scoped honestly

`.scratch/verification-gaps/issues/12` decided that the contract stops *requiring* Miri, because
`rust-toolchain.toml` pins stable and `AGENTS.md` states nightly is optional and non-blocking. It
also said Miri "is still the tool this gate would prefer" and that it should be run deliberately.

A `workflow_dispatch` job plus a `mise` task is what "deliberately" looks like without contradicting
that decision:

```sh
rustup toolchain install nightly --component miri
cargo +nightly miri nextest run --package orcvs -E 'test(/source/)'
```

Scoped by test filter, not by crate. `orcvs` links `midir` and a multi-threaded tokio runtime, and
Miri can execute neither, but Miri interprets what runs — an FFI dependency that no selected test
calls never becomes a problem. The Source model tests are pure logic over a `String`, and they are
what reaches the one `unsafe` block.

Keep it off the required checks. It is a tool for a change that touches the byte write, not a tax on
every pull request.

## Suggested sequence

1. Counting allocator plus zero-allocation and size-independence assertions in `lang/tests/` and
   `orcvs/tests/`. No CI change, no dependency, runs on both platforms in the pull-request tier.
2. `egui_kittest` native test for `shell` with the steady-state growth assertion, after resolving
   the MIDI-constructor and windowless-eframe questions above.
3. Linear-memory growth assertion in `shell/tests/wasm.rs`, running in the merge tier.
4. A `customSmallerIsBetter` series published from `bench.yml` with tighter thresholds.
5. A non-gating `workflow_dispatch` Miri task scoped to the Source model tests.

Steps 1 through 3 are where the signal is. Step 4 turns single assertions into a trend. Step 5 is
insurance on the one line of `unsafe` in the workspace.

## Primary sources

- [Miri](https://github.com/rust-lang/miri) — detected UB classes, nightly requirement, FFI and
  syscall limitations, `MIRIFLAGS`, and the CI shape.
- [cargo-nextest Miri integration](https://nexte.st/docs/integrations/miri/) — `cargo miri nextest run`,
  process-per-test isolation, and the archiving and cross-test race caveats.
- [Rust unstable book: sanitizers](https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/sanitizer.html) —
  available sanitizers, per-target support, `-Zbuild-std`, and LeakSanitizer's platform list.
- [`dhat`](https://docs.rs/dhat/latest/dhat/) — testing mode, `HeapStats`, single-profiler
  constraint, and the global-allocator requirement; [lib.rs](https://lib.rs/crates/dhat) for the
  version, dependency list, and the maintenance disclaimer.
- [`allocation-counter`](https://docs.rs/allocation-counter) and
  [lib.rs](https://lib.rs/crates/allocation-counter) — `measure()`, `AllocationInfo` fields,
  zero runtime dependencies, thread-local counting, and the 2023 release date.
- [`egui_kittest`](https://docs.rs/egui_kittest/latest/egui_kittest/) and
  [`HarnessBuilder`](https://docs.rs/egui_kittest/latest/egui_kittest/struct.HarnessBuilder.html) —
  `build_eframe`, frame stepping, and which features require a GPU.
- [`iai-callgrind`](https://docs.rs/iai-callgrind/latest/iai_callgrind/) — supported Valgrind tools,
  platform and Valgrind requirements, and regression thresholds.
- [`benchmark-action/github-action-benchmark`](https://github.com/benchmark-action/github-action-benchmark) —
  the `customSmallerIsBetter` JSON schema and the alert and failure thresholds.
- [Microsoft Rust engineering practices, ch. 5](https://microsoft.github.io/RustTraining/engineering-book/ch05-miri-valgrind-and-sanitizers-verifying-u.html) —
  the division of labour between Miri and the sanitizers, and the advice to run Miri on the crates
  that contain unsafe rather than the whole workspace.
