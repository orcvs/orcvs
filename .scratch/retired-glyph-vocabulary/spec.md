# Retire the vocabulary of the presentation that was replaced

**Status:** needs-triage

## Goal

Two visual concepts were replaced and neither was retired. The `+` Marker Glyphs became the
sector seams; the `.` Highlight dots became the Cursor bloom. `console/src/theme.md:41-42` states
the first replacement outright — the seams "replace the historical `+` Marker Glyphs, leaving
every empty Cell visually empty while preserving the configured Marker spacing as geometry" — and
the shipped code agrees. What did not happen is the other half of a replacement: the `Glyph`
variants, the glossary entry, the palette tokens, the blank-spelling table and the two `Opts`
fields all still speak the retired vocabulary, and the concepts that took over have no names at
all.

This effort finishes the replacement. It names the two surviving concepts, deletes the artefacts
of the two dead ones, and renames the two configuration fields that now control the survivors.

## What is actually dead

`Glyph::Marker` and `Glyph::Highlight` cannot occur. A `Glyph` reaches a Render Frame from exactly
one place — `orcvs/src/render_frame.rs:92-95` reads `source.language_map().glyph_at(position)` and
falls back to `Glyph::Space` — and a Language Map writes its `glyphs` array at exactly two places:
`orcvs/src/source/language_map.rs:388` stores `Glyph::from(entry.token)`, and
`orcvs/src/source/language_map.rs:404` backfills `Glyph::Char` into every non-empty unclaimed Cell.
`From<Token>` (`orcvs/src/glyph.rs:53-73`) has six arms plus an `Atom | Sequence => G::Char` arm and
no `Marker` or `Highlight` arm at all. The two variants are therefore unreachable from any Source.
The only construction sites in the whole workspace are `GlyphString::marker()`
(`orcvs/src/glyph.rs:16-21`), `GlyphString::highlight()` (`orcvs/src/glyph.rs:23-28`), and the test
at `orcvs/src/glyph.rs:118-132` that calls both. Nothing outside `orcvs/src/glyph.rs` names either
variant except the two exhaustive matches that are forced to mention them:
`console/src/style.rs:79-80` and `console/src/paint.rs:284-285`.

## What is alive and unnamed

The replacements ship and work. `sector_seam_strength` (`orcvs/src/render_frame.rs:179-194`) grades
a four-armed registration mark along each sector boundary from `SECTOR_SEAM_STRENGTHS`
(`orcvs/src/render_frame.rs:177`); `cursor_bloom` and `classify_cursor_bloom`
(`orcvs/src/render_frame.rs:141-156`, `:196-211`) grade four bands out from the Cursor. `CursorBloom`
is a public type (`orcvs/src/render_frame.rs:14-20`). `console/src/theme.md:26-42` describes both in
detail.

Neither is in the glossary. `CONTEXT.md` contains no occurrence of "bloom" at all, and the only
occurrence of "seam" in a console sense is inside the **Paint** entry at `CONTEXT.md:272`, which
lists "sector seams" among the things a Paint decides without that term being defined anywhere.
No ADR in `docs/adr/` records either concept. Meanwhile `CONTEXT.md:263-265` still defines **Marker**
as "a purely visual Glyph the console draws at every marker-spacing interval of Cells in both axes",
which nothing draws, and `CONTEXT.md:260` still lists Marker and Highlight as two of the three
Glyphs a Cell the Source has not parsed can carry.

`CLAUDE.md` names `CONTEXT.md` a source of truth for vocabulary and `docs/agents/domain.md` says
output naming a domain concept must use the glossary's term. A concept cannot be retired in favour
of one that was never defined, which is why ticket `01` comes first and everything else hangs off
it.

## The one that lies

`Opts::marker_spacing` and `Opts::highlight_dot_spacing` (`orcvs/src/opts.rs:22-23`) are the only
configuration these two live concepts have, and both are named for the dead ones.
`orcvs/src/render_frame.rs:88` reads `config.marker_spacing.cells()` and spends it on nothing but
the two `sector_seam_strength` calls at `:99` and `:103`. `orcvs/src/render_frame.rs:151` reads
`config.highlight_dot_spacing.cells()` straight into a local named `radius` and passes it to
`classify_cursor_bloom`, where it is the cumulative radius of the outermost bloom band. The doc
comment at `orcvs/src/opts.rs:12-16` still tells a reader that `marker_spacing` "counts the Cells
between visual markers".

The code already knows. `orcvs/src/opts.rs:111` names its test
`default_cursor_field_reaches_seven_cells_from_the_cursor` while asserting on
`highlight_dot_spacing`, and `orcvs/src/app.rs:657` names its test
`test_sector_edges_use_one_whole_cell_spacing_without_marker_glyphs` while setting `marker_spacing`.
Two tests describe the real concept in their names and then reach for the nearest surviving field to
address it. That is the shape of the defect: nothing was renamed, so each new concept was configured
through whichever old field happened to still be there.

## How it got this way

Nobody was careless. `source-playback-engine/07` — resolved — deliberately renamed these settings
*onto* marker vocabulary, to free the word "grid" for Cell addressing, and its acceptance line reads
"rendering is unchanged: markers, highlights and cursor blocks appear exactly where they did
before". `source-playback-engine/16` — also resolved — found that `GlyphString::highlight()` and
`GlyphString::space()` both constructed `Glyph::Marker` and made each construct its own variant, with
"pairwise constructor and display assertions" to keep them apart. That test is
`background_glyphs_remain_distinct` (`orcvs/src/glyph.rs:117-132`), and it was a correct test when it
was written.

Then the presentation changed. The markers became geometry and the highlight dots became the bloom,
and neither of those two settled tickets could reach forward to retire what it had just built. This
effort is the reach-forward.

## Scope

Five tickets:

- `01` names **Sector Seam** and **Cursor Bloom** in `CONTEXT.md`.
- `02` retires the **Marker** entry from the glossary.
- `03` deletes the two `Glyph` variants and their constructors, with the two exhaustive matches and
  the two tests that fall with them.
- `04` retires the `marker` and `highlight` palette tokens from `ConsolePalette`, `PALETTE` and
  `console/src/theme.md`.
- `05` renames the two `Opts` fields, their newtypes, their defaults and their doc comment.

Only `01`, `02` and `05` are certainly this effort's work. Read the next section before picking up
`03` or `04`.

## Overlap with typed-source-paint — read this before starting

**`typed-source-paint/01` already owns most of the code half of this effort, and it is
`ready-for-agent` with no unmet blockers.** That ticket deletes `orcvs::glyph::Glyph` entirely in
favour of `Option<Token>` on the `RenderCell`, and its acceptance lines name, by hand:
`GlyphString::marker()`, `GlyphString::highlight()`, `From<Token> for Glyph`, the blank spelling
table in `Display`, `console`'s `BLANK_GLYPHS` and `blank_glyph_index`, `PALETTE.marker`,
`PALETTE.highlight`, and the **Glyph** entry in `CONTEXT.md`. Its spec is already explicit about the
cause, at `typed-source-paint/spec.md:18-20`: "`Marker` and `Highlight` are Orca's terminal
inheritance — where the Grid's rulings are characters — and Orcvs draws both as geometry, so nothing
has produced either since."

So the framing that "not one artefact was retired" is true of the code and false of the tracker. The
retirement is scheduled; it is scheduled inside a larger change that has a parity risk of its own
("the parity claim is the whole risk", `typed-source-paint/01`). Tickets `03` and `04` here are the
narrow, parity-free version of the same deletions, and they exist so that the choice between the two
is made deliberately rather than by whichever agent reaches the files first. Both are `needs-triage`
for exactly that reason, and each states the decision triage owes.

**Three things in this effort are not covered by `typed-source-paint` anywhere.**

- The **Marker** glossary entry at `CONTEXT.md:263-265`. `typed-source-paint/01` removes the
  **Glyph** entry and no other, and a grep of that whole effort finds "Marker" only in its spec's
  comparison table and in a passing clause of its `02`. Ticket `02` here owns it.
- Glossary entries for the two survivors. `typed-source-paint/01` says the background rulings are
  "described where they are drawn" — which is `console/src/theme.md`, not the glossary. Ticket `01`
  here owns them.
- `Opts::marker_spacing` and `Opts::highlight_dot_spacing`. No ticket in any effort mentions either
  field. Ticket `05` here owns them, and it is the only ticket in this effort with no overlap of any
  kind.

## Not in scope

- The **Paint** entry at `CONTEXT.md:271-273`. It was written by `source-paint` and is correct; `01`
  may only add the definition of the term it already uses, never reword the entry itself.
- Any change to what appears on screen. Every ticket here is a naming or a deletion; the seams and
  the bloom render exactly as they do today, at the same defaults.
- `GlyphString::space()` (`orcvs/src/glyph.rs:30-35`), whose only callers are also tests
  (`orcvs/src/app.rs:639`, `:653`, `:748-750`). It stays. `Glyph::Space` is reachable — it is the
  fallback at `orcvs/src/render_frame.rs:95` — so its constructor is a convenience for a live
  variant rather than an artefact of a dead one. Do not fold it into `03`.
- The base16 block at `console/src/theme.md:44-87`, retained there as design context.
- The **Glyph** glossary entry at `CONTEXT.md:259-261`. `typed-source-paint/01` already carries an
  acceptance line removing it outright, and removing an entry and correcting one clause of it are
  not the same edit. Ticket `02` states what that entry's Marker/Highlight clause needs if
  `typed-source-paint/01` does not land first, and otherwise leaves it alone.
- Replacing `Glyph` with `Option<Token>`. That is the whole of `typed-source-paint`, it carries a
  parity risk this effort does not, and nothing here should pre-empt it.

## Overlap with render-frame-responsibility

`.scratch/render-frame-responsibility/` is being filed at the same time as this effort by another
agent and works on the code that computes both concepts. One of its ticket paths is
`issues/04-move-the-cursor-bloom-and-the-sector-seams-into-the-console.md`, and a grep of `.scratch/`
for field names shows that file names `marker_spacing` and `highlight_dot_spacing`. Its path and
title are the only things known about it here; its contents have not been read. So it may want to
define **Sector Seam** and **Cursor Bloom** itself, and it may move the very fields `05` renames.

This effort has not read that directory and must not write to it. Tickets `01` and `05` state the
collision and require whoever picks them up to read that directory first: the entries are
written once, in one change, by whichever effort reaches them, and the other effort's tickets then
reference them rather than restating them. This is the same treatment `source-paint/01` gave the
**Console** entry that `console-testing/02` already owned.

## Verification

Tickets `01`, `02` and `04`'s documentation half touch `.scratch/` and Markdown only; the tracker
gates apply:

```sh
node --test scripts/tests/roadmap.test.ts
node scripts/roadmap.ts > /dev/null
```

Tickets `03`, `04` and `05` touch Rust. On the crates each edits and their dependents — `orcvs`
means `orcvs` and `console`; `console` has no dependents:

```sh
cargo fmt --all -- --check
cargo clippy --package <crate> --all-targets --locked -- -D warnings
PROPTEST_CASES=32 cargo nextest run --package <crate> --locked
```

`03` and `05` also owe `cargo test --workspace --doc --locked`, because both change the public
surface of the `orcvs` crate (`orcvs::glyph` and `orcvs::opts` are both `pub mod` at
`orcvs/src/lib.rs:3-12`).

Deferred to CI, and named on the `Not run` line rather than left off: `mise run check_wasm`, since
nothing here is platform-conditional and no `cfg` or dependency changes. The persistence arm is not
owed by any ticket — see `05`, which establishes that `Opts` is not a persistence root.
