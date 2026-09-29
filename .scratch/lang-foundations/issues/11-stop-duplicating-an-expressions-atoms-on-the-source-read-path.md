# 11 — Stop duplicating an Expression's Atoms on the Source read path

**What to build:** Read an Expression's parsed values without building a fresh collection on every read, and stop storing a second copy of those values beside the Expression that already holds them.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Tags:** Improvement

`lang::Expression::atoms` (`lang/src/expression.rs:99-101`) collects a fresh `Atoms` on every call. The Source read path no longer pays that per read: `orcvs` calls it once per Expression when a row is derived (`orcvs/src/source/language_map.rs:813`), and `ExpressionEntry::atoms` hands out `Option<&Atoms>`. The duplicate that remains is stored. `DerivedExpression` holds both `expression: Expression` and `atoms: Option<Atoms>` (`language_map.rs:215-217`), so a populated Source keeps each executable Expression's values twice, and every row derivation allocates the second copy.

- [x] Reading an Expression's values does not allocate per call on the Source read path. *(Met: `ExpressionEntry::atoms` borrows the stored copy, `language_map.rs:296-298`.)*
- [ ] The Language Map holds one representation of an Expression's values rather than an Expression and a duplicate of its values.
- [ ] The existing allocation-shape assertions cover the read path and pass. A shape they reveal is recorded before it is relaxed, as that suite requires.
- [ ] Interpretation results, Language Map behavior, and native and WASM callers are unchanged.

## Comments

This is the part of the original `lang` artifact's redundant-reconstruction finding that survived auditing. The rest of that finding described a bounded-buffer handoff inside `lang` that issue 07 removed; see issue 04's comments. The duplication that remains is in `orcvs`, on the path the allocation suite was written to watch, which is why that suite is the gate here rather than the benchmark series — allocation shapes answer this question and wall-clock on a shared runner does not.

### Audit at cad296df — 2026-09-29

The claim that every read allocates at Render Frame rate is false now. In shipped `orcvs` code,
`Expression::atoms()` has one caller, at derivation (`language_map.rs:813`). Every other call
reads the stored `DerivedExpression.atoms` through `ExpressionEntry::atoms`, which returns
`Option<&'a Atoms>`, and `bangs()` (`:410`) uses that same reader. So the first criterion is met.

The duplicate is still there: `DerivedExpression` holds `expression` and `atoms` side by side
(`:215-217`), and has since `03f48d22`. The work is now the second criterion: hold one
representation, or derive the Atoms view from the Expression without a second owned `Vec<Atom>`.
