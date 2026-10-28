# Playback status

Status: resolved

## The decision

The console routes each frame's Playback diagnostics in one Playback status module. Toby picked this
(candidate 3) from the 2026-10-28 architecture review and settled it in a grilling the same day.
[ADR 0092](../../docs/adr/0092-the-panel-routes-playback-failures-in-one-place.md) records it.

## Rules

- **One router.** `console::playback_status` takes the drained diagnostics and the Playback
  observation each frame. An output failure is MIDI's, a network failure the network's, and a
  clock, start or retune failure the engine's. An Overrun or an omission summary is no failure.
- **Own messages.** MIDI selection keeps a refused discovery, a refused connect and "Opening …";
  the network setting keeps the value it refused. Neither reads Playback diagnostics.
- **MIDI.** Shown until the observation's `midi_failed` clears, which a successful delivery or an
  installed destination does. A new run does not, so a rewind cannot hide a refused safety action.
- **Network.** Shown until the observation's `network_failed` clears or the destination changes,
  and each failure is reported to the developer console.
- **Engine.** The latest is shown until the next run begins, read from the observation's
  `run_started_at`.
- **No MIDI branch.** The `console_midi::AVAILABLE` routing branch and
  `diagnostics::report_playback_failures` go.

## Tests

The Playback status is tested through its own interface with made-up diagnostics and observations.
The MIDI selection and network setting tests that drained diagnostics go. The browser test of the
developer-console report drives the Playback status with a network failure.

## Delivery

One pull request after #327, one commit per ticket.

## Out of scope

The network destination as its own Panel control (candidate 4), which follows this.
