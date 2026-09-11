# 01 — Say which Turn a computation took

**What to build:** Record the Turn each computation took, and let a test read it.

`execution::execute` already iterates the schedule's order. The ordinal is the loop counter:

```rust
let Schedule { lookup, order, diagnostics } = schedule;
let mut execution = Execution::new(grid, bytes, map, tick, &lookup, diagnostics);
for index in order {
    if let Break(diagnostic) = execution.take_turn(index) {
        return execution.reject(diagnostic);
    }
}
```

Add `turn: Option<usize>` to `ComputationState`, initialised `None` where `Execution::new` builds the states, and set from the loop as each Turn is taken. Add a `pub(in crate::source) fn turn(&self) -> Option<usize>` accessor beside `interpreted` and `interpretations`, matching their form: a doc comment saying what the field is, what reads it, and why the suppression is `allow` rather than `expect`, and `#[allow(dead_code, reason = "an output the shipped callers discard")]`.

`None` means the Tick ended before reaching that computation. Do not record at schedule time and do not use a sentinel — both conflate a Turn that ran with one the Tick never reached, and `reject`'s own comment says that distinction is the point.

**Demonstrate it, do not convert everything.** Rewrite a small number of existing tests whose claim is about ordering and whose only witness today is Source bytes — `live_a_row_reservation_names_no_computation_of_the_next_row` (`tick.rs:2906`) is the clearest, and its own 17-line comment says what it is really asserting. Add the Turn assertion **beside** the byte assertion rather than replacing it: the bytes stay as a second check, and the point is that a mutant producing the right bytes for the wrong reason now has a second assertion to get past. Three or four such tests is the right size. The other ~80 byte-offset assertions stay as they are.

Nothing about ordering changes. The same computations take the same Turns in the same sequence.

**Status:** resolved

- [x] `ComputationState` carries `turn: Option<usize>`, written in `execute`'s loop.
- [x] A computation the Tick never reached keeps `None`, and a test proves it through a rejected Tick.
- [x] The accessor matches `interpreted`'s form, including a doc comment explaining the `allow`.
- [x] Three or four ordering tests assert the Turn alongside their existing byte assertion.
- [x] No test's expectation about Source bytes, writes, Play Commands or diagnostics changes.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `orcvs` and `console`, which depends on it. `.scratch/` edits also owe `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`.

No benchmark is owed: one `Option<usize>` write per Turn on a path that already walks every computation, and the ticket claims no performance effect.
