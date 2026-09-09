# 04 — Add directional Jump chains

**What to build:** Implement `&^`, `&v`, `&<`, and `&>` chains that relay one complete aligned
two-Cell Language Unit according to ADR 0014. ADR 0032 removed Source-order turns and requires this
ticket to adopt the dependency model before adding its effects.

**Blocked by:** 02 — Add Source Bang activation and expiry.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Consecutive same-direction Jump Functions form one chain with one relaying head.
- [ ] Direction-specific input, member alignment, and output anchors match ADR 0014.
- [ ] Ordinary units overwrite one complete destination Span atomically. "Atomically" means this
      Jump's own write is all-or-nothing; under ADR 0034 other producers still compose Cell-wise
      over the same Span.
- [ ] Completely empty input clears the destination; partial/invalid input diagnoses and writes nothing.
- [ ] A relayed Bang activates the root it reaches, and the schedule orders that root after the
      relaying Jump regardless of Position. It writes into empty Source and diagnoses at an
      occupied non-root destination.
- [ ] Out-of-Grid destinations receive no partial write.
- [ ] Jump never transports a Sequence or an incomplete Language Unit.
- [ ] Backward routing is ordinary: a Jump below its consumer reaches it during the same Tick. A
      Jump that closes a same-Tick dependency cycle rejects every effect of that Tick, as
      `orcvs/src/source/tick.rs` already does.
- [ ] Each Jump spelling declares `can_emit_bang` in `define_functions!`, and the
      `equality_is_the_only_function_that_can_emit_bang` regression in `lang/src/atom.rs` is
      updated rather than deleted.
