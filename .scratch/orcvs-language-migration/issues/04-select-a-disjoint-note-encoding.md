# 04 — Confirm contextual Number and Note literals

**What to decide:** Confirm how the same two literal Source characters receive exactly one Number
or Note type from the containing Expression and its Function signature. Preserve the two-Cell Note
spelling from `C/` through `G9`, give Number Range and Note Range distinct names, and keep Comments
on a complete two-Cell introducer.

**Blocked by:** None — requires a syntax decision before implementation.

**Status:** resolved

- [x] Every MIDI Note from `00` through `7F` has one contextual two-Cell spelling from `C/` through `G9`.
- [x] A literal occurrence receives exactly one Atom type from its Expression and fixed Function signature.
- [x] `.v` and `.^` are the sole type-directed exception and always produce their named target type.
- [x] Number Range `:-` and Note Range `:#` have distinct monomorphic signatures; mixed bounds diagnose.
- [x] Comments begin with `##`; one `#` is incomplete or invalid Source.
- [x] `Display`, parsing, Glyph classification, Portal round-trips, Sequences, and Live Edits follow contextual typing.
- [x] A syntax prototype demonstrates the complete Note mapping and context-dependent classification.

## Comments

The original ticket incorrectly extrapolated ADR 0021 into a requirement that raw characters carry
type independently of their operand slot. The intended contract is contextual: `C4` is Number 196
for a Number operand and Note 60 for a Note operand. Source always stores literal characters; a later
consumer interprets them through its own fixed signature. ADR 0021 now states that distinction
explicitly.

Number Range remains `:-`; Note Range is `:#`. This avoids semantic overloading while retaining
type-preserving generic structural Sequence Functions. Moving the Comment introducer from `#` to
`##` makes `:#` unambiguous and follows the two-Cell Source form.

The replacement prototype is `lang/contextual-number-note-prototype.html`. The superseded
`lang/number-note-ambiguity-prototype.html` remains as the exploration that exposed the wording
problem.
