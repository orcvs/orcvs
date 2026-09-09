# 01 — Share the Expressions a rebuild carries forward

**What to build:** A Language Map rebuild costs what the edited row costs, not what the whole Source
holds, so a keystroke stops paying to copy the rows it did not touch.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] A rebuild no longer deep-clones the `ExpressionEntry` values of rows it did not re-parse.
      Sharing them — `Arc`, or a representation an unchanged row can be carried by reference — is
      the shape; which one is the ticket's design decision and is recorded here.
- [ ] The allocation cost of one Cell write stops scaling with the number of Expressions the Map
      carries. `a_language_map_rebuild_grows_with_the_expressions_it_carries_and_no_faster` in
      `orcvs/tests/allocation.rs` reports the number; it is a ratio assertion and passes either way,
      so the measured before-and-after counts are recorded here rather than asserted.
- [ ] Editing a Cell in an empty margin row costs measurably less than editing one inside an
      Expression. Today it costs almost the same, which is the clearest statement of the defect.
- [ ] `LanguageMap` stays a value the Source can hand out as `Arc<LanguageMap>` from
      `shared_language_map`, and `SourceRevision` keeps sharing one Map across unchanged reads —
      `unchanged_revision_reads_share_the_language_map` pins that and must keep passing.
- [ ] Every existing `orcvs` test and property passes unchanged, including the Language Map suite in
      `orcvs/tests/language_map.rs`.
- [ ] `source_edit_rebuild_valid` and `source_edit_rebuild_invalid` are the benchmarks that cover
      this path. Note them for the comparison the benchmark workflow runs; do not run the comparison
      locally.

## Comments

This is the largest of the three findings by a wide margin, and the only one whose fix is a design
change rather than a local edit.

What `memory-verification/02` measured, on the `orcvs` bench fixtures:

```
43 Expressions   ->  79 blocks,  47,971 bytes
160 Expressions  -> 247 blocks, 173,913 bytes
621 Expressions  -> 928 blocks, 658,728 bytes
```

About 1.45 blocks and a kilobyte for each Expression the Map carries, flat across Grid sizes and
driven by the Expression count rather than the Cell count. A fixed 32x32 Grid gives the same slope:
29 blocks at 5 Expressions, 244 at 160.

`LanguageMap::rebuild` re-parses only the dirty row and carries every other row forward with
`..entry.clone()`. An `ExpressionEntry` owns an `Atoms` — a `Vec<Atom>` — an `Expression`, and an
optional `Diagnostic` holding a `String`. So the row-level incrementality that already avoids
re-parsing does not avoid re-allocating, and the giveaway is that editing a Cell in an empty margin
row costs 229 blocks against 247 for editing one inside an Expression: nearly all of a keystroke's
allocation is rows the keystroke did not touch.

The rebuild runs on every keystroke and on every Tick that writes a Cell, so this is paid at typing
speed and at Tick speed both.

One thing to be careful of: the bytes half of a rebuild is legitimately Grid-sized — one byte per
Cell plus about 136 bytes per row for the `glyphs` vector, the three `row_runs` vectors, `walks` and
`row_units`. A rebuild that produces a whole Map cannot be smaller than the Map. That part is not
the defect and must not be optimised away by making the Map lazy; the defect is the per-Expression
carry on top of it.
