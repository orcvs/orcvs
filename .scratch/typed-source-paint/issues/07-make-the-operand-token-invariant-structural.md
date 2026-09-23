# 07 — Make `SourcePaint::Operand`'s operand-token invariant structural

**What to build:** `SourcePaint::Operand` accepts only the Tokens an operand can carry, so the two `unreachable!("SourcePaint::Operand carries only a declared operand Token")` arms in `console/src/style.rs` (`:212`, `:507`) disappear.

**Blocked by:** None — can start immediately.

**Status:** needs-triage

- [ ] Decide whether the invariant becomes a type: a narrower Token enum, or a constructor that refuses non-operand Tokens. The alternative is to keep it an assertion, with the reason recorded.
- [ ] If it becomes a type, both `unreachable!` arms are gone, and `SourcePaintVisuals`' fact index stays within `benches/floors.toml`.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.** The review of #115 raised this as a judgement call, and #118 deliberately left it out. Nothing tracked it until now.
