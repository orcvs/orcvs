# 04 — Language Map row partition

**What to build:** Encode the partition law from ADR 0018 and CONTEXT.md. Generate Source contents
over a Grid, build the Language Map, and check that the recognised units partition each row.

**Blocked by:** None — every listed blocker is resolved.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] No two Expression Spans overlap.
- [ ] No Expression Span crosses a row boundary.
- [ ] Every classified Cell receives a Function, Note, Number, Bang or Char Glyph, and every
      other Cell answers `None`. The Language Map assigns no Marker, Highlight or Space Glyph:
      the first two come from the UI helpers and Space is the render frame's fallback.
- [ ] Every Span start and end is a valid index for the Grid.
- [ ] Building the map twice from the same bytes gives the same result. The existing
      `a_rebuilt_map_equals_the_map_a_full_build_would_have_made` property in
      `orcvs/src/source/language_map.rs` already proves the stronger incremental-versus-full
      equality; reconcile against it rather than restating it.
- [ ] The drift this line was written for is now carried by `LanguageMap::rebuild`.
      `prospective_expression_range` was deleted in commit `28c5a85` and no longer exists, so the
      obligation is to cover `rebuild` against a full build, which the property above does.
- [ ] The generator produces `***`, `<<<`, and `^^^^`, which ADR 0018 names explicitly.

## Comments

The sentence being encoded is the one clarified in commit 8183720: "It partitions each row from left
to right into non-overlapping complete Language Units: after recognizing a unit it resumes after that
complete Span, and an unmatched character diagnoses without participating in an overlapping
unit."

ADR 0018 gives the worked cases: `***` is Bang `**` then one invalid `*`; `<<<` is `<<` then one
invalid `<`; `^^^^` is two Self-Banging Functions. Put them in the generator as literals, and let the
random input find the rest.

`prospective_expression_range` claims to answer without scanning any other row. That claim is a
property: its answer must equal the answer a full rebuild gives. It is the one most likely to drift
during the language migration.

These tests reach `pub(super)` items, so they must stay inline in `language_map.rs`.
