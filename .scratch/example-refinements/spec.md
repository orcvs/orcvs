# Example refinements

Status: ready-for-agent

Toby's ask (2026-10-29), on the shipped examples that `Help > Examples` lists on native and on the
site:

- Every example starts on Row 2, leaving one blank row above it.
- The 808's instruments stack, each under its own `|| name ||` label, so all three are visible on a
  phone.
- The 808 and the Cell Tracker get clearer names, chosen from options.
- "Orca" leaves the other examples' names: Tracker, Sequencer, Knobs.
- The Tracker and the Sequencer ship playing the basic 4/4 808 beat: kick `C2` on every beat,
  snare `D2` on beats 2 and 4, closed hat `f2` on every eighth.
- The Tracker's pattern rows lose their outer step marks: the Comment's own `||` marks the first
  voice's step on the left and the last voice's on the right.

A link names an example by its name in lowercase with a hyphen for each space (ADR 0094), so a
renamed example's link name changes with it. The examples' links shipped on 2026-10-29 with no
promise to keep them, so the old names are not kept as aliases.

## Decisions

- The beat's three voices play the Built-in Synth's kit on channel `08`, as the 808 does. The
  kit picks its instrument from the Note's pitch class, so the three share the channel; the
  Tracker's other voices keep channels `03` to `07` and the Sequencer's `03` to `09`.
- The voices keep the Learn Orca velocity `77`.
