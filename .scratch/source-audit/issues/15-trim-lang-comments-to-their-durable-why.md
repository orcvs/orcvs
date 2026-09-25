# 15 — Correct lang comments the source-audit changes leave behind

**What to build:** After source-comments/02 holds `lang` to `docs/agents/comments.md` (lineage, provenance citations, `SAFETY:`/doctest/intra-doc carve-outs), what remains is the work that rule does not cover and that depends on the code 05, 09, 24 and 25 change: each argument stated once, and comments that contradict the code corrected. About 41% of non-blank lines in `lang/src` outside inline test modules are comments.

**Blocked by:** source-comments/02; 05 — Diagnose silent drops in lang; 09 — Move lang test-only API behind cfg(test); 24 — Reduce lang's per-Tick work and allocations; 25 — Make an operand's token and bind agree at compile time.

**Status:** ready-for-agent

- [ ] Each argument appears once, at the item that enforces it.
- [ ] Measured-performance comments and safety or invariant statements are kept.
- [ ] Comments that contradict the code are corrected or removed, including: Range Functions "are unbuilt" (`interpreter.rs`); "No Function declares this operand today" (`expression.rs`); "issue 03 gives it the whole-`Value` pop" and "`Atom` or `Sequence` the day a row declares one" (`stack.rs`); "No row declares this today" on `Answer::Sequence` (`atom.rs`, also 05); "issue 03 adds" and "issue 03, Select does not" (`sequence.rs`); "Replace raise this in issue 03." (`error.rs`); "issue 03's decision" (`expression.rs`).
- [ ] Inline attributes are left as 24 settled them.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** The lineage and citation criteria duplicated source-comments/02 (filed by 0ce8d2c4, ready-for-agent), which now owns them; `docs/agents/comments.md` is the rule. This ticket keeps what that rule does not cover and runs after it. "Roughly 45%" corrected to about 41%. More contradicting comments listed.
