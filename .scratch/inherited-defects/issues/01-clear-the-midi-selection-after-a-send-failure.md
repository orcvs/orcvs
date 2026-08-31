# 01 — Clear the MIDI selection after a send failure

**What to fix:** `MidiOutputAdapter::submit` drops the connection after a failed send, but it keeps
`selected_destination_id`. Playback then continues and sends nothing. The user gets no second
warning.

**Status:** resolved

- [x] After a send failure, the adapter and the shell agree that no device is connected.
- [x] A later `submit` does not report success while it sends nothing.
- [x] The shell shows the user that the device is disconnected.
- [x] The user can reconnect the same device from the MIDI menu.
- [x] A test drives a backend that fails one send, then asserts the reported selection.

## Comments

`orcvs/src/midi.rs:136-151`. On a send error, `submit` records the error, sends All Notes Off, sets
`self.connection = None`, and returns the error. It does not clear `self.selected_destination_id`.

Two results follow. First, `selected_destination_id()` at line 113 still names the dead device, so
the shell's MIDI menu keeps a tick beside a device that is not connected. Second, every later
`submit` takes the early return at line 137, because `self.connection` is now `None`. That early
return is `Ok(())`. The Playback Engine therefore sees success on every following Tick and records
no diagnostic.

The user sees playback that runs, a device that looks selected, and no sound. Nothing in the
interface explains this state.

Unplug a USB MIDI cable during playback to reproduce it.

Decide what the adapter should own. Either it clears the selection with the connection, or it keeps
the selection and marks it as disconnected so that it can reconnect. The second choice is friendlier,
because the user does not lose the device they chose. Whichever you choose, `submit` must not report
success for a Tick it did not deliver.
