# 03 — Add visible Increment and Interpolation

**What to build:** Implement scalar feedback Functions `~+` and `~>` by reading the previous visible
Number at the ordinary result Portal in the current Source Snapshot.

**Blocked by:** 01 — Thread Tick and Position into interpretation;
`.scratch/sequence-values/issues/04-plan-complete-sequence-writes-through-portals.md`.

**Status:** ready-for-agent

- [ ] Empty Portal initializes as Number `00`.
- [ ] Increment returns `(previous + step) % modulus` and rejects zero modulus.
- [ ] Interpolation moves toward target without overshoot; rate `00` holds.
- [ ] Previous Note, Sequence, invalid Footprint, or non-Number operand diagnoses.
- [ ] Both Functions remain scalar exceptions to Sequence broadcasting.
- [ ] Cross-Tick state is visible only in Source Snapshot Cells.
- [ ] Live Editing and stale-tail behavior remain deterministic.
