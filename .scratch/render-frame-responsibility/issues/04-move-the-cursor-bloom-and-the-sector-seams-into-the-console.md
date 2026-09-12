# 04 — Move the Cursor bloom and the sector seams into the console

**What to build:** `cursor_bloom`, `signal_breakup`, `cell_hash`, `sector_seam_strength`,
`classify_cursor_bloom`, `SECTOR_SEAM_STRENGTHS` and the `CursorBloom` enum leave
`orcvs/src/render_frame.rs` and arrive in `console`. `RenderFrame` is left answering `grid()`,
`cursor()`, `cursor_visible()` and `at(Position) -> &RenderCell` over a flat `Vec` of Cells carrying
content and Glyph.

## What is being moved, and why it is not the model's

All of it is in `orcvs/src/render_frame.rs:141-209`:

- `cursor_bloom` (`:141-156`) — a Chebyshev-distance band classifier. Its own comment explains the
  choice of metric in visual terms: "A Cartesian distance produces a Cell-aligned focus matrix
  instead of a radial pool of light."
- `signal_breakup` (`:158-166`) — perturbs that distance by one Cell depending on a hash, so the
  bands do not read as clean rings.
- `cell_hash` (`:168-175`) — the hash, a 32-bit mix on the Position.
- `SECTOR_SEAM_STRENGTHS: [u8; 4] = [100, 72, 34, 13]` (`:177`) — four alpha percentages.
- `sector_seam_strength` (`:179-194`) — a four-arm falloff that picks one of them, then drops the two
  faintest when `cell_hash(position).is_multiple_of(4)`, under a comment that reads "Preserve a
  legible four-arm registration mark, then let only the phosphor-faint middle between corners acquire
  stable gaps from the absolute Cell address."
- `classify_cursor_bloom` (`:196-209`) — sorts a distance into `Core`/`Inner`/`Mid`/`Outer` on band
  widths its comment states as "1:1:2:3, with cumulative radii 1:2:4:7".

Phosphor. Pool of light. Registration mark. Those comments are not describing a Source; they are
describing a CRT. The functions decide the appearance of the Grid down to the opacity ramp, and the
console's entire contribution is a `match` from four band names to eight palette colours
(`console/src/style.rs:106-113`) and one multiply (`console/src/style.rs:115-119`).

`CursorBloom` is also a public type of the core crate (`orcvs/src/render_frame.rs:15`) with no
`CONTEXT.md` entry and no ADR behind it — `bloom` appears nowhere in either source of truth. It got
there because it does not name `Color32`, which is the fault ticket `01` records.

## What must be decided before this can be built

**Where the spacings live.** `RenderFrame::derive` takes a `RenderFrameConfig`
(`orcvs/src/render_frame.rs:9-12`) carrying `marker_spacing` and `highlight_dot_spacing`, and
`Orcvs::render_frame` fills it from `self.opts` (`orcvs/src/app.rs:223-224`). Move both consumers out
and that struct has no fields left. But `console` cannot read them today: `Orcvs::opts` is a private
field with no accessor but `bpm()`/`set_bpm()` (`orcvs/src/app.rs:64, 163, 173`), and `console`
contains no `opts.` read at all. So one of two things has to happen, and this ticket must not pick
silently:

1. `RenderFrame` answers `marker_spacing()` and `highlight_dot_spacing()`, leaving the options in
   `orcvs` and handing the numbers across; or
2. the presentation fields of `Opts` move to `console` with the code that reads them, leaving `Opts`
   holding `bpm`, `cursor_delay` and `mode`.

(2) is the answer the criterion in ticket `01` implies — `Opts`'s own doc comment opens "How the
console presents and plays a Source" (`orcvs/src/opts.rs:12-17`) — and it is the larger change,
because it reaches persistence and the menu. (1) is smaller and leaves a smaller version of the same
drift in place. **Ticket `01` should settle this; this ticket implements whichever it settles.**

A neighbouring effort has already reached the same two fields from the other side.
`retired-glyph-vocabulary/05` (`ready-for-agent`, blocked by its `01`) renames `marker_spacing` to
the sector seam's period and `highlight_dot_spacing` to the Cursor bloom's radius, and renames
`MarkerSpacing`, `HighlightSpacing`, both `DEFAULT_*` constants and `RenderFrameConfig`'s fields with
them — **leaving them in `orcvs`**. That is orthogonal rather than conflicting: it fixes what the
fields are called, this ticket asks which crate they belong to. Whichever lands first, the other
takes the result; if `retired-glyph-vocabulary/05` has landed, use its names throughout and do not
reopen them.

**Do not decide the Marker question here.** `MarkerSpacing` still carries the name of the `+` Marker
Glyphs the seams replaced, and `CONTEXT.md:263-265` still defines **Marker**.
`retired-glyph-vocabulary` owns all of that — its `02` retires the glossary entry, its `03` deletes
the unreachable `Glyph` variants, its `05` renames the field. Move the seam code; leave the Marker
vocabulary to the effort that is already holding it.

## How this sequences with `render-frame-derivation`

`.scratch/render-frame-derivation/issues/02-stop-paying-for-the-bloom-at-every-cell.md`
(`needs-triage`, blocked by its `01`, which is `ready-for-agent`) owns **what the bloom costs**. This
ticket owns **where the bloom lives**. Neither subsumes the other and this ticket does not duplicate
it — but they collide, because moving the code changes which crate pays for it.

The collision is concrete. `[[bench]]` appears only in `orcvs/Cargo.toml:67` and
`lang/Cargo.toml:35`; there is no `console` benchmark. The only harness that can see the bloom is the
`source_render_frame` group (`orcvs/benches/source.rs:284-295`), which calls `orcvs.render_frame()`
over 16x16, 32x32 and 64x64. After this ticket, that group no longer executes a single line of bloom
code, and `render-frame-derivation/02`'s acceptance bar — "the win is stated as a measured difference
against issue 01's baseline through `source_render_frame`" — becomes unmeetable.

So the order is forced rather than preferred:

1. `render-frame-derivation/01` measures the baseline while the bloom is still inside
   `RenderFrame::derive`.
2. `render-frame-derivation/02` is decided on that number: it either lands the early-out or closes as
   too small to measure.
3. This ticket moves whatever is then there, unchanged.

This ticket therefore blocks on `render-frame-derivation/01` and **not** on its `02`, because `02`
may legitimately close without a change. Whoever works this ticket must check `02`'s state first and,
if it is still open, carry its early-out across with the rest rather than reverting it. Say so in the
commit message.

**This ticket claims nothing about cost, in either direction.** `CLAUDE.md` requires a benchmark or
profile for any claim about performance, and this change is a relocation: the same arithmetic runs
per Cell, in a different crate. Do not write "no slower" in the commit message either.

**Blocked by:** 01, 02, 03, render-frame-derivation/01, retired-glyph-vocabulary/01

`01` settles the criterion and the spacing question. `02` and `03` settle what `RenderFrame` looks
like, and this ticket is the one that finishes that shape. `render-frame-derivation/01` measures
while there is still something to measure. `retired-glyph-vocabulary/01` settles what the two
concepts are called, so the code that arrives in `console` arrives named after the glossary rather
than after a `render_frame.rs` identifier that predates it.

**Status:** needs-triage

- [ ] `orcvs/src/render_frame.rs` contains no bloom, no seam strength, no hash, and no alpha
      percentage. `CursorBloom` is not exported by `orcvs`.
- [ ] The moved functions arrive in `console` with their comments intact. The phosphor and
      registration-mark comments are the record of why the numbers are what they are and must not be
      dropped in transit.
- [ ] `RenderCell` carries the Cell's content and its Glyph, and nothing about how either is shown.
      `selected`, `cursor_visible`, `cursor_bloom`, `sector_left_strength` and
      `sector_top_strength` are gone from it; `RenderFrame::cursor()` and
      `RenderFrame::cursor_visible()` answer what the first two were for.
- [ ] `Paint::derive` computes the bloom and the seams itself from `frame.grid()`, `frame.cursor()`
      and `frame.cursor_visible()`. It still takes a `&RenderFrame` and nothing else — no `Orcvs`,
      no `egui::Context` — which is the decision `source-paint` turns on and this ticket must not
      weaken.
- [ ] Every test that moves, moves. The bloom band tests
      (`orcvs/src/render_frame.rs:402` and the seam tests around `:660-690` in `orcvs/src/app.rs`)
      assert appearance and belong beside the code that decides it. Their assertions do not change,
      only where they live.
- [ ] `CONTEXT.md` is **not** edited by this ticket, and the **Cursor Bloom** and **Sector Seam**
      entries are not written here. `retired-glyph-vocabulary/01` already owns both, specified down
      to their `_Avoid_` lists, and this ticket blocks on it so the moved code can be named after the
      words it settles. This follows `source-paint/01`'s handling of the **Console** entry that
      `console-testing/02` owned: block, do not duplicate.
- [ ] Nothing about what is drawn changes. Same colours, same geometry, same order, both blink
      phases.
- [ ] The commit message states that no cost claim is made and why.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --package orcvs --all-targets --locked -- -D warnings
cargo clippy --package console --all-targets --locked -- -D warnings
PROPTEST_CASES=32 cargo nextest run --package orcvs --locked
PROPTEST_CASES=32 cargo nextest run --package console --locked
cargo test --workspace --doc --locked
```

The doctest run is owed: `orcvs`'s public surface loses a type. No benchmark is owed and none may be
cited — see above.

## Comments

`needs-triage`. It is the largest change in the effort, it removes a public type from the core crate,
it has an undecided sub-question (where the spacings live) that ticket `01` must answer first, and it
has to be sequenced against another effort's measurement. None of that is an agent's call.
