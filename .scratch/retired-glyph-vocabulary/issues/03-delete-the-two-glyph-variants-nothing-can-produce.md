# 03 — Delete the two Glyph variants nothing can produce

**What to fix:** `Glyph::Marker` and `Glyph::Highlight` cannot occur on any path, and four files
carry code that exists only to handle them.

**Blocked by:** 02 — Retire the Marker entry from the glossary.

**Status:** needs-triage

**Triage owes one decision, stated before the acceptance bars because it may empty them:**
`typed-source-paint/01` is `ready-for-agent` with no unmet blockers and deletes `Glyph` outright,
naming `GlyphString::marker()`, `GlyphString::highlight()`, `From<Token> for Glyph`, the `Display`
spelling table, `BLANK_GLYPHS` and `blank_glyph_index` in its own acceptance lines. Every bar below
is a strict subset of that ticket. Take this one only if the larger change is not going to land
soon; otherwise mark this `wontfix` and note in `typed-source-paint/01` that it absorbs it. Do not
do both.

- [ ] `Glyph` (`orcvs/src/glyph.rs:38-49`) has seven variants: Bang, Char, Comment, Function, Note,
      Number, Space.
- [ ] `GlyphString::marker()` (`orcvs/src/glyph.rs:16-21`) and `GlyphString::highlight()`
      (`orcvs/src/glyph.rs:23-28`) are gone. `GlyphString::space()` and `GlyphString::new` stay.
- [ ] The `Display` arms for the two variants (`orcvs/src/glyph.rs:90-91`) are gone, so `"+"` and
      `"."` are no longer spellings any empty Cell can take.
- [ ] `console/src/style.rs:79-80` no longer maps either variant, and `cell_visuals` still matches
      `Glyph` exhaustively with no wildcard arm.
- [ ] `BLANK_GLYPHS` (`console/src/paint.rs:260-270`) is seven entries and `blank_glyph_index`
      (`console/src/paint.rs:278-287`) is a seven-arm exhaustive match whose indices still agree with
      the table's order.
- [ ] The four doc comments that state the table's size in words say seven:
      `console/src/paint.rs:100` ("read once for the nine blank spellings"), `:296` ("once per
      Render Frame for the nine Glyphs"), `:321` ("the nine blank spellings are read once") and
      `:323` ("an array of nine `char`s").
- [ ] `background_glyphs_remain_distinct` (`orcvs/src/glyph.rs:117-132`) and
      `local_highlights_are_distinct_from_global_markers` (`console/src/style.rs:244-252`) are gone,
      not weakened into assertions about the remaining variants.
- [ ] The two spelling assertions at `console/src/paint.rs:617-618` are gone; the surrounding test
      `a_blank_cell_shows_what_its_glyph_spells` keeps its loop over `BLANK_GLYPHS` and its
      `Glyph::Space` assertion at `:619`.
- [ ] Nothing about what appears on screen changes. No Cell painted before this change is painted
      differently after it.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package orcvs --all-targets --locked -- -D warnings`,
      `cargo clippy --package console --all-targets --locked -- -D warnings`, and
      `PROPTEST_CASES=32 cargo nextest run` on both packages pass, plus
      `cargo test --workspace --doc --locked`.

## Comments

**The unreachability was proved, not assumed.** A `Glyph` reaches a `RenderCell` at exactly one
place: `orcvs/src/render_frame.rs:92-95`,
`source.language_map().glyph_at(position).unwrap_or(Glyph::Space)`. `glyph_at`
(`orcvs/src/source/language_map.rs:281`) reads the `glyphs` array
(`orcvs/src/source/language_map.rs:318`), which is initialised to `None` (`:356`), cleared to `None`
(`:368`), and written at exactly two places:

```rust
row.glyphs[cell - row_start] = Some(Glyph::from(entry.token));   // :388
row.glyphs[column] = Some(Glyph::Char);                          // :404
```

`From<Token>` at `orcvs/src/glyph.rs:53-73` maps Bang, Comment, Function, Note, Number and Char each
to its own variant and `Atom | Sequence` to `Char`. There is no `Marker` arm and no `Highlight` arm,
and `Token` has no variant that could reach one. So no Source, valid or invalid, can produce either
Glyph.

A workspace grep for `Glyph::Marker`, `Glyph::Highlight`, `G::Marker`, `G::Highlight`,
`GlyphString::marker` and `GlyphString::highlight` across every `.rs` file outside `target/` returns
eleven lines, and every one of them is in this ticket: the two constructors, the two enum variants,
the two `Display` arms, the two calls and two assertions inside
`background_glyphs_remain_distinct`, and the two exhaustive matches in `console`.

**Why it is one ticket and not three.** Removing a variant breaks both exhaustive matches in the
same compilation, and `console/src/style.rs:244-252` and `console/src/paint.rs:617-618` stop
compiling with it. Every commit is expected to pass its own gates, so the enum, the two matches and
the three tests move together or not at all. The parts that *can* be separated are separated: the
palette tokens go to `04` for a reason stated there.

**One consequence worth stating plainly.** `blank_character` (`console/src/paint.rs:299-307`) reads
`GlyphString::new(None, glyph).to_string()` for every entry of `BLANK_GLYPHS`, once per
`CellCharacters::new()` (`console/src/paint.rs:333-337`), so the `Display` arms for Marker and
Highlight *do* execute today — they are not dead statements, they are live statements computing
entries of a lookup table that no Cell can ever index. That is why
`console/src/paint.rs:617-618` passes: it asserts the table's content, not any Cell's appearance.
Deleting the arms deletes two table entries nothing reads.

**Why this is blocked by `02` rather than independent.** `CLAUDE.md` makes `CONTEXT.md` the source
of truth for vocabulary. Deleting the variants while the glossary still describes a Marker Glyph
would put the code ahead of its own source of truth, in the direction that is hardest to detect
later.

**Note for whichever ticket wins.** `BLANK_GLYPHS`, `blank_glyph_index`, `blank_character` and
`CellCharacters` live in `console/src/paint.rs`. They moved there from `console/src/console.rs` with
the `source-paint` work, landing on this branch in `df3d2e4`, and every line number above is read
from that tree rather than from `main`. `typed-source-paint/01`'s acceptance line still says "`console`'s
`BLANK_GLYPHS`, `blank_glyph_index` and `GlyphTable::blanks` are deleted, and `GlyphTable::character`
collapses to the Cell's content" — `GlyphTable` no longer holds them, `CellCharacters` does. That
ticket's line needs the same correction whether or not this one is taken.
