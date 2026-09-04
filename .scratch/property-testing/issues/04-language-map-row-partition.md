# 04 — Language Map row partition

**What to build:** Encode the partition law from ADR 0018 and CONTEXT.md. Generate Source contents
over a Grid, build the Language Map, and check that the recognised units partition each row.

**Blocked by:** 01 — Add proptest for native targets; language-map/03 — Move Source consumers behind the Language Map.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] No two Expression Spans overlap.
- [ ] No Expression Span crosses a row boundary.
- [ ] Every parsed Cell receives a Function, Note, Number, Bang or Char Glyph, and every unparsed
      Cell receives a Marker, Highlight or Space Glyph.
- [ ] Every Span start and end is a valid index for the Grid.
- [ ] Building the map twice from the same bytes gives the same result.
- [ ] `prospective_expression_range` agrees with a full rebuild after the same single-Cell edit.
- [ ] The generator produces `***`, `<<<`, and `^^^^`, which ADR 0018 names explicitly.

## Comments

The sentence being encoded is the one clarified in commit 8183720: "It partitions each row from left
to right into non-overlapping complete Language Units: after recognizing a unit it resumes after that
complete Footprint, and an unmatched character diagnoses without participating in an overlapping
unit."

ADR 0018 gives the worked cases: `***` is Bang `**` then one invalid `*`; `<<<` is `<<` then one
invalid `<`; `^^^^` is two Self-Banging Functions. Put them in the generator as literals, and let the
random input find the rest.

`prospective_expression_range` claims to answer without scanning any other row. That claim is a
property: its answer must equal the answer a full rebuild gives. It is the one most likely to drift
during the language migration.

These tests reach `pub(super)` items, so they must stay inline in `language_map.rs`.
