# 10 — A page with settled MIDI permission starts with its answer

**What to build:** A performer who has already granted or blocked MIDI access for the page sees the console start with that answer on its first frame. A granted page shows its first destination selected from the start, and a blocked page never shows "waiting for the browser to grant MIDI access", not even for one frame.

The web entry point waits for the page's one MIDI access request before it starts the console, but only up to a short timeout. A first visit that shows the permission prompt must never hold the console back: once the timeout passes, the console starts with access pending and relies on `midi-port-ownership/09` to catch up when the performer answers. An ignored or quietly dismissed prompt therefore costs at most the timeout.

**Blocked by:** 09 — its timeout fallback is the pending path the Panel catch-up makes correct, and both change the one access request.

**Status:** ready-for-human

**Tags:** release/v1

- [x] The web entry point awaits the MIDI access request, bounded by a timeout, before starting the console.
- [ ] A page whose permission is already granted starts with its destinations listed and the first selected, with no pending status on any frame.
- [ ] A page whose permission is blocked starts reading `None` with no status on any frame.
- [ ] A page showing the permission prompt starts once the timeout passes and then behaves as 09 describes.
- [x] The console still starts when the browser has no Web MIDI at all, without waiting for the timeout.
- [x] ADR 0059 records the timeout's value and why it is that value.
- [ ] The headless browser suite still passes whatever the browser's permission policy is.

## Comments

**2026-09-29 — implemented.** The web entry point awaits `console::native_midi::request_access_within` before starting eframe. It sends the page's one access request and races the Promise that resolves once the answer is stored against a 250 ms `setTimeout` Promise; a browser whose `requestMIDIAccess` is missing or throws is settled at once and has nothing to race. No dependency or `web-sys` feature was added. ADR 0059 records the value and why.

Evidence: `console/tests/wasm.rs`'s `the_midi_access_wait_ends_whether_or_not_the_browser_answers` holds in the headless browser suite that the wait ends even when the browser leaves the request unanswered; the suite runs in the merge tier.

Left open, for a person with a browser (Chrome or Edge, `cd console && trunk serve --open`):

1. With MIDI access already allowed for the page and a MIDI output connected, reload. The first painted frame shows the port selected and no "waiting for the browser to grant MIDI access" status.
2. Block MIDI access in the site settings and reload. `None` shows with no status on any frame.
3. Clear the site's MIDI permission and reload. The console appears within about a quarter of a second while the prompt is still showing; answering the prompt then behaves as `midi-port-ownership/09` describes.

**2026-09-30 — what the automated suites can and cannot show.** The two "on any frame" lines are held in the model and not in a browser. `console/src/web_midi.rs`'s `a_console_built_after_the_answer_never_shows_access_pending` builds a `MidiDeviceSelection` over access that is already granted or refused, runs the console's startup discovery and three Panel frames, and holds that the status is `None` throughout, that granted access selects the first output, and that refused access selects nothing. It proves the selection's half: given a settled answer when the console is built, no frame reads pending. It does not prove the entry point's half, that the answer really is settled by then on a real page.

The headless browser suite cannot close that gap. `MidiDeviceSelection` is `pub(crate)`, so `console/tests/wasm.rs` cannot build one or step its frames; the suite drives no eframe Panel; and headless Firefox's permission answer is its own policy, which the test cannot set to granted or blocked and has no MIDI output to list. `web_midi_answers_discovery_and_connect_without_waiting` now requires discovery and connect to agree on whichever of pending, refused or granted the browser gives, and `the_midi_access_wait_ends_whether_or_not_the_browser_answers` holds that the wait ends. The per-frame lines, and the prompt line, stay with the three manual steps above.

### Issue audit against d3fd1b27 — 2026-10-01

The `Blocked by:` prose named 09 a second time, which `scripts/roadmap.ts`'s reference pattern
read as a second blocker; reworded so the line names it once. Read `console::native_midi::request_access_within`
in the 2026-09-29 comment as `console::console_midi::request_access_within`. The headless suite
passed on merge-group run 36799739445, under one permission policy (headless Firefox).
