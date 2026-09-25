# 04 — Make the browser build's MIDI Output agree with its reporting path

**What to build:** On the web build, the MIDI Output control presents the build as having no MIDI backend, and Playback failures reach the developer console, as the diagnostics reporting path documents. Today the no-backend stub declares itself available in the browser, so the web build shows an enabled Output picker whose Scan never finds a destination, and Playback failures go to the panel status instead.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The web build and native builds without a platform MIDI service report MIDI as unavailable.
- [x] On those builds the Output control is disabled, shows no Scan item, and Playback failures are reported through the no-backend path.
- [x] The redundant special case in the destination status that returns what the general fallback already returns is removed.
- [x] A WASM test asserts the unavailable presentation.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid; no change needed.

**2026-09-25 — implementation (epic PR 14).** Delivered in orcvs/orcvs#157. The no-backend stub's `AVAILABLE` is `false` on every target it serves, which disables the Output ComboBox, hides Scan and routes Playback failures through `report_playback_failures`. The `UNAVAILABLE` special case in `MidiDeviceSelection::status` is removed. `midi_output::the_browser_output_control_is_disabled_and_offers_no_scan` drives the running console with AccessKit in headless Firefox; it failed on the old constant. The native no-platform-service arm has no CI target, so it rests on the shared constant.
