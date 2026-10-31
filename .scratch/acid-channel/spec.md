# Acid channel

Toby asked on 2026-10-31 how close the Built-in Synth can get to a TB-303, and what is missing from
sound generation and control. The Knobs example's bassline plays channel `02`, a raw saw under the
tones' shared on/off envelope, which is about a fifth of a 303.

## What a 303 is, and what the synth lacks

| 303 | Built-in Synth today |
| --- | --- |
| Saw or square oscillator, one switch | Naive saw on `02` and square on `03`; both alias at high notes |
| 24 dB/oct resonant lowpass (diode ladder) | No filter |
| Filter envelope: each note sweeps the cutoff down; decay and env-mod amount are knobs | No filter envelope |
| Accent per step: louder, filter opens further, decays faster | Velocity scales loudness only |
| Slide per step: mono voice, an overlapping note glides to its pitch (~60 ms) without retriggering | Sixteen-voice polyphony; every note retriggers |
| Knobs: cutoff, resonance, env mod, decay, accent | Control Change 1 to 8 set global effects (ADR 0088), of which only `MD` sounds so far; every other controller is ignored (ADR 0087) |

## What the language already has

- A Control Change every Tick, which is how the Knobs example sweeps its effects and how a 303's
  knobs are played.
- Velocity per Play, which can carry accent.
- A Timed Play whose length runs past its step, which overlaps the next note the way a 303 ties a
  slid step. Monophonic Play stops the prior note before it starts the next, so it cannot slide.

So no new Function is needed. The work is one voice in the synth core and the Control Changes that
reach it on channel `09`, which today only repeats the kit on `08`, decided in [ADR 0096](../../docs/adr/0096-one-built-in-synth-channel-plays-a-303-style-voice.md).

## Prototype

Branch `prototype/acid-channel` on `origin`, `synth/prototype/acid/`: renders the same sixteen-step
line on the acid voice and on channel `02` as it is, at 130 BPM for eight bars. The renders are not
in the repo; the branch's README says how to make them.
