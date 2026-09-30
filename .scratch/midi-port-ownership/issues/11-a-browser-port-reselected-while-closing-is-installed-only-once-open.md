# 11 — A browser port reselected while closing is installed only once open

**What to build:** A performer who picks a browser MIDI port again straight after leaving it either
hears output from it or sees why not. The console installs the connection only once the browser has
really opened the port, and a refusal reaches the status line, as the spec requires: "a queued
request always carries a connection that opened".

**Blocked by:** None — the Web MIDI backend from 06 is in place.

**Status:** needs-triage

## The gap

Reselecting a port just after its last connection dropped waits for the browser's close to answer
before opening it again (`a_new_claim_waits_for_an_in_flight_close_before_reopening` in
`console/src/web_midi.rs`). Past that point the console trusts the browser. If a browser resolves
`close()` before it has let go of the device, or resolves the following `open()` while the port is
still closing, the connection is installed as open. Web MIDI then reopens the port implicitly on
the next `send`, and a refusal there reaches no status line.

Reaching it takes that exact timing and an exclusively held device. Browser behaviour here is
unverified, and the fake Web MIDI access in `web_midi::tests` cannot reproduce it.

- [ ] Establish whether any supported browser (Chrome, Edge, Firefox with its add-on) settles
      `close()` or `open()` before the device is actually released or acquired. Proof needs
      `midi-port-ownership/06` manual step 8 run with a reselect straight after the port was left.
- [ ] If one does, the console confirms the port is open (for example from its `connection` state)
      before installing it, or reports the implicit reopen's refusal on the status line.
- [ ] If none does, record the evidence here and resolve as `wontfix`.

## Comments

**2026-09-30 — split from 06.** Recorded on `midi-port-ownership/06` by `7e99c280` and moved here so
the gap outlives 06. Not tagged `release/v1`: whether it blocks the release is for triage to decide.
