# Draw the crate seam on responsibility rather than on the dependency

**Goal:** Move the presentation decisions that accumulated in `orcvs` into `console`, and record the
criterion that keeps them there — so that what belongs on each side of the seam is answered by whose
job it is, not by whether the answer happens to name a type `egui` owns.

## Why

ADR 0022 drew the `orcvs`/`console` seam on a dependency criterion. Its words are exact:

> `orcvs` does not depend on `egui`, `eframe`, or `winit`, and no module moves into `orcvs` until it
> is free of them.

That is a necessary condition, and it has since been used as a sufficient one. Nothing enforces the
seam but the compiler, and the compiler only knows about imports. So every presentation decision
that can be spelled without naming `Color32` has drifted across into the core crate, and it drifted
without argument, because at each step it compiled. `RenderFrame` is where they collected.

The seam is now a colour firewall. `orcvs` decides the entire aesthetic of the Source Grid and hands
`console` a list of colours to look up.

### The exhibit that settles it

`orcvs/src/render_frame.rs:177` declares:

```rust
const SECTOR_SEAM_STRENGTHS: [u8; 4] = [100, 72, 34, 13];
```

Those are alpha percentages. The only thing that reads them is `sector_seam_strength`
(`orcvs/src/render_frame.rs:179-194`), which picks one by a four-arm falloff and hands it out as
`RenderCell::sector_left_strength`/`sector_top_strength`. The only thing that consumes *those* is
`console/src/style.rs:115-119`:

```rust
pub(crate) fn sector_line(strength_percent: u8) -> Color32 {
    let [red, green, blue, base_alpha] = PALETTE.sector_line.to_srgba_unmultiplied();
    let alpha = u16::from(base_alpha) * u16::from(strength_percent.min(100)) / 100;
    Color32::from_rgba_unmultiplied(red, green, blue, alpha as u8)
}
```

`console` contributes the RGB triple. `orcvs` contributes the opacity ramp, the falloff that selects
it, the hash that punches stable gaps into the faint middle of each arm
(`orcvs/src/render_frame.rs:190-193`), and the rule that a seam only exists on a Cell whose column or
row is a multiple of the marker spacing (`:93-100`). The same is true of the Cursor bloom:
`cursor_bloom` (`:141-156`) computes a Chebyshev distance, perturbs it with a hash-driven
`signal_breakup` (`:158-166`), and `classify_cursor_bloom` (`:196-209`) sorts the result into four
bands whose widths the comment states as "1:1:2:3, with cumulative radii 1:2:4:7". `console`'s whole
contribution is `bloom_colours` (`console/src/style.rs:106-113`), eight palette entries behind a
`match`.

None of that is a fact about a Source. All of it is a fact about how this console wants a Grid to
look.

### The vocabulary confirms it

`CLAUDE.md` names `CONTEXT.md` and `docs/adr/` as the sources of truth for architecture and
vocabulary. `CursorBloom` is a publicly exported enum of the core crate — `pub enum CursorBloom` at
`orcvs/src/render_frame.rs:15`, imported by `console/src/style.rs:5` — and:

- the string `bloom` appears nowhere in `CONTEXT.md` and nowhere under `docs/adr/`;
- `CONTEXT.md` has no **Cursor Bloom** entry and no **Sector Seam** entry. Its 67 glossary terms
  include **Marker**, **Glyph**, **Render Frame**, **Cursor** and (since `20d2a8a`) **Paint**. The
  only occurrence of "sector seams" in the whole file is inside the **Paint** entry at
  `CONTEXT.md:272`, where it is named as a thing a Paint carries — not defined as a thing;
- no ADR records the decision to have a bloom at all.

A public type of the core crate, carrying the console's entire aesthetic, with no glossary entry and
no recorded decision. That is what a seam drawn on imports permits.

### It is not only the Render Frame

`orcvs/src/opts.rs:3` declares `pub const DEFAULT_FONT_SIZE: f32 = 18.0;`. Nothing inside `orcvs`
reads it; every reader is in `console` (`console/src/console.rs:18, 278, 753, 1832, 2106, 2155`). A
font size is a presentation quantity living in the core crate because `f32` is not `Color32`. The
doc comment on `Opts` (`orcvs/src/opts.rs:12-17`) opens "How the console presents and plays a
Source", which states the drift in the crate's own words.

This effort does not chase that one. It is named here because it shows the fault is the criterion
rather than one careless module.

## Provenance

Three independent design reviews of the `source-paint` branch reached this finding separately. One
of the three designed the layering from the problem statement before reading the shipped code, and
arrived at the same top-level split with a different distribution of responsibility below it. The
overlap between three reviews that did not see each other's output is what makes this worth an
effort rather than a comment on a pull request.

## What `source-paint` got right — this effort does not reverse it

`source-paint` is a genuine improvement and this effort builds on it.

**The top-level `Paint` | `SourceShapes` split was judged correct by all three reviews**, including
the one that designed the problem from scratch. Deriving the per-Cell decision as a value that
carries no geometry, and converting it to Shapes in a separate step, is right, and nothing here
proposes undoing it. The effort's own record stands: `source-paint/02` through `06` are `resolved`,
and `source-paint/01` — the ADR — remains `ready-for-agent` and is not superseded by anything here.

What is wrong is everything *below* that line, and the reviews traced it to one constraint stated
in commit `20d2a8a`'s own message:

> Nothing is rewired. `show_source` keeps its loop and paints exactly what it painted, so the module
> lands with nothing broken and ticket 06 is a rewire rather than an invention.

That was the right call for landing a module safely. It also means the new layer was fitted to the
shape of what was already there: `Paint` accepts a `RenderFrame` that destroys its Cursor into a
thousand bools and then recovers it by scanning; it accepts a `Vec<Vec<RenderCell>>` it immediately
flattens; it accepts a `CellVisuals` whose background it has to second-guess against a palette
constant. Each of those is a seam the layer below chose, and none of them was reopened, because
reopening them was out of scope by construction.

This effort is that reopening.

## What changes

`RenderFrame` ends up answering four things: the `Grid` it was derived from, the `Position` of the
Cursor, whether the Cursor is currently visible, and `at(Position) -> &RenderCell` over a flat `Vec`
of Cells carrying content and Glyph. The classification of a Cell as Source content is the model's;
the decision to surround it with a four-band glow is the console's.

Concretely:

- `RenderFrame::cursor()` replaces a round trip through a thousand bools (ticket `02`).
- `RenderFrame` stores its Cells flat, like `Paint` already does (ticket `03`).
- `cursor_bloom`, `signal_breakup`, `cell_hash`, `sector_seam_strength` and `classify_cursor_bloom`
  move to `console` (ticket `04`).
- `CellVisuals.background` becomes `Option<Color32>`, so the "no fill here" decision is made once,
  where the palette is (ticket `05`).
- Three smaller boundary faults, each of which is the same criterion failing at a smaller scale
  (tickets `06`, `07`, `08`).

### How the small findings were grouped

Six small findings came out of the reviews. They are filed as three tickets rather than six or one,
and the grouping is by *sentence*, not by size — a ticket should be one thing a reviewer can agree or
disagree with:

- **`06` — `Orcvs`'s public surface.** The missing `Orcvs::grid()`, the unused public
  `Orcvs::index()`, and the test whose title (`app_exposes_a_render_frame_without_leaking_its_grid_or_cursor`)
  has been false since `RenderFrame::grid()` was published. One sentence: `Orcvs`'s API was settled
  by what its callers could reach rather than by what they were asking for.
- **`07` — the geometry layer's parameters.** `SourceShapes::new` taking a `scale` alongside the
  viewport it was computed from, and `grid_viewport.rs` taking loose `usize` counts and then
  re-guarding them. One sentence: take the thing, not a copy of a fact about the thing.
- **`08` — glyph placement welded to rectangle placement.** Filed alone, because it is not small. It
  restructures the loop every painted shape goes through, and its payoff needed correcting.

A single "boundary tidy" ticket was refused. It would have carried three unrelated arguments under
one `Status:`, so a maintainer wanting two of the three would have had nothing to say yes to.

## What is deliberately not in scope

- **Splitting `orcvs` further.** ADR 0022 leaves the ADR 0001 cut — Source from Playback — waiting
  for a measurement that asks for it. Nothing here asks for it.
- **The `Opts` drift.** `DEFAULT_FONT_SIZE` and `Opts`'s "how the console presents" doc comment are
  cited above as evidence and are not touched by any ticket. They are a second application of the
  same criterion and deserve their own effort, after the criterion is written down.
- **Making `orcvs` depend on `egui`.** The dependency rule of ADR 0022 is kept exactly as written.
  This effort refines the criterion for what may live in `orcvs`; it does not relax the one thing
  that criterion was always sufficient for.
- **The bloom's cost.** `render-frame-derivation` owns that. See below.
- **The Evaluator/Interpreter and Marker/Highlight vocabulary drifts.** `source-paint` put them out
  of scope and they stay out.

## How this sits with the other open efforts

**`source-paint/01` — ADR 0040.** Still open, still `ready-for-agent`, still takes **0040**. It
records a different decision: that the console derives a per-Cell paint as a value and converts it to
Shapes separately — a decision about the internals of one crate. This effort's ADR records where the
model/presentation boundary sits between two crates, and refines ADR 0022's criterion. Two
decisions, two files. **`source-paint/01` is not superseded**, and ticket `01` here takes **0041**
and cites 0040 rather than replacing it. The alternative — folding both into one ADR — was
considered and refused: it would re-open a ticket that is specified down to its checkboxes, and it
would put a decision about the inside of `console` and a decision about the seam between crates in
one file, which is exactly the conflation this effort exists to undo.

**`render-frame-derivation/02` — the bloom's cost.** That ticket owns *what the bloom costs*; ticket
`04` here owns *where the bloom lives*. They are not duplicates and neither subsumes the other, but
they collide, because moving the code changes which crate pays for it — and `[[bench]]` exists only
in `orcvs/Cargo.toml:67` and `lang/Cargo.toml:35`. The only harness that can see the bloom's cost is
`source_render_frame` (`orcvs/benches/source.rs:284-295`), and after ticket `04` the bloom is no
longer inside what that group measures. So the sequence is forced rather than chosen:
`render-frame-derivation/01` (`ready-for-agent`) measures first, `render-frame-derivation/02` decides
on that number, and ticket `04` moves the code afterwards, carrying whatever early-out `02` landed
with it. Ticket `04` blocks on `render-frame-derivation/01` for that reason and says so.

**`retired-glyph-vocabulary` — the words.** Filed concurrently with this effort, against the same
two concepts from the other side: its `01` writes the **Sector Seam** and **Cursor Bloom** glossary
entries, and its `05` renames `Opts::marker_spacing` and `Opts::highlight_dot_spacing` after what
they actually control. **This effort writes neither.** Ticket `04` blocks on that effort's `01` and
takes its words, on the precedent `source-paint/01` set with the **Console** entry
`console-testing/02` owned. The split is clean: `retired-glyph-vocabulary` decides what the two
concepts are *called*, this effort decides which crate they *live in*. Nothing here edits
`CONTEXT.md`.

## Triage

Every ticket's `Status:` and the argument for it. The rule applied: `ready-for-agent` only where the
change is fully specified, decided, and needs no judgement an agent should not be making;
`needs-triage` where a maintainer has to agree to something before the work is worth starting.

| Ticket | Status | Why |
| --- | --- | --- |
| `01` ADR 0041 | `needs-triage` | It proposes revising the criterion of an accepted ADR. That is the maintainer's call and it is the premise the rest of the effort rests on. |
| `02` Cursor round trip | `ready-for-agent` | Mechanical, decided, no aesthetic judgement. `Position` is `Copy` and is already a parameter of `derive`. Nothing about the seam has to be settled first. |
| `03` Flat Cells | `needs-triage` | `source-paint/spec.md` explicitly decided the other way and said it was not to be filed as a follow-up. This ticket argues that decision was reached on test-churn cost. A maintainer has to overrule it. |
| `04` Move the aesthetic | `needs-triage` | The largest change here, gated on `01`'s criterion and on a measurement in another effort. |
| `05` Background decided once | `ready-for-agent` | The core change is two `match` arms and a type; the tests that delete themselves are named. Its knock-on cascade is explicitly filed as a question, not as part of the acceptance bar. |
| `06` `Orcvs`'s surface | `ready-for-agent` | Three small facts about the public API, each independently checkable, none of which needs the seam decision. |
| `07` Viewport derives its inputs | `ready-for-agent` | Two parameters that can be computed from another parameter. The one trap — the name `scale` is taken — is named in the ticket. |
| `08` Glyph placement | `needs-triage` | A restructure of a shipped loop whose payoff (five tests stop building a `Context`) is real but partial; the reviews' estimate that only one test would still need the atlas does not survive checking. |

## Verification

Documentation and `.scratch/` only for `01`. For each code ticket, on the crates it edits and their
dependents — `orcvs` means `orcvs` and `console`; `console` has no dependents:

```sh
cargo fmt --all -- --check
cargo clippy --package <crate> --all-targets --locked -- -D warnings
PROPTEST_CASES=32 cargo nextest run --package <crate> --locked
```

Tickets `02`, `03` and `04` also owe `cargo test --workspace --doc --locked`: `orcvs`'s public
surface changes, and `Orcvs::render_frame`'s doctests read `rows()` directly
(`orcvs/src/app.rs:96-97, 128`). Ticket `06` owes it for the same reason. Tickets touching `.scratch/`
owe `node --test scripts/tests/roadmap.test.ts` and `node scripts/roadmap.ts > /dev/null`.

Deferred to CI, and named on the `Not run` line rather than left off: `mise run check_wasm` — no
ticket here is platform-conditional, adds a `cfg`, or adds a dependency, and the merge tier compiles
the WASM target. The persistence arm is not owed; no ticket touches the feature.

No benchmark is owed by any ticket and no ticket may claim a cost it has not measured. Ticket `04`
moves code between crates and is explicitly forbidden from claiming that makes anything faster or
slower; the measurement question belongs to `render-frame-derivation`.
