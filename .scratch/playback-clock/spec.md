# Playback clock

The Playback Engine's clock had no shared statement of what a run owes a deadline it
could not reach in time, and each of its four loop bodies answered separately. Two
different answers were running: the native clock replayed the backlog and held its
grid, the browser clock dropped the backlog and moved its grid. Neither was chosen.

Two efforts, in order. The first names the rule and makes both targets hold it. The
second removes the duplication that let them disagree unnoticed.

## Background

`orcvs/src/playback.rs` spawns a clock in four places — native `start`, native
`retune`, browser `start`, browser `retune` — each embedding the same six steps around
a target-specific way of waiting. The 58 uses of `activate_for_test` / `clock_tick`
supply `TickTiming` ready-made, so no test in the suite observes a deadline being
produced. That is why a policy divergence survived a same-day refactor, a browser port,
a crate split, and an architecture review.

## Not in scope

Reshaping the 58-use test door. Those tests exercise Tick execution — ownership,
expiry, delivery order — and driving them through a clock would make them slower
without making them truer. What was missing was a test asserting the two targets
schedule alike, which effort 01 adds.
