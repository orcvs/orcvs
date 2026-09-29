# 06 — The browser target opens its port the same way

**What to build:** A performer running Orcvs in the browser gets the same MIDI menu behaviour as on
the desktop: destinations discovered on the main thread and shown in the frame that asked, a port
opened there, and Playback delivering to it through the same open connection handed across the same
ordered queue. One description of MIDI selection covers both platforms, and the browser path stops
being the one that has to be reasoned about separately.

**Blocked by:** None — 03 landed; the architectural shape is in place.

**Status:** ready-for-human

**Tags:** release/v1

## Already landed

The console owns the backend on every target, calls discovery and connect synchronously, and queues
an open connection through `install` on the ordered Playback queue. The WASM build compiles with
that shape. What remains is wiring the Web MIDI API so the browser backend is functional rather
than a stub that lists nothing and refuses every connect.

- [x] The browser delivers an already-open connection to the Playback Engine on the ordered queue;
      the engine's task owns it exactly as it owns the native one.
- [x] Console frame reads stay nonblocking on the browser main thread, which has no blocking
      receive.
- [x] The WASM target compiles and the headless browser suite passes, per the repository
      verification contract.

## Remaining

- [x] Browser discovery and port opening happen on the main thread through the Web MIDI API and
      answer the caller that asked, matching the native shape.
- [ ] A performer can refresh destinations in the browser, select one, and hear output from it.
- [x] A build for a target with no MIDI service still falls back to the silent backend and offers an
      empty destination list rather than an error. Met today by the stub every non-native target
      builds (`console/src/native_midi.rs:137-175`); what this line owes is that it stays met once
      the browser has a real backend, since the browser will no longer be that target.

## Comments

**2026-09-24 — joins `release/v1`.** The release decision is to build Web MIDI: the WASM build promises MIDI output, as the release goal ("deterministic native and WASM playback") and ADR 0041 state. This issue blocks `v1-release/03`. Proof is fake-adapter tests of exact bytes and lifecycle on the browser path; physical-device evidence stays native-only under `v1-release/04`. Until this lands, the web console offers no MIDI destination: `AVAILABLE` is false on wasm, so the list is disabled and Refresh is hidden (see the 2026-09-29 audit).

**2026-09-29 — audit at `cad296df`.** The 2026-09-24 comment said the web console's MIDI list and Refresh are offered while `AVAILABLE` is true on wasm. That was false and is corrected in place: every target outside native macOS, Windows and Linux builds the stub backend (`console/src/native_midi.rs:137-175`), whose `AVAILABLE` is `false` (`:150`), whose `destinations()` answers `Ok(Vec::new())`, and whose `connect` refuses. `destination_presentation` then disables the destination list and hides Refresh (`console/src/midi.rs:48-63`, from `1033fcad`), and refused connects go to the developer console (`console/src/console/panel.rs:224-232`). So the silent-fallback line is met today by the stub; it stays open to be re-proven once wasm stops building that stub. Nothing references Web MIDI yet: `console/Cargo.toml`'s wasm `web-sys` features carry no MIDI entries, and no source mentions `MIDIAccess`. The two Remaining lines on discovery and audible output are unmet.

**2026-09-29 — Web MIDI backend.** The browser build now carries a real backend
(`console/src/web_midi.rs`, reached as `console::native_midi::NativeMidiBackend` on `wasm32`), and
ADR 0059 records how the synchronous `MidiBackend` contract meets Web MIDI's Promise. Access is
requested once, when the console builds its backend. The answer is kept on the main thread, and
every discovery and connect reads it without waiting. While the browser has not answered, both
say access is awaited, and the performer's next Scan asks again. Once access is granted,
`MIDIAccess.outputs` is enumerated and a port found exactly as the native backend does. A browser
without Web MIDI, or one that refuses access, lists nothing and reports no error. The connection
crosses into Playback through the existing `install`. It names its output by id and sends through
the kept access, so no JavaScript value crosses a `Send` bound and no `unsafe` is involved.
`AVAILABLE` is true on `wasm32`, so the Panel enables the destination list, offers Scan, and shows
Playback failures beside it.

Ticked with this comment:

- *Discovery and port opening on the main thread, answering the caller*: `WebMidiBackend`'s
  `destinations` and `connect` return to the frame that called them. `web_midi::tests` drive them
  over a fake of Web MIDI's access, through the console's `MidiDeviceSelection` and a running
  Orcvs. The tests assert the exact Note On bytes the opened output receives and the full
  48-message safety action the outgoing output receives on a destination change. They also assert
  that the incoming output takes the next run's notes, that a disconnected output's refusal reaches
  the status line, and that a Scan after a pending request lists and auto-selects once access is
  granted. `console/tests/wasm.rs`'s `web_midi_answers_discovery_and_connect_without_waiting`
  builds the real backend in the headless browser suite, which runs in the merge tier.
- *Silent fallback*: a target that is neither native nor the browser still builds the silent
  backend, which `native_midi::silent::tests` now runs on every host. A browser without MIDI access
  answers an empty list with no status, proved by
  `a_browser_without_midi_access_offers_an_empty_list_rather_than_an_error`.

Left open: *a performer can refresh destinations in the browser, select one, and hear output from
it.* That needs a browser, a MIDI device and a person listening. Manual steps:

1. Connect a MIDI output (a hardware synth, or a software one on a virtual port such as the macOS
   IAC Driver or loopMIDI on Windows) and start a sound source listening to it.
2. `cd console && trunk serve --open`, in Chrome or Edge (Firefox needs the site-permission add-on
   it prompts for; Safari has no Web MIDI).
3. Allow MIDI access when the browser asks. Before that, the Panel's status reads "waiting for the
   browser to grant MIDI access".
4. Click the Output (`O`) readout; with an empty list that Scans. The device's port should be
   listed and selected.
5. Type `.=0101` on the first row and `!>007FC4` two rows below its first Cell, then press Space.
   The device should sound C4 on channel 1 every Tick.
6. Pick another port, or unplug the device. The first should fall silent (All Notes Off, Reset All
   Controllers, centred bend), and an unplugged device's refusal should appear beside the readout.
7. Deny MIDI access in the site settings and reload. The list should read `None` with no error.

