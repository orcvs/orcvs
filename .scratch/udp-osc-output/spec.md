# UDP and OSC output

Status: resolved

## The decision

`!u` sends UDP and `!o` sends OSC, each from the Cells after its spelling up to the next `!` on
its row. Toby settled the design in a grilling on 2026-10-26, and
[ADR 0085](../../docs/adr/0085-udp-and-osc-send-the-cells-up-to-their-closer.md) records it. The
research behind it is [`docs/research/udp-osc-output.md`](../../docs/research/udp-osc-output.md).

The build comes after v1. `.scratch/v1-release/definition-of-done.md` keeps UDP and OSC as accepted
deferrals, so none of these tickets carries `release/v1`.

Native is the point. A browser cannot send UDP, so the web build only diagnoses.

## What a performer writes

```
!uhello!                    sends the datagram "hello"
!o/synth/freq 40 7F!        sends /synth/freq with int32 64 and int32 127
!o/synth/freq ,fs 64.0 hi!  sends /synth/freq with float32 64.0 and the string "hi"
!o/amp 0.5!                 sends /amp with float32 0.5
```

A Bang activates them as it activates any Terminal Output Function. A Write into the Payload
changes what the next send carries.

## Rules

- **Payload.** The Cells from the end of the spelling through the next `!` on the row, or the row's
  end. It is never parsed and cannot hold `!`. Only the spelling and the closer are occupied, as a
  Comment's delimiters are. The extent is fixed at parse and holds for the whole Tick, so a `!` a
  Write puts inside the Payload is sent as a character and closes it only from the next parse. The
  Function reads the Payload at its Turn, after every writer of those Cells.
- **UDP bytes.** One byte per Cell, a space for an empty Cell, no terminator, one datagram. `!u!`
  sends nothing.
- **OSC.** Words are split on spaces. The first word is the address and starts with `/`. An
  optional `,` type string of `i`, `f` and `s` follows. Without one, one to eight uppercase hexadecimal
  digits are an int32 whose bits they spell (`40` is 64, `1B8` is 440, `FFFFFFFF` is -1), a decimal matching `-?[0-9]+\.[0-9]+` is a float32, and anything else is a string
  (`ab`, `.5`, `5.` and `1e3` included). Under a type string, `i` needs one to eight uppercase
  hexadecimal digits, `f` such a decimal, and `s` takes the word as written.
  A bad address, an unknown letter, a count mismatch or a misfit word diagnoses and sends nothing.
- **Delivery.** One plain message per activation, in Tick Plan order, through the Playback Engine to
  a network output adapter. No bundles or timetags. A failed send is a diagnostic and never stops
  the performance. Stopping sends nothing on UDP or OSC.
- **Destination.** One host and two ports in the console's settings, defaulting to `127.0.0.1`,
  UDP `49161` and OSC `49162`, saved with the other settings.
- **Web.** The web build parses and paints both Functions as native does, and an activated one
  diagnoses that the build cannot send UDP or OSC.
- **Out of scope.** Receiving UDP or OSC, Application Command `!$`, OSC bundles and timetags, and
  type letters other than `i`, `f` and `s`.

## Constraints the research found

- The transport is `std::net::UdpSocket`, with no new dependency. On `wasm32-unknown-unknown` it
  compiles but `bind` fails at runtime, so the network adapter sits behind the same native-only
  gate as midir.
- OSC is encoded by hand: a padded address, a padded type tag string and big-endian arguments
  (OSC 1.0, "OSC Syntax"). `rosc` is the only mature crate and has not released since March 2025.
  If a decoder is wanted to check the encoder, it is a dev-dependency and needs the rationale the
  dependency policy asks for.
- On Windows, a send to a port with no listener reports `ConnectionReset` on a later send. It
  reports an earlier datagram, so it is ignored, not diagnosed; every other send error diagnoses
  (`docs/research/windows-support.md`, "OSC/UDP").
- The adapter seam is `OutputAdapter::submit` in `orcvs/src/playback.rs`, which the Playback
  Engine calls synchronously once per Tick.
