# 07 — Play UDP and OSC through a running Orcvs to a loopback listener

Status: resolved
Type: task
Blocked by: 01, 02, 03, 04

**What to build:** A test in the shape of the MIDI loopback
(`console/src/console_midi/loopback_tests.rs`, `docs/tooling.md` "MIDI loopback"). A running
Orcvs plays a Source of `!u` and `!o` rows over several Ticks to a listener bound to a loopback
socket. The listener reads the wire itself and the test checks what arrived against what the Source
asked for, Tick by Tick ([spec](../spec.md), ADR 0085).

The seams are the running Orcvs, driven only through its public input and destination setting,
and the UDP wire the listener reads. Nothing in shipped code changes for the test.

A loopback socket needs no platform service, unlike a virtual MIDI port, so the test runs in the
ordinary suite on every native platform.

- [x] A Clock writes into each Payload, so every datagram carries the Tick that sent it and a
      Write into a Payload is seen to land in the same Tick.
- [x] The listener decodes OSC with its own reading of OSC 1.0, never the adapter's encoder, and
      refuses a packet that is not a complete, padded message.
- [x] Each Tick's UDP datagram and OSC message arrive once, in Tick Plan order.
- [x] Stopping sends nothing more, and the run reports no diagnostic.

## Answer

`orcvs/tests/network_loopback.rs` plays a three-row program through a running Orcvs to one
loopback listener that both the UDP and OSC ports name, so the order between a Tick's two sends is
observable. Two Clocks write the Tick into the Payloads, and Tick 0 must carry `00`, so a Write
that landed a Tick late would fail. The listener decodes OSC 1.0 itself and refuses unpadded or
incomplete packets, with its own tests including the specification's example. The run stops half
a period after Tick 7 and waits four more periods, then asserts exactly eight Ticks of sends and no
diagnostic. `mise run test_network_loopback` runs it alone.
