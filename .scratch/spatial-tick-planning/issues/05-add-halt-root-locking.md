# 05 — Add Halt root locking

**What to build:** Implement active Halt `*!` as a dependency lock on the Expression root directly
south, scheduled ahead of that root, with no separate control phase. ADR 0032 removed Source-order
turns and requires this ticket to adopt the dependency model before adding its effects.

**Blocked by:** 02 — Add Source Bang activation and expiry; evaluation-machine/05 — Widen the Function kind to value or effect; grid-boundedness/01 — Decide whether the Grid's edge is a language concept.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] An active Halt establishes a dependency edge that locks the complete root one row south
      before that root executes. `CONTEXT.md` already states Halt this way.
- [ ] Empty target is a no-op; occupied non-root target diagnoses.
- [ ] A suppressed Halt does not lock its own target.
- [ ] Halt is never revisited after its turn.
- [ ] The lock is an ordering edge, so it never needs to reach an already-executed root. A lock
      that does is a scheduler defect and rejects the Tick, as `orcvs/src/source/tick.rs` already
      does for a write that reaches an executed computation.
- [ ] Multiple Halts and activations follow ADR 0032's dependency order, with Position breaking
      ties. ADR 0020 governs only each producer's own emission order and Cell-wise conflict
      resolution.
- [ ] Halt answers an effect rather than a value, so ADR 0025's single construction point refuses
      it by its declared kind, per ADR 0029.
