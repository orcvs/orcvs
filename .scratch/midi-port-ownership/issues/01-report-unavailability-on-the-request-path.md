# 01 — Selection requests report unavailability on the request path

**What to build:** A performer whose running Orcvs has died — its Playback task ended while the
application it belongs to is still alive — sees the MIDI menu say so when they pick a device,
rather than watching the status line clear as though the device connected. Asking for a refresh or
selecting a destination answers on the same terms the menu's observations already answer on: if
there is no longer a running Orcvs to queue the request for, the caller is told, and the console
reports it instead of reporting success.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Requesting discovery or selection through the restricted selection handle answers
      unavailability whenever observing the published selection would answer unavailability. The
      request path and the read paths cannot disagree about whether a running Orcvs exists.
- [x] The documented claim that publication detects a task that ended while its owner remains alive
      holds for every method on the handle, not only the observing ones.
- [x] Selecting a destination against a dead engine leaves a visible message in the console; it does
      not clear the status line.
- [x] A test drives an adapter whose task ends while its owner is retained and asserts that a
      subsequent selection request reports unavailability. The existing adapter fake that models a
      failing output is prior art for constructing that state.
- [x] Existing behaviour when the owner is dropped is unchanged: requests and observations both
      report unavailability as soon as the owner is gone.
