# 03 — Deliver each Tick through Outputs

Status: resolved
Type: task
Blocked by: 02

**What to build:** one Outputs module takes over the MIDI adapter, the note schedule, the MIDI
connection state, the MIDI latch and the network output from `PlaybackInner`, and the engine makes
one call per Tick and per lifecycle request ([spec](../spec.md)).

- [x] A Tick's MIDI is delivered before its Network Sends.
- [x] Network Sends go on while no MIDI destination is connected.
- [x] Stop, rewind, disconnect and the safety action send nothing to the network.
- [x] These are tested through Outputs' interface.
