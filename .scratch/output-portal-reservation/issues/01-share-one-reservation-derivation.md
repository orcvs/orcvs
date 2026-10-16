# 01 — Share one Output Portal Reservation derivation

Status: resolved
Blocked by: None (can start immediately)

**What to build:**

An allocation-free function in `portal.rs` answers a Function's Output Portal
Reservation at an anchor, root or nested. A shared walk over
`ScheduleInputs::expressions()` yields each Function's anchor and whether it is
nested. `PortalAccess::resolve`'s Static arm and `computations` use them, and
`output_portal_reservations` is rebuilt on them. See `../spec.md`.

## Acceptance criteria

- [x] The paint highlight and the schedule's value-answer write Claims come
  from the same function and the same walk.
- [x] `language_map.rs` no longer resolves a Portal for paint itself.
- [x] No test changes, and every existing test passes.
- [x] The allocation test for the paint walk still passes.
- [x] Gates pass with `PROPTEST_CASES=32`: fmt, clippy and nextest on `orcvs`
  and `console`.

## Comments

2026-10-16 — `PortalAccess::answer_site` decides where a value answer writes, root or nested; `PortalAccess::resolve`'s Static arm and `PortalAccess::output_portal_reservation` both read it, and the Reservation is `Portal::at(site).reservation()` in both the highlight and `Lookup::new`. `tick::output_portal_reservations` walks `ScheduleInputs::expressions()` with `ScheduledEntry::function`, the anchor rule `computations` also uses. `LanguageMap::output_portal_reservations` now collects that walk; `function_candidate` is test-only because no shipped code reads it. No test changed: 1496 orcvs and console tests pass with `PROPTEST_CASES=32`.
