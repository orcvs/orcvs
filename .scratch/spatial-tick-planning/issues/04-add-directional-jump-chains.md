# 04 — Add directional Jump chains

**What to build:** Implement `&^`, `&v`, `&<`, and `&>` as ordinary Functions: each has an input
Portal and an output Portal at its own declared displacement, and relays one complete aligned
two-Cell Language Unit. Consecutive same-direction Jumps compose by overlapping Portals rather than
forming a chain. ADR 0032 removed Source-order turns and requires this ticket to adopt the
dependency model before adding its effects.

**Blocked by:** 02 — Add Source Bang activation and expiry; grid-boundedness/01 — Decide whether the Grid's edge is a language concept.

**Status:** claimed

**Tags:** release/v1

- [x] Consecutive same-direction Jump Functions compose through overlapping Portals: the first
      writes onto the next and suppresses it.
- [x] Each Jump's input and output Portals are this Function's declared displacement.
- [x] Ordinary units overwrite one complete destination Span atomically. "Atomically" means this
      Jump's own write is all-or-nothing; under ADR 0034 other producers still compose Cell-wise
      over the same Span.
- [x] Completely empty input clears the destination; partial/invalid input diagnoses and writes nothing.
- [x] A relayed Bang activates the root it reaches, and the schedule orders that root after the
      relaying Jump regardless of Position. It writes into empty Source and diagnoses at an
      occupied non-root destination.
- [x] Out-of-Grid destinations receive no partial write.
- [x] Jump never transports a Sequence or an incomplete Language Unit.
- [x] Backward routing is ordinary: a Jump below its consumer reaches it during the same Tick. A
      Jump that closes a same-Tick dependency cycle rejects every effect of that Tick, as
      `orcvs/src/source/tick.rs` already does.
- [x] Each Jump spelling declares `can_emit_bang` in `define_functions!`, and the
      `equality_is_the_only_function_that_can_emit_bang` regression in `lang/src/atom.rs` is
      updated rather than deleted.

## Comments

Consecutive same-direction Jumps are ordinary Portal composition, not a chain with one
relaying head. ADR 0014's Jump geometry now states each Function's own Portals.
