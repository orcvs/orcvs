# 01 — Correct the evaluation machine decision

**What to fix:** ADR 0028 makes two statements that are false against the code and one requirement that is stronger than the fix needs. Correct all three in `docs/adr/0028-specify-the-orcvs-evaluation-machine.md`.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Sources of truth:** the code cited below; `pre-split-defects/15` for the Operand Stack proof; `lang-foundations/02` and `06` for what the single declaration already covers.

- [x] The fifth paragraph no longer says that exceeding the Operand Stack bound "aborts the process".
- [x] The fifth paragraph states what a panic there actually does, and the severity argument survives the correction.
- [x] The fifth paragraph no longer requires the sufficiency proof to be restated whenever a Function's arity grows.
- [x] The third paragraph credits the declaration that exists and names only the parts that are genuinely spread.
- [x] No statement in the ADR contradicts `pre-split-defects/15`.

## Comments

**"Exceeding it aborts the process" is false.** No manifest sets `panic = "abort"`, so the panic unwinds. It unwinds out of `Interpreter::execute`, through `Source::plan_tick` — which runs before `commit_tick`, so the Source is not left half-written — through the `RwLock` write guard taken by `SourceCommander::execute` at `orcvs/src/source/mod.rs:107`, through the `Mutex` taken by `lock_recover` at `orcvs/src/playback.rs:473`, and into the tokio clock task whose `JoinHandle` is dropped. Both locks now recover from poisoning, because `pre-split-defects/02` resolved that. So the process survives, the editor survives, the Source is consistent, and Playback silently stops with no diagnostic. That is a worse failure to diagnose than an abort, not a milder one, and the paragraph should say so.

**The per-arity restatement clause is stronger than the fix needs.** `pre-split-defects/15` carries a proof that never needs restating: a literal pushes one value, a Function pops at least one and pushes one, so net change is never positive, peak depth never exceeds the Atom count, and the parser already bounds that at `EXP_LEN`. `Stack<EXP_LEN>` is therefore correct for every arity, now and later. Requiring a fresh proof on every arity change invites exactly the drift the ADR is trying to prevent. Require instead that the bound be derived from `EXP_LEN` rather than written as a separate number.

**The third paragraph understates what exists.** `define_functions!` in `lang/src/atom.rs` is already one declaration generating the enum, `ALL`, `spelling`, `kind`, `signature`, and `TryFrom<&str>`, with a compile-time assertion on spelling width and `#[deny(unreachable_patterns)]` against duplicate spellings. `lang-foundations/02` and `06` delivered that and are resolved. Two of the ADR's four "places that cannot check each other" are therefore already one place.

What remains true, and is what the paragraph should say: an operand's role and position live only in the body that reads it, as `operands.number(0)` and `operands.number(1)` throughout `lang/src/functions/math.rs` and `lang/src/functions/mod.rs`, so nothing detects a transposition; and `.^` and `.v` do not consume their declared signature at evaluation, taking `Stack::try_pop::<NumericValue>` instead, which accepts both Number and Note. That second one is not drift. `lang-foundations/06` records it as a deliberate exclusion, and ADR 0021 is the rule it serves: the literal signature is monomorphic while evaluation is idempotent for nested values. The ADR already calls this "one rule with one exception" and is right to; it should not be read as a defect.
