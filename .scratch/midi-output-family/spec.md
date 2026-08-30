# Implement the MIDI terminal-output Function family

**Status:** ready-for-agent

## Goal

Implement ADR 0016's fixed-arity Raw Play, Timed Play, Monophonic Play, Control Change, and Pitch Bend
contracts while preserving the Source/Playback Engine and output adapter seams.

## Delivery order

1. `issues/01-generalize-play-commands-for-midi-output.md`
2. `issues/02-schedule-timed-play-note-off.md`
3. `issues/03-own-monophonic-voices-per-channel.md`
4. `issues/04-send-control-change-and-pitch-bend.md`

## Required behavior

- Terminal Functions perform only when their root is active, return no language value, and never
  write ordinary result Cells.
- Each spelling has one fixed operand contract and validates direct hexadecimal MIDI domains.
- Tick Plan order is preserved through Playback Engine delivery.
- Stop, disconnect, and destination changes retain all-notes-off safety.

## Out of scope

- UDP and OSC until Orcvs has a message value.
- Application Command until its command value encoding is decided.
- Implicit Note/Number coercion, scaling, clamping, or optional operands.
