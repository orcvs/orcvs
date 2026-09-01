# 03 — Add visible Increment and Interpolation

**What to build:** Implement scalar feedback Functions `~+` and `~>` by reading the previous visible
Number at the ordinary result Portal in the current Source Snapshot.

**Blocked by:** 01 — Thread Tick and Position into interpretation; sequence-values/04; lang-foundations/06.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Empty Portal initializes as Number `00`.
- [ ] Increment returns `(previous + step) % modulus` and rejects zero modulus.
- [ ] Interpolation moves toward target without overshoot; rate `00` holds.
- [ ] Tests prove that a previous Note, a Note operand, a Sequence, an invalid Footprint, or another
      non-Number operand diagnoses rather than converting implicitly.
- [ ] Both Functions remain scalar exceptions to Sequence broadcasting.
- [ ] Cross-Tick state is visible only in Source Snapshot Cells.
- [ ] Live Editing and stale-tail behavior remain deterministic.
