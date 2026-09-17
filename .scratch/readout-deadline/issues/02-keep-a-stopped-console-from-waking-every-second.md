# 02 — Keep a Stopped console from waking every second

**What to build:** A Stopped console with Cursor Effect off does not request a timed wake,
so rest does not poll once a second. Playing Run Clock honesty from ticket 01 stays.

**Blocked by:** 01 — Wake Run Clock on a quiet Playing pass

**Status:** resolved

- [x] Stopped plus reduced motion, after a settling pass, requests no timed wake.
- [x] Confirm when writing the test whether the console's first passes request their own repaints (focus, destination auto-select); settle before asserting.
- [x] The Playing, Cursor-Effect-off, at-most-one-second pass from ticket 01 still holds.

## Comments

`until_next` already takes `None` for a clock that is not moving. This ticket is the console-pass
guard that rest does not grow a one-second metronome. Spec: [../spec.md](../spec.md).

## Answer

Stopped with Cursor Effect off passes `None` for the Run Clock term, so `until_next` requests
no timed wake. The first two console passes request an immediate Render Frame (0 ns: focus,
destination auto-select); after those settle, `repaint_delay` is `Duration::MAX`. The Playing
at-most-one-second pass from ticket 01 still holds.

Files: `console/src/console.rs` (`a_stopped_console_with_cursor_effect_off_requests_no_timed_wake`).
