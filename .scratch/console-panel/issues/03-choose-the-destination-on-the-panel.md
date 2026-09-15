# 03 — Choose the destination on the Panel

**What to build:** The output destination is chosen from the Panel: a ComboBox and Refresh. The MIDI menu is gone. One destination. Empty copy is "No output destination." The first discovered destination is selected when nothing is. On a build with no native MIDI backend, the destination Readout is disabled with that copy and Refresh is hidden.

**Blocked by:** 01

**Status:** resolved

- [x] Readout order is BPM · Tick · Run Clock · destination · Refresh. BPM may still be the Tempo menu if 02 has not landed; the destination Readout still sits after Run Clock.
- [x] The destination control is a ComboBox of output destinations plus a Refresh control. There is no always-open list and no cycling by clicking the name.
- [x] When the last discovery returned no destinations, the copy is "No output destination" — not "not found" and not "No MIDI destinations found."
- [x] Discovery that returns a non-empty list while nothing is selected selects the first destination. A later Refresh does not steal a selection the user already made.
- [x] Refresh asks the Playback Engine to discover again. There is no periodic polling.
- [x] The MIDI menu is removed. File and View stay. A refused connect still reaches the performer next to the destination Readout.
- [x] On a target or build with no native MIDI backend, the destination Readout is shown disabled as "No output destination" and Refresh is hidden. BPM, Tick, and Run Clock remain.
- [x] Destination is not persisted. A new launch auto-selects the first destination again when one exists.

## Answer

Destination is an `egui::ComboBox` on the bottom Panel after Run Clock, with a Refresh button that calls `MidiDeviceSelection::refresh_destinations`. The MIDI menu is gone. Empty copy is the named constant `NO_OUTPUT_DESTINATION` (`"No output destination"`), used by `destination_selected_text` / `destination_presentation` and the ComboBox.

`MidiDeviceSelection::auto_select_first_if_unselected` runs each frame while drawing the ComboBox. If `selected_destination_id` is already `Some`, it returns without calling `select` — a later Refresh that publishes a different first item, or drops the chosen id, does not steal. Empty discovery leaves selection empty. Destination is not in `Console::save`; a new launch starts unselected and auto-selects the first discovered id again.

`destination_presentation(available, …)` is what `Console::ui` calls with `native_midi::AVAILABLE`. When `available` is false the ComboBox is drawn disabled with that empty copy and Refresh is hidden. Refused-connect status is the existing `midi.status()` colored label, now next to the ComboBox. File and View stay. ComboBox focus uses the same last-frame latch as BPM so `event_handler` does not take digits while the popup is open.

Files: `console/src/midi.rs`, `console/src/console.rs`, `console/src/diagnostics.rs`.
