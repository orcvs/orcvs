# 01 — Preserve parser-owned input layouts

Status: resolved
Tags: release/v1
Blocked by: None

**What to build:** Retain the parser's complete input layout so Tick scheduling can identify current data suppliers without reconstructing grammar from spellings. Implement the parser portion of [ADR 0032](../../../docs/adr/0032-schedule-tick-execution-by-dependency.md).

- [x] Expose source-relative positions, expected Tokens, and complete/incomplete/invalid state for operand slots, including slots after an invalid operand and missing tails.
- [x] Preserve nested Function ownership and original Function identities.
- [x] Retain this structure in the Language Map when an Expression has no complete runtime Atoms.
- [x] Bind projected Cell encodings through the established operand types without reparsing the whole Source or changing nested typed-value semantics.
- [x] Distinguish repairable operand errors from trailing or structurally malformed Source; binding a valid prefix must not make an invalid whole Expression executable.
- [x] Keep output behavior metadata with Function declarations so potential Bang dependencies can be established before execution.
- [x] Preserve invalid embedded `**` as an operand diagnostic, with no activation eligibility.
- [x] Cover nested inputs, missing and invalid inputs, invalid early operands followed by valid later slots, row limits, parser capacity, and contextual encoding (`C4` as Note versus hexadecimal Number).

Run the `lang` scoped gate, relevant Language Map checks, and parser boundary/property checks. No dependency or feature additions are required.

## Comments

2026-09-06: Draft changes exist in `lang/src/expression.rs`, `parser.rs`, `atom.rs`, and `orcvs/src/source/language_map.rs`. They add layout/binding, continue analysis after invalid operands, retain the Expression, and expose possible-Bang metadata. `cargo fmt --all` ran; `cargo test --package lang --locked --target-dir /tmp/orcvs-tick-target` passed 159 tests. Language Map tests, scoped gates, and integration remain pending. Draft binding currently ignores trailing Cells; this must be addressed before the scheduler can rely on it. Implementation is paused, not complete.

2026-09-06: Resolved. `Expression::layout` retains typed slots through invalid and missing operands, `bind_source` decodes current Cells through that layout, and the Language Map retains incomplete Function candidates. The scheduler rejects a starting Expression whose parsed Span extends past its layout, so a valid prefix cannot promote trailing malformed Source. Parser property coverage and the complete native/persistence suites pass.
