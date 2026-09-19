# 05 — Decide how a Result is known before a Tick

**What to build:** A recorded decision on how a Source revision names the Cells that hold a Function's Result, so the paint can colour them, and an amendment to the spec, which says the role already exists.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The decision states which Cells are Result Cells for a root: at least the Output Portal site one row south, and how far a Function that can answer a Sequence reaches (for example, to the end of that row, as a Reservation does).
- [x] It states where the fact lives — the Language Map, the Render Frame's derivation, or neither — and why that is not a third classifier.
- [x] It states what the written Cells parse as. Today a `07` left south of `.+0304` re-parses as two unknown one-Cell Functions with diagnostics, so it would paint as Function and Diagnostic, not Result. Whether that parse changes is part of the decision, because it is a language change, not a paint change.
- [x] It states whether a Result is recognised by position alone (a stale write under an edited Function) or only while its root still answers that shape.
- [x] The spec's claim that every role already exists is corrected.
- [x] If the decision changes parsing or the Language Map's vocabulary, it is recorded in `CONTEXT.md` and an ADR.
- [x] The decision defines which Expressions are eligible. The scheduler selects by `function_candidate()` (`orcvs/src/source/tick.rs:919`), not `root()`, so the decision states whether an incomplete Function candidate's destination counts.
- [x] The decision states whether each scheduler case below counts as a Result, and over which Cells. `10` tests its derivation against this list.

## Answer

Decided 2026-09-19 in a grilling session. The spec's "Result" is the Function's **Output Portal** (`CONTEXT.md`). This is syntax highlighting only: no `lang` change and no change to parsing or semantics. The highlight is derived in `orcvs` from declarations `lang` already makes.

- **What is highlighted.** The Output Portal each root Function declares, placed by its anchor in the current Source revision. It is known before any Tick. It is not a record of what a Tick wrote: the fact is recomputed per revision, so a write left behind by a deleted or edited Function is not highlighted, and an empty Output Portal is.
- **Parse.** Unchanged. The highlight is an overlay. A `07` in `.+0304`'s Output Portal still parses as two unknown one-Cell Functions with diagnostics. Which fact paints on such a Cell is `06`/`09`'s precedence decision.
- **Width.** The Reservation (ADR 0036) from the Output Portal: the Cell pair for a Function that can only answer a scalar, and from the Output Portal to the end of that row for one that can answer a Sequence.
- **Eligibility.** Every root Function that answers a value, whether or not its operands bind, matching the scheduler's `function_candidate()`. Nested Functions are never highlighted: their answer goes to the parent.
- **Where it lives.** On the Language Map, as the root Function's declaration placed at its anchor. It interprets no spelling, so it is not a third classifier. The Render Frame carries it, as it carries claims.
- **Overlap.** The fact covers every Cell of the Reservation whatever else claims it (a consumer's operand, another root). Paint precedence is `06`/`09`'s.
- **Naming.** The paint role, the Theme colour and `SourcePaintSettings::result` are renamed Output Portal. The Theme colours are stored by position (`console/src/source_paint.rs:240`), so the stored format is unchanged.
- **No ADR.** The decision is presentation and reversible.

| Case | Highlighted |
|---|---|
| Root that answers a value, including incomplete operands | Its Reservation from the Output Portal |
| Nested Function | No |
| Terminal Output Function | No: it writes nothing through its Output Portal |
| Halt | No: it locks the root at its Output Portal and writes nothing |
| Jump | Yes, at the Output Portal its direction declares |
| Source-writing Function, including an Advance's cleared anchor | No: its writes are its Source effect. `10` names this as a deliberate exclusion from the scheduler's reservations |
| Scalar at the row edge | No: the scheduler reserves nothing when the pair cannot fit |
| Sequence-capable root | From the Output Portal to the end of that row |

Input Portals (Jump, Increment, Interpolation) are not highlighted. That would be a new role and its own issue.

Sharing one derivation between the highlight and the scheduler, instead of proving two derivations agree, is `11`.

## Comments

Research found no Result notion before Tick planning: Reservations and Portal destinations are computed while a Tick is planned and are not reachable from the Source revision or Render Frame. A Function's declared Output Portal exists on the Function itself but nothing at parse or render time reads it. A `**` a Function wrote is indistinguishable from a typed one.

**Scheduler cases the decision must cover (added 2026-09-18, from the claim-seam design review).** These are the scheduler's current behaviours. A position-based rule has to say, for each one, whether it counts as a Result and over which Cells:

| Case | Current scheduler behaviour |
|---|---|
| Recognised Function with incomplete operands | Included as a computation |
| Nested Function | Supplies a typed result; has no write Portal |
| Terminal output or locking Function | Has no Cell write site |
| Jump | Uses its declared displacement |
| Source-writing Function | Uses `source_effect()`; an Advance includes its original anchor |
| Scalar at the row edge | Reserves nothing if the pair cannot fit |
| Sequence-capable computation | Reserves the destination row's remaining Cells |

If every scheduler write reservation counts as a Result, the Advance's cleanup at the producer's own anchor counts too. The decision has to include it or exclude it explicitly.

One candidate answer to the stale-write and parse criteria: recompute destinations from the current revision, and overlay them on the existing parse without changing it. Under that answer, a Cell is a Result only while an eligible Expression in the current revision reserves it. That is a decision to record here. ADR 0036 does not already make it. `10` builds the derivation and its agreement test once this issue resolves.
