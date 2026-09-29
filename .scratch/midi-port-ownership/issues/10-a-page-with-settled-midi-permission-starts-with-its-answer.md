# 10 — A page with settled MIDI permission starts with its answer

**What to build:** A performer who has already granted or blocked MIDI access for the page sees the console start with that answer on its first frame. A granted page shows its first destination selected from the start, and a blocked page never shows "waiting for the browser to grant MIDI access", not even for one frame.

The web entry point waits for the page's one MIDI access request before it starts the console, but only up to a short timeout. A first visit that shows the permission prompt must never hold the console back: once the timeout passes, the console starts with access pending and relies on `midi-port-ownership/09` to catch up when the performer answers. An ignored or quietly dismissed prompt therefore costs at most the timeout.

**Blocked by:** 09 — its timeout fallback is the pending path that 09 makes correct, and both change the one access request.

**Status:** ready-for-agent

- [ ] The web entry point awaits the MIDI access request, bounded by a timeout, before starting the console.
- [ ] A page whose permission is already granted starts with its destinations listed and the first selected, with no pending status on any frame.
- [ ] A page whose permission is blocked starts reading `None` with no status on any frame.
- [ ] A page showing the permission prompt starts once the timeout passes and then behaves as 09 describes.
- [ ] The console still starts when the browser has no Web MIDI at all, without waiting for the timeout.
- [ ] ADR 0059 records the timeout's value and why it is that value.
- [ ] The headless browser suite still passes whatever the browser's permission policy is.
