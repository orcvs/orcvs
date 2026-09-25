# 02 — Close the Tick gate race between answering and requesting a stop

**What to build:** A stop requested from one handle while the Playback task is answering another's stop keeps the Tick gate shut, so no further Tick is admitted while a stop request remains outstanding. A Tick already admitted may finish; playback may resume after the requests are answered and a subsequent Start is applied. Today answering a stop decrements the outstanding count and reopens the gate as two separate atomic steps; a request landing between them leaves the gate open with a stop outstanding.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The outstanding-stop count and the gate state change together, so the gate cannot be open while any stop request stands.
- [x] The count and the gate state share one atomic word, so no separate answer-then-reopen sequence exists for a request to land inside; the only read-then-commit window is inside `clear_stop`'s `fetch_update` retry. Tests show two outstanding stops keep the gate shut until both are answered.
- [x] The existing guarantee that a stop raised during an executing Tick survives that Tick's finish still holds.
- [x] The single-word design, with `two_outstanding_stops_keep_the_gate_shut_until_both_are_answered`, is the decisive regression. Any concurrent start/stop stress test observes Tick admission order against outstanding requests, permits completion of already-admitted Ticks and legitimate restarts, and does not infer admission order from MIDI delivery timestamps. Test synchronization stays in test-only code.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid; no change needed.

**2026-09-25 — acceptance-criteria review against `199c3331`.** The isolated forced-interleaving reproduction admitted a Tick with one stop outstanding while all seven existing gate tests passed. This proves the gate defect, not an end-to-end unwanted MIDI event. Narrowed the guarantee to admission rather than completion.

**2026-09-25 — implementation.** Implemented in orcvs/orcvs#144 (epic PR 1): stop count and executing flag share one atomic word. The hand-driven interleaving test was removed in cf24590d (it exercised compare-and-swap, not `clear_stop`); the race window now lives only inside `fetch_update`. Resolve on merge.

**2026-09-25 — criteria revised.** A test that drives `clear_stop`'s read and commit by hand only shows that a compare-and-swap against a stale word fails, which std guarantees; with the count and gate in one word no separate window exists outside `fetch_update`. Criteria 2 and 4 now name the single-word construction and the two-outstanding-stops test instead of a forced-interleaving test. Decided by the maintainer.

**2026-09-25 — resolved.** Merged in orcvs/orcvs#144 (`c7249ed0`); every criterion verified on `main`.
