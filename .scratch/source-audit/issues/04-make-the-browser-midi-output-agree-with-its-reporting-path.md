# 04 — Make the browser build's MIDI Output agree with its reporting path

**What to build:** On the web build, the MIDI Output control presents the build as having no MIDI backend, and Playback failures reach the developer console, as the diagnostics reporting path documents. Today the no-backend stub declares itself available in the browser, so the web build shows an enabled Output picker whose Scan never finds a destination, and Playback failures go to the panel status instead.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] The web build and native builds without a platform MIDI service report MIDI as unavailable.
- [ ] On those builds the Output control is disabled, shows no Scan item, and Playback failures are reported through the no-backend path.
- [ ] The redundant special case in the destination status that returns what the general fallback already returns is removed.
- [ ] A WASM test asserts the unavailable presentation.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid; no change needed.
