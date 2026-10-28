# The Panel routes Playback failures in one place

Status: accepted. Settled with Toby on 2026-10-28 in `.scratch/output-status/`. Builds on
[ADR 0090](0090-each-output-destination-keeps-its-policy-in-one-module.md), whose latches it
publishes.

The console drains the Playback Engine's diagnostics once a frame. The network status took its
failures out of that list first, MIDI selection showed whatever was left, and a build with no MIDI
backend sent the rest to the developer console instead. The leftover held the clock, start and
retune failures as well as MIDI's, so an engine failure read as a MIDI one, and which status showed
a failure depended on call order. A MIDI failure cleared only when a new destination was installed,
though the engine already latched it.

## Decision

**One console Playback status takes each frame's diagnostics and the Playback observation.** It
alone decides which Panel status a failure belongs to. MIDI selection keeps only its own messages,
a refused discovery, a refused connect and "Opening …", and the network setting keeps only the
value it refused.

**Engine failures have their own Panel status.** Clock, start and retune failures are shown beside
the MIDI and network statuses, drawn the same way. They are events, not states: the latest is shown
until the next run begins, and a newer one replaces it. A retune failure therefore stays for the
rest of its run.

**The observation publishes whether the latest MIDI delivery failed, beside the network flag.**
`midi_failed` clears once a delivery succeeds or a destination is installed, so the MIDI status
clears on recovery. It is not the latch: a new run clears the latch so that the next failure is
reported again, but only a delivery says whether the device recovered. A rewind while playing stops
and begins a run in one engine step, so a flag that cleared with the latch would hide a refused
safety action before the console drew it.

**No build branches on whether it has MIDI.** Without a MIDI backend no connection is ever
installed, and an adapter with no connection delivers nothing, so no MIDI failure can occur there;
the engine failures that branch existed for now have a status of their own. Each status keeps what
it did: network failures are shown and reported to the developer console, and MIDI and engine
failures are shown.

## Considered options

- **Reporting every failure to the developer console in every build.** It invents a rule for a
  case that cannot happen: a build without MIDI has no MIDI failure to report.
- **Keeping engine failures in the MIDI status.** A clock failure is not a MIDI fault, and it kept
  the call-order routing.
- **Clearing an engine failure when the run it ended is followed by Playing.** A rewind while
  playing begins a new run without leaving Playing, so the new run is read from its start instead.
