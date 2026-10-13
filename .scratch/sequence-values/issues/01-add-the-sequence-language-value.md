# 01 — Add the Sequence language value

**What to build:** Represent one flat ordered Sequence of Atoms as a language value that can cross
Function evaluation without becoming Source writes prematurely.

**Blocked by:** orcvs-language-migration/04 — Confirm contextual Number and Note literals.

**Status:** resolved

**Tags:** release/v1

- [x] Sequence preserves Atom order and each Atom's type.
- [x] Atom promotion creates singleton Sequences where a Sequence Function requires it.
- [x] Nested Sequences are impossible or flattened at the one controlled construction point.
- [x] Empty and singleton Sequences have explicit behavior.
- [x] Encoding is deterministic and concatenates complete Atom encodings.
- [x] Bang is a permitted Atom and survives construction, promotion, and encoding.
- [x] Self-Banging Functions remain root-only Source effects and never become Sequence members.
- [x] Language errors distinguish Atom, Sequence, and incompatible operands.

## Comments

Sequence is a value, not a Cell batch or implicit Tick Plan write list.

`Sequence` and `Value` live in `lang/src/sequence.rs`. `Stack` holds a `Value`, so a Sequence
crosses Function evaluation without becoming Source writes; `Interpretation::Sequence` carries one
out of evaluation, and `plan_tick` routes it through the existing below-root, complete-fit,
horizontal-write path. No Source-parseable Function returns a Sequence yet, so that arm is
exercised by direct `Sequence` and `Stack` tests rather than by Source text.

The Bang item originally paired two claims: that Bang is a permitted member, and that structural
Sequence operations preserve its type and encoding. Only the first is about this issue — the
operations named in the second are Reverse, Concatenate, Select, and Replace, which issue 03 builds.
The item is narrowed here to the half this issue owns and tested; the structural half moved to
issue 03, which is the issue that can close it.

### Audit against origin/main cf01e3c — 2026-10-13

Retired. ADR 0063 superseded ADR 0007 and retired the Sequence value, pervasive extension and the
`:` family; `cell-tracker/01` (PR #199, `eca29a15`) removed them. On `cf01e3c` no `:`-spelled
Function is in `define_functions!` (`lang/src/atom.rs:727-781`), `lang/src/sequence.rs` and
`lang/src/functions/sequence.rs` do not exist, and no `Sequence` type remains. The boxes above
record what this ticket delivered; none describes main, and the tests named above no longer exist.
