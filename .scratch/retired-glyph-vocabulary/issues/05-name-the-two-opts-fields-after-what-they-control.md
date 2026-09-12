# 05 — Name the two Opts fields after what they now control

**What to fix:** `Opts::marker_spacing` is the sector seam period and `Opts::highlight_dot_spacing`
is the Cursor bloom radius. Both are named for concepts the console no longer draws, and the doc
comment on the struct states the false one outright.

**Blocked by:** 01 — Name the Sector Seam and the Cursor Bloom in the glossary.

**Status:** ready-for-agent

- [ ] `Opts` (`orcvs/src/opts.rs:18-25`) names the two fields after the concepts they configure,
      using the glossary terms `01` settles — `sector_seam_spacing` and `cursor_bloom_radius` unless
      `01` lands different words, in which case the field names follow `01`.
- [ ] `MarkerSpacing` (`orcvs/src/opts.rs:37`) and `HighlightSpacing` (`orcvs/src/opts.rs:40`) are
      renamed with their fields. The newtypes carry the same misnomer and renaming the field alone
      leaves `sector_seam_spacing: MarkerSpacing`, which is worse than either name on its own.
- [ ] `DEFAULT_MARKER_SPACING` and `DEFAULT_HIGHLIGHT_DOT_SPACING` (`orcvs/src/opts.rs:4`, `:6`)
      are renamed to match, keeping the values `8` and `7` exactly.
- [ ] The struct doc comment at `orcvs/src/opts.rs:12-16` no longer says `marker_spacing` "counts
      the Cells between visual markers". It says what the field now counts. The sentence it sits in —
      that nothing in the file is a Source dimension — is preserved, because that is the point the
      comment is actually making and it is still true.
- [ ] `RenderFrameConfig` (`orcvs/src/render_frame.rs:9-12`) renames both fields with them, and
      `orcvs/src/app.rs:222-225` follows.
- [ ] The local at `orcvs/src/render_frame.rs:151` and the parameters named `radius` through
      `signal_breakup` and `classify_cursor_bloom` are left alone or aligned deliberately — they are
      already truthful, which is half the evidence that the field name is not.
- [ ] The three test names that already describe the real concept keep describing it:
      `default_cursor_field_reaches_seven_cells_from_the_cursor` (`orcvs/src/opts.rs:111`),
      `marker_spacing_accepts_only_whole_positive_cell_counts` (`orcvs/src/opts.rs:120`) and
      `highlight_spacing_accepts_only_whole_positive_cell_counts` (`orcvs/src/opts.rs:127`). The
      first is already right and must not be renamed backwards; the second and third are named for
      the old fields and are renamed with them.
- [ ] `test_sector_edges_use_one_whole_cell_spacing_without_marker_glyphs`
      (`orcvs/src/app.rs:657`) keeps its assertions unchanged. Its "without marker glyphs" clause
      may be dropped once `03` has landed, since there will be no marker Glyph to be without.
- [ ] No default, no computed value and no rendered pixel changes. This is a rename.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package orcvs --all-targets --locked -- -D warnings`,
      `cargo clippy --package console --all-targets --locked -- -D warnings`,
      `PROPTEST_CASES=32 cargo nextest run` on both packages, and
      `cargo test --workspace --doc --locked` pass.

## Comments

**What the two fields actually control.**

`marker_spacing` is read once, at `orcvs/src/render_frame.rs:88`, and spent on nothing but the two
sector-seam decisions immediately below it:

```rust
sector_left_strength: (position.x() > 0
    && position.x().is_multiple_of(marker_spacing))
.then(|| sector_seam_strength(position.y(), marker_spacing, position))
```

and the transposed pair for `sector_top_strength` at `:101-104`. There is no other reader in the
workspace.

`highlight_dot_spacing` is read once, at `orcvs/src/render_frame.rs:151`:

```rust
let radius = config.highlight_dot_spacing.cells();
```

and handed to `classify_cursor_bloom` (`orcvs/src/render_frame.rs:196-211`), where it is the
cumulative radius of the outermost bloom band — `distance <= radius` is the last arm that returns
`Some(CursorBloom::Outer)`. It is a radius in Cells, not a spacing between dots, and there is no
dot.

**The code already knows, which is the strongest evidence available.** `orcvs/src/opts.rs:111`
names its test `default_cursor_field_reaches_seven_cells_from_the_cursor` and then asserts on
`highlight_dot_spacing`. `orcvs/src/app.rs:657` names its test
`test_sector_edges_use_one_whole_cell_spacing_without_marker_glyphs` and then sets
`app.opts.marker_spacing`. Two tests reach for the nearest surviving field to address a concept
they can already name correctly in prose. That is what "configured by reaching for the nearest
surviving field" looks like from the inside.

**One ticket, not two.** The two renames touch the same three files and, in `orcvs/src/opts.rs` and
`orcvs/src/render_frame.rs`, adjacent lines of the same structs and the same test literals —
`RenderFrameConfig` is constructed with both fields at seven places in `render_frame.rs`'s tests
(`:262`, `:299`, `:326`, `:350`, `:373`, `:493`, `:546`) and once in shipped code at
`orcvs/src/app.rs:222`. Splitting them would put two mechanical renames in conflict over every one of
those construction sites for no gain in reviewability; there is no ordering between them and no
decision either one makes that the other does not. They are one change.

**Blast radius, counted.** Every mention of either name in the workspace is inside `orcvs`:

| File | `marker` family | `highlight` family |
| --- | --- | --- |
| `orcvs/src/opts.rs` | 11 lines | 11 lines |
| `orcvs/src/render_frame.rs` | 15 lines | 11 lines |
| `orcvs/src/app.rs` | 6 lines | 1 line |

`console`, `lang`, the benches and the WASM tests contain no occurrence of `marker_spacing`,
`MarkerSpacing`, `highlight_dot_spacing` or `HighlightSpacing`. The majority of the lines above are
test literals — `MarkerSpacing::new(8).unwrap()` and its siblings.

**Serialisation risk: none. This was checked, not assumed.** `Opts` derives only `Clone, Debug`
(`orcvs/src/opts.rs:18`) and neither `Serialize` nor `Deserialize`, under any `cfg`. The
`persistence` feature is `persistence = ["dep:serde"]` (`orcvs/Cargo.toml:30`) and the only types
behind it are `Grid` (`orcvs/src/grid.rs:89-92`) and `Source` (`orcvs/src/source/model.rs:136-167`).
`console/src/persistence.rs:4` states it directly: "`Source` is the only supported persistence
root." So no stored file, no key and no schema names either field, and the rename cannot break a
restore. The persistence feature arm is therefore not owed by this ticket.

**Public-API risk: contained, and the repository contract sizes it.** `orcvs::opts` is a `pub mod`
(`orcvs/src/lib.rs:9`) and both fields are `pub`, so this is a public-surface change to the `orcvs`
crate. `CLAUDE.md` says neither workspace crate is publishable, and the sole consumer is `console`,
which does not reference either field — it imports only `Bpm` and `DEFAULT_FONT_SIZE` from that
module (`console/src/console.rs:18`). `cargo test --workspace --doc --locked` is owed because the
public surface moves; nothing else is.

**Who else names these fields.** A grep across `.scratch/` finds `marker_spacing` and
`highlight_dot_spacing` in four files. Three are settled or unrelated:
`source-playback-engine/07`, resolved, which is the ticket that *gave* them these names to free the
word "grid"; `source-playback-engine/17`, about measuring the spacing in whole Cells; and
`restyle-egui-console/02`, in a comment tracing where `marker_spacing` is spent. `typed-source-paint`
— which owns the rest of the Glyph retirement — does not mention `Opts` at all.

The fourth is
`.scratch/render-frame-responsibility/issues/04-move-the-cursor-bloom-and-the-sector-seams-into-the-console.md`,
which was being filed concurrently by another agent and which this effort has deliberately not read.
Its path and title are all that is known here, and both say it is working on the same two concepts.
**Read it before starting.** If it proposes moving the seam and bloom computation into `console`,
this rename either lands first and that ticket carries the new names, or lands after and renames
whatever fields survive the move — but it must not be done blind, because a rename and a move of the
same fields will conflict in every one of the files counted below.
This ticket is the only one in this effort with no overlap of any kind.

**One line of prose falls with the rename.** `console/src/theme.md:41-42` says the seams preserve
"the configured Marker spacing as geometry". That sentence is otherwise the best record in the
repository of why the Marker was retired, so update the phrase rather than deleting the paragraph —
and do it here, in the change that makes it wrong, not in `04`.

**Why `ready-for-agent`.** Every call site is enumerated, the compiler catches any that is not, and
the serialisation and public-API questions are answered rather than flagged rather than left for an
implementer to discover. The two open questions — what the concepts are called, and whether
`render-frame-responsibility` is about to move the code — both sit behind `01`, which blocks this
and is itself `needs-triage` precisely so that the cross-effort check happens before either ticket
is picked up. Nothing is left for the implementer of this ticket to decide.
