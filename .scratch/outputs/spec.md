# Outputs

Status: ready-for-agent

## The decision

Each Tick's MIDI and network output, and every rule that tells the two apart, live in one Outputs
module in the Playback Engine. Toby picked this from the 2026-10-28 architecture review (candidates
1 and 2) and settled it in a grilling the same day.
[ADR 0090](../../docs/adr/0090-each-output-destination-keeps-its-policy-in-one-module.md) records
it, and `CONTEXT.md` defines Network Send.

This builds on the UDP and OSC review fixes (branch `claude/project-thread-5iqvlt`). Where those
fixes add a thin UDP payload pointer and a wasm `cfg` that skips OSC encoding, tickets 01 and 02
replace them.

## Rules

- **Network Sends.** The Tick Plan carries `!u` and `!o` sends in their own ordered list. A Play
  Command is MIDI only, so nothing filters the other's commands out.
- **Outputs owns.** The MIDI adapter, the note schedule, the MIDI connection state, the MIDI failure
  latch, the network output and its latch. The engine calls it to deliver a Tick, stop, rewind,
  disconnect, install a MIDI connection and begin a run.
- **Order.** Within each list, Tick Plan order. Across them, MIDI first, then the Network Sends.
- **The connection is MIDI's.** Network Sends go on while no MIDI destination is connected. Stop,
  rewind, disconnect and the safety action reach MIDI only.
- **Native network latch.** A failed send is reported once until it changes. A successful send, a
  new network destination and the start of a run clear it. The latched failure is the "latest send
  failed" field of the Playback observation, so the console sees recovery as it clearing.
- **Web.** Every activated `!u` and `!o` still diagnoses (ADR 0085), with no encoding.
- **Seam.** One trait takes a Network Send and the destination. Adapters: native socket (encodes,
  sends, ignores Windows' `ConnectionReset`), web refusal, and a recording adapter behind the
  `test-output-adapter` feature. Outputs holds it boxed, so the engine gains no generic.
- **Destination changes.** The network destination stays on its `watch`; MIDI keeps the mailbox.

## Tests

Outputs is tested through its own interface with the in-memory MIDI adapter and the recording
network adapter: MIDI before network within a Tick, sends while MIDI is disconnected, nothing to
the network on stop, rewind or disconnect, and the latch. The engine tests that bind loopback
sockets go. `orcvs/tests/network_loopback.rs` stays the one real-socket test. The OSC encoder keeps
its pure tests.

## Delivery

One pull request, one commit per ticket, stacked on the review-fixes pull request.

## Out of scope

The console's routing of Playback diagnostics (review candidate 3) and the Panel's network control
(candidate 4). They read what this effort publishes.
