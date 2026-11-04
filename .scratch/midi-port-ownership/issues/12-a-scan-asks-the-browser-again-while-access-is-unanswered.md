# 12 — A Scan asks the browser again while MIDI access is unanswered

**What to build:** A performer whose browser left the page's MIDI access request unanswered can ask again. On a return visit to https://orcvs.com/ in Firefox, the console read "waiting for the browser to grant MIDI access" and no permission prompt appeared. Firefox grants Web MIDI through a site permission add-on install prompt; a dismissed prompt leaves its request pending for good. The console has no way to tell that request apart from one the performer is still answering, so the pending message names Scan, and Scan sends the browser a fresh request, which the browser prompts for.

**Blocked by:** none

**Status:** resolved

**Tags:** release/v1

- [x] Scan asks the browser for access again while access is pending, and only then.
- [x] Panel frames never ask again on their own.
- [x] A grant from any request settles access; a refusal settles it only when it answers the latest request.
- [x] The pending message names Scan.
- [x] ADR 0059 records the amendment.
- [ ] In Firefox with a MIDI device connected, dismiss the add-on prompt, reload, open Output and press Scan: the prompt shows again, and allowing it lists the device without another click.

## Comments

**2026-11-01 — implemented.** `MidiBackend::request_access_again` (default: nothing) is called by `MidiDeviceSelection::scan`, which the Output menu's Scan uses. `WebMidiBackend` forwards it to `WebMidiAccess::ask_again` while access is pending; the browser adapter sends another `requestMIDIAccess` and counts it as outstanding. Evidence: `console/src/web_midi.rs`'s `a_scan_asks_the_browser_again_only_while_access_is_unanswered`, and the headless browser suite's `web_midi_answers_discovery_and_connect_without_waiting`, which now holds that a fresh request returns without waiting. The last line needs a person with Firefox and a MIDI device: Firefox rejects the request after a few seconds when no device is connected, and the headless suite has no device or prompt.

**2026-11-01 — review round.** The review on PR #348 found that an outstanding-request count let a dismissed request that never settles hold back a later refusal, dropped a grant that arrived after a refusal, and was tested only in the browser. The rules now live in `console/src/web_midi/requests.rs` (`AccessRequests`), which the browser adapter uses and native tests drive: the latest request's refusal counts, a stale refusal does not, and a grant from any request wins. The pending message is re-exported as `console::console_midi::ACCESS_PENDING`, so the headless browser suite asserts against the constant.

**2026-11-01 — Toby: "Open firefox, no permission, no sound."** Scan alone did not help in Firefox. The console also made no sound: while access was pending it listed the Built-in Synth but picked nothing, and in Firefox access never settled. Automatic selection now picks the synth while access is pending and hands over to the first destination once discovery settles, unless the performer has chosen since (`built_in_synth.rs`'s `pending_midi_access_plays_through_the_synth_until_a_port_is_granted` and its two neighbours). Why Firefox shows no prompt, even for Scan's fresh request, is still open.
