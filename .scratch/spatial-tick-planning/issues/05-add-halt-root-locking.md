# 05 — Add Halt root locking

**What to build:** Implement active Halt `*!` as a Source-order lock on the Expression root directly
south, with no separate control phase.

**Blocked by:** 02 — Add Source Bang activation and expiry.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Active Halt locks a complete root one row south before that root's later turn.
- [ ] Empty target is a no-op; occupied non-root target diagnoses.
- [ ] A suppressed Halt does not lock its own target.
- [ ] Halt is never revisited after its turn.
- [ ] A lock cannot retroactively suppress a root whose turn already passed.
- [ ] Multiple Halts and activations retain ADR 0020 producer order.
- [ ] Halt is refused Sequence membership at ADR 0025's single construction point, by name rather
      than by admitting the Function family, per ADR 0029.
