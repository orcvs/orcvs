# 06 — Retire the identity test Function

**What to build:** Remove `id` from parsing, display, interpretation, fixtures, and tests. Replace
tests that need a real computation with audited numeric Functions rather than retaining a
test-only language concept.

**Blocked by:** 01 — Move arithmetic onto the `.` family.

**Status:** ready-for-agent

- [ ] `id` no longer parses as a Function or appears in `Function`/`Atom` display.
- [ ] Parser, interpreter, Source, persistence, native, and WASM tests use real Functions.
- [ ] Nested-expression and result-commit coverage remains behaviorally equivalent.
- [ ] Unknown `id` Source receives the ordinary unknown-Function diagnostic.
- [ ] No replacement identity Function is introduced without a user-facing composition need.

## Comments

ADR 0015 retires `id`; its current role is test scaffolding rather than language capability.
