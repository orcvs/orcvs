# 01 — Apportion the per-Cell regression across the four facts the paint gained

**What to build:** A profile that says how much of the ~5.5x per-Cell regression each of the four
additions in `syntax-highlighting/08`–`10` accounts for, so the fix targets the cost rather than the
most recently written line.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

## The evidence

The Benchmark workflow on `main` at `22ca2718` (the merge of pull request #109,
`syntax-highlighting` `01`–`10`) failed, with an alert on every paint point. Measured against
`6101ca94` (#108) immediately before it:

| benchmark | #108 | #109 | ratio |
|---|---|---|---|
| `paint_derive/culled/16x16` | 2,165 | 12,506 | 5.78x |
| `paint_derive/fitted/16x16` | 2,165 | 12,224 | 5.65x |
| `paint_derive/culled/32x32` | 2,178 | 12,500 | 5.74x |
| `paint_derive/fitted/32x32` | 8,993 | 50,140 | 5.58x |
| `paint_derive/culled/64x64` | 2,182 | 12,328 | 5.65x |
| `paint_derive/fitted/64x64` | 37,243 | 197,261 | 5.30x |
| `paint_derive/culled/128x128` | 2,165 | 11,960 | 5.52x |
| `paint_derive/fitted/128x128` | 155,917 | 790,456 | 5.07x |
| `paint_derive/culled/256x256` | 2,187 | 11,931 | 5.46x |
| `paint_derive/fitted/256x256` | 629,884 | 3,232,981 | 5.13x |
| `paint_background_runs/fitted/256x256` | 84,705 | 125,777 | 1.48x |

The action's own words, from that run's log:

```
##[warning]Performance alert! Previous value was 629884 and current value is 3232981.
It is 5.132660934394269x worse than previous exceeding a ratio threshold 1.5
```

`console/benches/paint.rs` and `orcvs/benches/` are unchanged between those two commits, so the
fixture did not move. This is code.

## What the shape already tells us

The regression is **per drawn Cell, not a fixed setup cost.** `culled/16x16` walks a constant 256
Cells whatever the Grid holds, and it regressed 5.78x — the same factor as `fitted/256x256`'s 5.13x
over 65,536 Cells. A whole-Grid cost paid once could not produce that: it would barely show in the
culled series and would dominate the fitted one. So the added cost is inside the loop body, paid
once per Position.

The Render Frame's own derivation is not implicated by these numbers either way: `frames()` builds
each frame once in a `OnceLock`, outside the timed body.

## The four candidates

`Paint::derive_with_colours`'s per-Cell body gained all of these together, which is why the series
cannot apportion them:

1. `frame.at(position).claim()` — an `Option<&Claim>` where `token()` and `bound()` were two scalar
   reads. Each claim is one `Arc` shared by every Cell it covers (ADR 0044).
2. The `written_cache` lookup — `HashMap<*const Claim, bool>`, hashed **once per Cell** even though
   `slot_written` itself is computed once per claim. `std`'s default hasher is SipHash-1-3.
3. `cell.output_portal()` — a third input to the decision, added by `06`.
4. `claim_paint` itself — one wider decision in place of two narrower Token matches.

Arithmetic worth carrying into the profile: `culled/16x16` is 256 Cells, so the body went from about
8.5 ns to about 48 ns per Cell. A SipHash lookup on a pointer key is plausibly a double-digit
nanosecond share of that ~40 ns, which makes candidate 2 the first suspect — but suspicion is not
apportionment, and three other things changed in the same commit.

- [ ] A profile of `Paint::derive_with_colours` attributes the per-Cell cost across the four
      candidates, on a stated machine, with the method recorded so it can be re-run.
- [ ] Each candidate's share is a measured figure, not an inference from the diff.
- [ ] The profile states which candidates are worth fixing and which are noise at this size.
- [ ] Where a candidate is cheap in isolation but expensive in place (a cache miss the profile can
      see, an `Arc` deref that defeats a prefetch), that is said rather than averaged away.
- [ ] Local absolute numbers are not compared against CI's. `benchmarks/07` records why a figure
      without its runner misleads; the same applies here, and the deliverable is the *split*, not a
      number to put beside CI's.

## Then

The fix is `02`, once this says what to fix. `syntax-highlighting/07` already carries the remedy
that candidate 2 would want — "answer written once per claim without hashing, e.g. on the Render
Frame beside the claim" — and that line stays open there until this profile either justifies it or
points elsewhere.

## Comments

**2026-09-20.** Found while opening pull request #111 to answer `syntax-highlighting/07`'s last
item, which asks whether the paint bench moved when the claim cache landed. It moved by 5.5x, which
is a larger question than the one asked, so it is tracked here rather than inside that ticket.

The gate caught this and the catch was not acted on. `benchmark-action/github-action-benchmark`
compares each run against the previous stored point, so once 3,232,981 published as `main`'s value
it became the baseline: #106, #110 and #111 all read green against it, and #111 measured 3,190,267.
The regression is now invisible to the gate that found it. `benchmarks/07` is the ticket for that
mechanism and this occurrence is recorded there as its second piece of evidence.
