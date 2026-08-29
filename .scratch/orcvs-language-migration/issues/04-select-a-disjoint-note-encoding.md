# 04 — Select a disjoint two-Cell Note encoding

**What to decide:** Choose the canonical two-Cell ASCII Source encoding for every Note from MIDI
`00` through `7F`. No Note spelling may also be a hexadecimal Number, Function, Bang, Activation
Character, or reserved Source form. The Language Map must classify the encoding without consulting
an operand slot or surrounding Function.

**Blocked by:** None — requires a syntax decision before implementation.

**Status:** needs-info

- [ ] Every MIDI note from `00` through `7F` has one canonical two-Cell encoding.
- [ ] Every Note encoding is disjoint from Numbers `00` through `FF` and every reserved Language Unit.
- [ ] Naturals and sharps remain readable enough for live musical editing.
- [ ] `Display`, parsing, Glyph classification, Portal round-trips, Sequences, and Live Edits use the same mapping.
- [ ] A syntax prototype demonstrates the complete mapping and collision-free classification.

## Comments

The retired natural-note spelling overlaps hexadecimal Numbers such as `A0`, `C4`, and `B7`.
ADR 0021 requires lexical type identity but deliberately does not invent the replacement spelling.
The Number/Note ambiguity prototype records the failure mode that this issue must eliminate.
