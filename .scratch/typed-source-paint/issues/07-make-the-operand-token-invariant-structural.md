# 07 — Make `SourcePaint::Operand`'s operand-token invariant structural

**What to build:** `SourcePaint::Operand` accepts only the Tokens an operand can carry, so the two `unreachable!("SourcePaint::Operand carries only a declared operand Token")` arms in `console/src/style.rs` (`:205`, `:497`) disappear.

**Blocked by:** None — can start immediately.

**Status:** needs-triage

- [ ] Decide whether the invariant becomes a type: a narrower Token enum, or a constructor that refuses non-operand Tokens. The alternative is to keep it an assertion, with the reason recorded.
- [ ] If it becomes a type, both `unreachable!` arms are gone, and `SourcePaintVisuals`' fact index stays within `benches/floors.toml`.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.** The review of #115 raised this as a judgement call, and #118 deliberately left it out. Nothing tracked it until now.

**2026-09-29 — audit at `cad296df`.** Both `unreachable!` arms still exist; they moved from `:212` and `:507` to `console/src/style.rs:205` and `:497`, and the body now cites the current lines. `SourcePaint::Operand { token: Token, state }` still takes any `Token` (`orcvs/src/source/mod.rs:53`). `benches/floors.toml` has no floor named for `SourcePaintVisuals`; the criterion means the `paint_derive/*` floors (`benches/floors.toml:78-117`). No criterion is met.
