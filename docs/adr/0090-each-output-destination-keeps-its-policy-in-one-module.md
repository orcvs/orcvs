# Each output destination keeps its policy in one module

Status: accepted. Settled with Toby on 2026-10-28 in `.scratch/outputs/`. Amends
[ADR 0085](0085-udp-and-osc-send-the-cells-up-to-their-closer.md) on how a failed native send is
reported.

The Playback Engine delivers each Tick to two destinations, MIDI and the network, and they differ:
MIDI waits for a connection, owns note lifetimes and takes the safety action, and the network does
neither. Those rules were spread across the engine, the note schedule and the network output, and
each consumer filtered the other's commands out of one list.

## Decision

**One Outputs module in the Playback Engine owns every destination and its policy.** It takes a
Tick Plan and delivers it; it alone holds the MIDI adapter, the note schedule, the MIDI connection
state, each destination's failure latch and the network output. A third destination is one more
entry in it.

**A Tick Plan carries Network Sends in their own ordered list, beside the Play Commands.** A Play
Command is MIDI only. Order holds within each list. Across them, a Tick's MIDI is delivered before
its Network Sends, even where the Source interleaves `!>` and `!u`: splitting one MIDI submission to
interleave them would buy no audible difference.

**The network keeps sending while no MIDI destination is connected.** The MIDI connection gates
MIDI alone. Stopping, rewinding, disconnecting and the safety action reach MIDI alone, as ADR 0085
decided.

**A failed native network send is latched, as a MIDI failure is.** A failure is reported once
until it changes, and a successful send, a new destination and the start of a run each clear the
latch. The latched failure is published on the Playback observation, so the console sees a
recovery as the failure clearing. The web build still diagnoses every activated `!u` and `!o`, as
ADR 0085 decided, since there each send is the performer's evidence that nothing left.

**The network sits behind a seam that takes a Network Send and its destination.** The native
adapter encodes and sends one datagram; the web adapter refuses without encoding. The encoder is a
pure function. Tests record Network Sends, and one loopback test proves the real socket.

## Considered options

- **A thinner module that only dispatches,** leaving the schedule, connection state and latches in
  the engine. Deleting it would move nothing, so the policy would stay spread.
- **One list of commands that Outputs splits,** with UDP and OSC kept as Play Command variants. It
  keeps every MIDI command the size of the largest network one and keeps a Play Command from
  meaning MIDI.
- **Strict Tick Plan order across MIDI and the network.** It needs one MIDI submission per
  interleaved run.
- **Reporting every failed native send,** as ADR 0085's implementation did. A host that refuses
  every Tick floods the diagnostic log and gives the console no sign of recovery.
- **A seam that takes raw datagrams.** The web build would encode OSC only to refuse it.
