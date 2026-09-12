# 05 — Decide a Cell's background once

**What to build:** `CellVisuals.background` becomes `Option<Color32>`, and the two arms of
`cell_visuals` that answer `PALETTE.source` answer `None` instead.

## One decision, answered in two modules

`cell_visuals` returns a background unconditionally (`console/src/style.rs:83-92`):

```rust
background: if selected && !cursor_visible {
    PALETTE.selection_fill
} else if cursor_visible {
    PALETTE.source
} else if let Some(bloom) = cursor_bloom {
    bloom_colours(bloom).0
} else {
    PALETTE.source
},
```

Two of its four arms answer `PALETTE.source`. `Paint::derive` then re-decides the same question
(`console/src/paint.rs:137-138`):

```rust
background: (visuals.background != PALETTE.source)
    .then_some(visuals.background),
```

The function that owns the palette says "fill it with the Source colour"; the function one module
away says "then don't fill it". One decision, made twice, in two places, by comparing values instead
of by stating it.

**`None` does not mean transparent.** It means the `CentralPanel` behind the Grid has already painted
that exact colour — a fact established by `source_panel_frame()` (`console/src/console.rs:950-952`),
whose doc comment says so: "The fill is load-bearing rather than decorative." So `Paint` depends on
the panel's fill, and that dependency appears in no signature anywhere. It is carried entirely by a
comment and by two tests.

## What the double decision costs

**A comment block.** `console/src/paint.rs:120-136` — seventeen lines explaining the comparison,
ending on the admission that the condition it works out to "reads wrong and is right". A comment that
has to apologise for the code it describes is the code asking to be changed.

**A test that evaluates the expression under test.** `console/src/paint.rs:438-475`,
`the_cell_needing_no_background_is_exactly_the_one_filled_with_the_source`, calls `cell_visuals`
itself (`:449-454`) and then asserts (`:457-462`):

```rust
assert_eq!(
    background.is_none(),
    visuals.background == PALETTE.source,
```

The right-hand side is the expression under test, recomputed by the test. It cannot fail for the
reason a test is supposed to catch, because both sides come from the same call. Its own doc comment
knows what it is for — "the one assertion tying the skip to the function it must not drift from"
(`:416-417`) — and drift is only possible because the decision is in two places.

**A second drift guard for a phase the first cannot reach.**
`the_skip_condition_matches_cell_visuals_in_both_blink_phases` (`console/src/console.rs:2364-2389`)
is an 80-case truth table over `cell_visuals` — four Glyphs, five bloom states, two selected, two
blink phases — asserting that
`visuals.background == PALETTE.source` equals `cursor_visible || (!selected && bloom.is_none())`. It
exists because the first test can only ever see `cursor_visible == false` — a running `Orcvs` turns
the Cursor on by elapsed time and nothing public sets it — so the two halves of one guard live in two
files, each with a doc comment explaining why the other one cannot do its job.

**And the guard on the panel fill.** `the_omitted_background_is_the_colour_the_panel_is_filled_with`
(`console/src/console.rs:2328-2336`) pins `source_panel_frame().fill == PALETTE.source`. That one
earns its keep and stays whatever else happens: the panel really does have to be filled with the
colour the Cells decline to paint.

**`cell_visuals` has exactly one production caller** — `console/src/paint.rs:112`. Every other call in
the workspace is a test (`console/src/style.rs:161-166, 246-247, 268-270, 295-299`,
`console/src/paint.rs:378, 449`, `console/src/console.rs:2376`). So the signature change reaches one
shipped line.

## The fix

Return `None` from the two `PALETTE.source` arms and make the field an `Option<Color32>`.
`Paint::derive` then writes `background: visuals.background` and the seventeen-line comment, the
comparison, and both drift tests delete themselves — there is no longer a second decision for them to
hold to the first. Move what the comment explains into `cell_visuals`'s own doc: that a `None` Cell
is one the panel behind the Grid has already painted, naming `source_panel_frame`, so the dependency
is stated where the decision is made rather than in a third module that neither of them imports.

The two `style.rs` tests asserting `ordinary.background == PALETTE.source` and
`distant.background == PALETTE.source` (`console/src/style.rs:272, 276, 305`) become assertions that
the background is `None`, and are then the one place the skip is stated — beside the palette, which
is where it belongs.

## The consequence to evaluate, not to promise

The reviews noted a cascade below this, and it is filed as a **question for the implementer to
answer in the ticket**, not as an acceptance bar:

With the background un-filtered at source, `Paint::background_runs()`
(`console/src/paint.rs:217-250`) need not be a second pass over already-derived data — the run could
be closed as the derive walks the row. If runs were built that way, `BackgroundRun`'s half-open
`Range<usize>` (`console/src/paint.rs:70`) need not leak to its consumer, and the consumer's
`run.columns.end - 1` with the `debug_assert!` guarding it
(`console/src/console.rs:597-605`, added `df3d2e4`, 2026-09-12) could go with it.

**Do not commit to that cascade.** It is a second derivation shape with its own trade-offs — a
single-pass fold is harder to test in isolation than the standalone `background_runs()` the
`source-paint` spec deliberately chose, and "derived, never stored" was an explicit decision there.
Report whether it holds; do not treat failing to do it as failing this ticket.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] `CellVisuals.background` is `Option<Color32>`. The `cursor_visible` arm and the final `else`
      arm of `cell_visuals` answer `None`.
- [ ] `cell_visuals`'s doc comment states what `None` means and names `source_panel_frame` as the
      thing that makes it true.
- [ ] `Paint::derive` passes the background through. No comparison against `PALETTE.source`
      anywhere in `console/src/paint.rs`, and the seventeen-line comment at `:120-136` is gone.
- [ ] `the_cell_needing_no_background_is_exactly_the_one_filled_with_the_source` and
      `the_skip_condition_matches_cell_visuals_in_both_blink_phases` are deleted, not rewritten. They
      guard a drift that no longer has two places to drift between.
- [ ] `the_omitted_background_is_the_colour_the_panel_is_filled_with` survives unchanged. The panel
      still has to be filled with the colour the Cells decline to paint, and nothing else says so.
- [ ] `console/src/style.rs`'s existing `cell_visuals` tests assert `None` where they asserted
      `PALETTE.source`, and at least one of them says in a comment that `None` is a Cell the panel
      has already painted.
- [ ] Nothing about what is drawn changes. Same colours, same geometry, same order, both blink
      phases — and in particular the Cursor's own Cell still takes no fill on the visible half of the
      blink, which is the case the deleted truth table existed to pin.
- [ ] The ticket reports, in its Comments, whether the single-pass `background_runs` cascade holds —
      and if it does, files it rather than doing it here.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --package console --all-targets --locked -- -D warnings
PROPTEST_CASES=32 cargo nextest run --package console --locked
```

`console` has no dependents and nothing here touches `orcvs`, so the crate-scoped run is the whole
gate. No doctest run is owed: `CellVisuals` is `pub(crate)`.

## Comments

`ready-for-agent`. The change is one type, two `match` arms and one field assignment; the tests that
go are named individually; the test that stays is named; and the part that requires judgement — the
`background_runs` cascade — is explicitly carved out as a question to report rather than work to do.

This one is independent of the seam criterion. Both modules are already in `console`, so the fault
here is not the ADR 0022 drift at all; it is the same "nothing is rewired" constraint from `20d2a8a`
landing a new layer on top of an unchanged one. It survives on its own merits if ticket `01` is
refused.
