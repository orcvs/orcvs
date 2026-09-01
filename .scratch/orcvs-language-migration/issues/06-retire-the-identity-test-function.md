# 06 — Retire the identity test Function

**What to build:** Remove `id` from parsing, display, interpretation, fixtures, and tests. Replace
tests that need a real computation with audited numeric Functions rather than retaining a
test-only language concept.

**Blocked by:** 01 — Move arithmetic onto the `.` family.

**Status:** resolved

- [x] `id` no longer parses as a Function or appears in `Function`/`Atom` display.
- [x] Parser, interpreter, Source, persistence, native, and WASM tests use real Functions.
- [x] Nested-expression and result-commit coverage remains behaviorally equivalent.
- [x] Unknown `id` Source receives the ordinary unknown-Function diagnostic.
- [x] No replacement identity Function is introduced without a user-facing composition need.

## Comments

ADR 0015 retires `id`; its current role is test scaffolding rather than language capability.

The implementation had already removed `id` from the language and migrated its computation tests.
Completion replaced the remaining Language Map extent fixtures with audited arithmetic Functions
and added Source-level regression coverage for the ordinary unknown-Function diagnostic. The
scoped `orcvs` gate and `mise run check`, including persistence and WASM checks, pass.
