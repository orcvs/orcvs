# 01 — Share one Output Portal Reservation derivation

Status: ready-for-agent
Blocked by: None (can start immediately)

**What to build:**

An allocation-free function in `portal.rs` answers a Function's Output Portal
Reservation at an anchor, root or nested. A shared walk over
`ScheduleInputs::expressions()` yields each Function's anchor and whether it is
nested. `PortalAccess::resolve`'s Static arm and `computations` use them, and
`output_portal_reservations` is rebuilt on them. See `../spec.md`.

## Acceptance criteria

- [ ] The paint highlight and the schedule's value-answer write Claims come
  from the same function and the same walk.
- [ ] `language_map.rs` no longer resolves a Portal for paint itself.
- [ ] No test changes, and every existing test passes.
- [ ] The allocation test for the paint walk still passes.
- [ ] Gates pass with `PROPTEST_CASES=32`: fmt, clippy and nextest on `orcvs`
  and `console`.
