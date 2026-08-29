# 01 — Move arithmetic onto the `.` family

**What to build:** The arithmetic Functions use their audited `.`-family spellings. `lang/src/stack.rs`
maps `.+`, `.-`, `.x`, and `./` to Addition, Subtraction, Multiplication, and Division, and
`lang/src/atom.rs` renders them the same way, replacing `++`, `--`, `**`, and `//`.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] `.+`, `.-`, `.x`, and `./` parse to their Functions and round-trip through `Display`.
- [x] The retired `++`, `--`, `**`, and `//` spellings no longer parse as Functions.
- [x] Parser and interpreter tests, including the nested cases, use the new spellings.
- [x] `console/tests/wasm.rs` and any Source fixtures are updated.

## Comments

Canonical forms are given by ADR 0011 and the ADR 0019 capability index. Freeing `**` is a
prerequisite for issue 02, since Bang needs that spelling.

Implemented the four audited spellings in Function parsing and display, added rejection coverage
for every retired spelling, migrated nested/parser/interpreter and console Source fixtures, and
added a WASM playback fixture for `.+`. Verified with both crate scoped gates, `mise run
check_wasm`, and `mise run check`.
