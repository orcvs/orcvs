# 04 — Bind and read editable Note Cells

Status: needs-triage
Blocked by: 02, 03

## Work

Implement the accepted Function/declaration and data interpretation from 01–03.
Use the existing Function table, Parser, Language Map, typed operands and Portal
binding abstractions. A result has the accepted Note/Sequence/rest representation;
it is not a string reparsed by MIDI or an independent spelling classifier.

## Acceptance

- [ ] The signature declares kind, activation, pervasion, result width and read
      geometry consistently. Reserved spellings become parseable only as accepted.
- [ ] The region can hold repeated and nonadjacent pitches with one spelling per
      note; each step is independently editable through ordinary Source edits.
- [ ] Unit and boundary/property tests cover the full contract matrix in 02,
      extent limits, single-step and all-rest patterns, and malformed intermediate
      edits. Diagnostics identify the relevant Source Cells.
- [ ] A file read/write round trip preserves spacing, blanks and step positions.
- [ ] Existing Sequence, literal-context and generated-code behavior is preserved
      except for expressly accepted amendments, with regression coverage.
- [ ] Each new Function appears in `console/assets/function_reference.orcvs`
      and passes the reference's inventory and execution tests.
- [ ] No shipped parameter or branch exists solely to inject test-only data.

Likely implementation areas: `lang/src/atom.rs`, `parser.rs`, `expression.rs`,
`functions/`, and `orcvs/src/source/language_map.rs` and `portal.rs`.
Read the Rust-change skill; read egui guidance if Function reference work reaches
presentation. Keep partial implementation unshipped until 05 makes it executable.
