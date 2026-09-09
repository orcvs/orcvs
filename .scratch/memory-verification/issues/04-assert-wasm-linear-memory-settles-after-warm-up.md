# 04 — Assert wasm linear memory settles after warm-up

**What to build:** A long run of the web application in the browser suite leaves the wasm linear
memory at the size it reached after warm-up, so a leak on the web target is caught by the merge tier
instead of by a user's tab.

**Blocked by:** None — can start immediately.

**Status:** ready-for-human — the assertion is written and compile-checked, and has never run.

- [x] Linear-memory size is sampled in the browser regression suite, after a warm-up run and again
      after a long sequence of Source writes and Ticks, and the two are asserted equal.
- [ ] The warm-up length and the run length are chosen so the first sample is past allocator growth,
      and the choice is stated. **The choice is stated; that it is past allocator growth is
      reasoned, not measured** — see the finding below.
- [x] The assertion states why monotonicity makes this a sound leak signal: freed memory returns to
      the allocator's free list rather than to the host, so linear memory never shrinks and any
      growth after warm-up is real growth.
- [x] The test runs inside the existing headless browser task in the merge tier. No new task, no
      workflow change, and `scripts/check-tooling-contract.sh` is untouched.

## Comments

Independent of every other issue in this effort. It needs no counting allocator, no dependency, and
nothing from `01`, because the measurement is a stable standard-library call on the wasm target.

It belongs in the merge tier rather than the pull-request tier for two reasons: the pull-request tier
does not run the browser suite at all, and the assertion wants a long run.

Neither Miri nor any sanitizer reaches wasm, so this is the only memory signal the web target gets.

### Finding: the assertion is unverified, and `check_wasm` cannot verify it (2026-09-09)

`web_linear_memory_settles_after_warm_up` in `shell/tests/wasm.rs` is landed as the ticket asks —
`core::arch::wasm32::memory_size(0)` sampled after warm-up and again after a longer run, asserted
equal. It has never been executed. `mise run test_wasm` needs headless Firefox and a full wasm-pack
build, and `CLAUDE.md` defers it to the merge tier, so the merge tier is the first place this
assertion will run at all.

`mise run check_wasm` is *not* a substitute, and the surprise is worth recording. The task builds
the Trunk distribution and type-checks with `--lib`, and its own inline comment says so: it
"type-checks no test target". Running it against this change passes without ever compiling the file
the change is in. The strongest check available without a browser is therefore the test target
itself, compiled for the wasm triple:

```sh
cargo clippy --package shell --target wasm32-unknown-unknown --tests --locked -- -D warnings
```

That is what was run, and it is clean. It proves the API surface the test leans on — `Source::set`,
`Source::execute`, `Grid::cell_index`, `Tick::new`, and the `memory_size` intrinsic's legacy
const-generic call form — and nothing about the assertion's truth.

**Why the equality was kept rather than softened.** The honest alternative is a ceiling, in the
convention `01` established: `assert!(pages() <= settled + slack)`. It was not taken. A ceiling here
would be a number chosen to absorb an effect nobody has observed, and the first run in the merge
tier would then be unable to distinguish "the page budget was right" from "the slack swallowed a
leak". Equality is the shape the ticket asked for, it is the shape monotonicity actually licenses,
and if the merge tier reds it, that failure carries information a green ceiling would have hidden.
Anyone relaxing it should land the observed page delta here first, the way `01` recorded its counts.

**Why 512 and 2,048, given no measurement to tune them to.** Deliberately, so that neither number is
a fitted constant:

- The workload is one program rewritten and Ticked, identically, every iteration. For a repeating
  workload the allocator's high-water mark is reached once the busiest single iteration has run
  once — the dlmalloc arena, the parser's and interpreter's scratch buffers, and every collection's
  capacity all stop growing there. That is a handful of iterations. 512 is two orders of magnitude
  past it, which is slack against a future path that needs more, not a tuned number.
- The measured span is `4 * WARM_UP_TICKS`, so it does strictly more work than the span the first
  sample was taken after. A per-iteration leak large enough to have grown memory during warm-up
  grows at least four times as much in the measured span and cannot hide inside a page warm-up
  already paid for.
- At 28 Cell writes per Tick the measured span is 2,048 Ticks and 57,344 Source writes, so the
  assertion's resolution is one 64 KiB page over 2,048 Ticks — about 32 leaked bytes per Tick. It
  will not catch anything finer than that, and saying so is part of the claim.

**Why the test drives `Source` and not `SourceCommander`.** `SourceCommander::execute` is
`pub(crate)`, and the only public way to spend a Tick from outside `orcvs` is
`PlaybackEngine::start`, which is driven by a browser timer. A page count taken across a timed run
would depend on how many Ticks happened to fire, which is not a shape an assertion can hold.
`Source` is the object both the Commander and the engine drive, so writing and Ticking it directly
is the same work, synchronously.

**A risk the merge tier will settle.** Linear memory is per-instance, not per-test, and every
`#[wasm_bindgen_test]` in `shell/tests/wasm.rs` shares one wasm instance. The two samples are taken
inside one test body with no harness code between them, and wasm-bindgen-test runs tests one at a
time, so nothing else should allocate inside the measured span. "Should" is doing work in that
sentence, and only a run can remove it.
