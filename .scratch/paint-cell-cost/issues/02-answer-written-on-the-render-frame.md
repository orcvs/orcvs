# 02 — Answer "written" on the Render Frame instead of hashing it per Cell

**What to build:** Whether a claim's slot holds written content becomes a per-Cell fact the Render
Frame's own derivation fills, read in the paint loop as an indexed `bool`. The
`HashMap<*const Claim, bool>` in `Paint::derive_with_colours` goes away, and with it the per-Cell
hash and the rehash cascade that rebuilds the map from empty on every call.

**Blocked by:** 01 — Apportion the per-Cell regression across the four facts the paint gained.

**Status:** ready-for-agent

`01` measured this at 77–79% of the regression: 16.2–18.3 ns of the 20.5–23.8 ns added per Cell. A
variant that keeps every answer identical and only moves the fact onto a slice the Frame holds
measured 2.8x better — `fitted/256x256` from 27.8 to 10.0 ns/Cell, `culled/16x16` from 24.2 to 8.5.

## Two fixes that look cheaper and are not

Both were measured in `01` and both are refused here, so they are not proposed again in review:

- **A faster hasher** recovers 7.7–10.3 ns of the 16.2–18.3. Roughly half the map's cost is not
  hashing at all: it is `HashMap::new()` at zero capacity on every `derive` call and the rehash
  cascade as tens of thousands of claims go in. The sampling profile puts `reserve_rehash` alone at
  25.7–29.0% of `derive` time, from one call site.
- **A one-entry memo on the last claim** recovers 2.5–3.2 ns. Claims are about two Cells wide and
  the walk changes claim constantly, so it hits too rarely to matter.

The word "without hashing" in `syntax-highlighting/07`'s follow-up understates the problem. The map
has to go, not get faster.

- [ ] The Render Frame answers, per Cell, whether that Cell's claim's slot holds written content —
      a `RenderCell` field or a slice parallel to `cells`, filled once during the Frame's own
      derivation, computed once per claim rather than once per Cell.
- [ ] `Paint::derive_with_colours` reads it as an indexed `bool`. The `HashMap`, its per-Cell
      lookup, and `slot_written`'s use from the paint loop are gone.
- [ ] Every answer is identical. Every existing paint and style test passes unchanged — this is a
      pure cost change, and a test needing an edit means the answers moved.
- [ ] The Frame's own derivation does not acquire the cost the paint shed: "written" is still
      answered once per claim, not once per Cell, and the `orcvs` allocation tests still pass.
- [ ] `01`'s finding is not re-litigated by a cheaper hasher or a memo. If the implementation
      reaches for either, that is a signal the fact did not move where it needed to.
- [ ] The paint benchmark comparison on the pull request shows the recovery. Expect roughly
      10 ns/Cell, about 2.1–2.2x over pre-#109 — **not** a return to 4.3 ns/Cell. The rest is the
      claim model (ADR 0044) and `claim_paint`, which this does not remove. The comparison lives in
      the action, so it is read there rather than locally.
- [ ] The scoped gates for `console` and `orcvs` pass.

## Optional, and only if it is free

`claim_paint` is the other 16–19% (3.4–4.5 ns/Cell), of which `fill_tint_colour`'s `lerp_to_gamma`
is 1.0–1.3 ns. Precomputing the ten role tints once per `SourcePaintSettings` rather than mixing per
Cell would take that third. Worth about 5% of the added cost — take it if it falls out of the work,
not as a reason to widen this ticket.

Candidates 1 and 3 from `01` — the `Arc` deref and the `output_portal` read — measured at or below
zero. Do not touch them.

## Then

`benchmarks/07`'s floor should guard `paint_derive` once this lands, so the recovered figure is held
rather than left to drift back. `syntax-highlighting/07`'s last open follow-up closes with this
ticket.
