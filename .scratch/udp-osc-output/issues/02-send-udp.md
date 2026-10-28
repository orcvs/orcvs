# 02 — Send UDP from `!u`

Status: resolved
Type: task
Blocked by: 01

**What to build:** An activated `!u` sends its Payload as one UDP datagram through the Playback
Engine to a native network output adapter ([spec](../spec.md), ADR 0085).

- [x] Evaluation turns the Payload into one command carrying its bytes: one per Cell, a space for an
      empty Cell. `!u!` emits nothing.
- [x] The command keeps Tick Plan order with every other Terminal Output command.
- [x] A network output adapter sends it with `std::net::UdpSocket` to the UDP destination, behind
      the native-only gate midir uses. No dependency is added.
- [x] A failed send reports a diagnostic and playback continues. `ConnectionReset` is ignored with
      no diagnostic, because on Windows it reports an earlier datagram that found no listener; a
      test pins that it neither diagnoses nor stops the adapter.
- [x] Stopping, rewinding and the MIDI safety action send nothing over UDP.
- [x] A loopback test binds a local socket and receives the exact bytes.
