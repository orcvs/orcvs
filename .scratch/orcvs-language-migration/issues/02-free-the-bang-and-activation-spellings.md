# 02 — Free the `**` and `>>` spellings for Bang and directional activation

**What to build:** `**` denotes the Bang Atom and `>>` denotes the east Self-Banging Function, as
`CONTEXT.md` and ADR 0006 now define. Raw Play moves to `!>` per ADR 0016.

**Blocked by:** 01

**Status:** resolved

- [x] `**` is no longer parsed as Multiplication and `>>` is no longer parsed as Play.
- [x] `!>` parses as Raw Play and round-trips through `Display`.
- [x] `console/tests/wasm.rs` dispatches MIDI through `!>` rather than `>>`.
- [x] `console/src/glyph.rs` classifies Bang from the two-Cell Atom rather than a bare `*`.

## Comments

Until this lands, `**` and `>>` keep parsing as the pre-audit Functions, so Source written against
the documented language is misinterpreted silently rather than diagnosed. The `glyph.rs` criterion
supersedes the interim fix that merely excludes `Glyph::Function` from the `*` Bang test.
