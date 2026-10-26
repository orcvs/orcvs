# 02 — Send UDP from `!u`

Status: ready-for-agent
Type: task
Blocked by: 01

**What to build:** An activated `!u` sends its Payload as one UDP datagram through the Playback
Engine to a native network output adapter ([spec](../spec.md), ADR 0085).

- [ ] Evaluation turns the Payload into one command carrying its bytes: one per Cell, a space for an
      empty Cell. `!u!` emits nothing.
- [ ] The command keeps Tick Plan order with every other Terminal Output command.
- [ ] A network output adapter sends it with `std::net::UdpSocket` to the UDP destination, behind
      the native-only gate midir uses. No dependency is added.
- [ ] A failed send reports a diagnostic and playback continues. `ConnectionReset` is ignored with
      no diagnostic, because on Windows it reports an earlier datagram that found no listener; a
      test pins that it neither diagnoses nor stops the adapter.
- [ ] Stopping, rewinding and the MIDI safety action send nothing over UDP.
- [ ] A loopback test binds a local socket and receives the exact bytes.
