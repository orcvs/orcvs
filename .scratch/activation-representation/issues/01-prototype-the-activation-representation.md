# 01 — Prototype the Activation representation

**What to build:** Implement throwaway, focused alternatives for Activation recognition and one-Tick
planning so the production ticket can choose between a distinct spatial Language Unit and a
self-reproducing Function/value model from evidence rather than ADR shape alone.

**Blocked by:** language-map/01 — Partition Source into Language Units.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Both alternatives recognize all four encodings and preserve direction in Source.
- [ ] Both plan exactly one move per Tick and replace the current Footprint with Bang on collision or
      an out-of-Grid move.
- [ ] The comparison exercises initial emission, overlapping movement, aligned root contact,
      partial-unit contact, generated Source, and Source-order scheduling.
- [ ] The comparison states what Function evaluation, runtime-value, Sequence, Source-write, and
      Glyph machinery each alternative reuses or duplicates.
- [ ] The chosen model maximizes locality behind the Language Map/Tick-planning interface and does
      not grant unrequested Expression or Sequence capabilities.
- [ ] The discarded prototype code is removed; the answer and selected representation constraints
      are recorded in this ticket before `spatial-tick-planning/03` begins.
