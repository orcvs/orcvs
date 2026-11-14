# 09 — Map the kit to General MIDI drum keys

**What to build:** A General MIDI drum part plays the right instruments on the Built-in Synth.

**Blocked by:** 08 (the kit's channels and the reference section).

**Status:** resolved

- [x] The kit answers General MIDI drum keys, each family with its GM neighbours (kick 35/36, snare
      38/40, clap 39, closed hat 42 and 44, open hat 46, toms, cymbals 49/51/57, cowbell 56, congas);
      every other key is silent and the octave-free mapping goes.
- [x] The pedal hat chokes the open hat, as GM's exclusive class does.
- [x] Shipped examples still play the instruments they name.
- [x] The reference's Built-in Synth section lists the kit's keys and says other keys are silent; the
      pitch-class wording is gone from the glossary, the example walk-throughs and the drum code's
      comments, and ADR 0087's 2026-11-13 amendment supersedes the Decision bullet that still
      states it, since an ADR is amended rather than rewritten.
- [x] Synth byte tests cover each family's keys and a silent key.
- [x] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.
