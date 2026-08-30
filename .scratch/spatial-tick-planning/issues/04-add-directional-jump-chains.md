# 04 — Add directional Jump chains

**What to build:** Implement `&^`, `&v`, `&<`, and `&>` chains that relay one complete aligned
two-Cell Language Unit according to ADR 0014.

**Blocked by:** 01 — Order effects by Language Map Position.

**Status:** ready-for-agent

- [ ] Consecutive same-direction Jump Functions form one chain with one relaying head.
- [ ] Direction-specific input, member alignment, and output anchors match ADR 0014.
- [ ] Ordinary units overwrite one complete destination Footprint atomically.
- [ ] Completely empty input clears the destination; partial/invalid input diagnoses and writes nothing.
- [ ] Bang activates a later root, writes into empty Source, and diagnoses at occupied non-root Source.
- [ ] Out-of-Grid destinations receive no partial write.
- [ ] Jump never transports a Sequence or an incomplete Language Unit.
- [ ] Backward routes and cycles stop when a root turn has passed.
