# 02 — Record the machine vocabulary in the glossary

**What to build:** Add the terms ADR 0028 introduces to `CONTEXT.md`, so the machine's parts have agreed names and stated non-names.

**Blocked by:** 01 — Correct the evaluation machine decision.

**Status:** resolved

**Sources of truth:** ADR 0028 defines the machine; `CLAUDE.md` makes `CONTEXT.md` authoritative for vocabulary; `CONTEXT.md` shows the entry format.

- [x] `CONTEXT.md` defines Atom.
- [x] `CONTEXT.md` defines Evaluator, and its `_Avoid_` line refuses virtual machine, VM, and interpreter loop.
- [x] `CONTEXT.md` defines Operand Stack, stating that it is created for one Expression and discarded when that Expression answers.
- [x] `CONTEXT.md` defines the absence marker, and distinguishes it from the empty Sequence.
- [x] Each entry carries an `_Avoid_` line in the existing format.
- [x] No entry restates a rule the ADRs own; each names the thing and points at the deciding ADR where a rule is needed.

## Comments

`CONTEXT.md` currently holds 49 glossary entries. It defines Language Unit, Expression, Function, Source Snapshot, Sequence, Play Command, and every Function family. It defines none of the machine's parts.

Atom is the sharpest gap. The Language Unit entry already uses the word — "such as a Function or Atom" — so the glossary depends on a term it never defines, and `lang/src/atom.rs` is built around it.

Evaluator needs its `_Avoid_` line as much as its definition. ADR 0028 spends a paragraph establishing that Orcvs is not a virtual machine, has no bytecode, and has no control flow, and that reasoning is lost if the glossary lets the wrong word back in.

The absence marker is `Atom::Empty`. It renders as `_`, it is what an Expression leaving no value answers, and `orcvs/src/source/tick.rs:267` plans no write for it. ADR 0007's empty Sequence also plans no write, and `tick.rs:280` handles it separately. The two agree on effect and differ in kind, which is exactly why the glossary should separate them — ADR 0026 records that making every value a Sequence would put pressure on this distinction, and an undefined term cannot survive that pressure.
