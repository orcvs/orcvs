# 10 — Per-channel effects on standard MIDI controllers

**What to build:** A musician's MIDI controller knowledge works on the Built-in Synth, and turning
one channel's effects changes only that channel.

**Blocked by:** 08 (both change the synth's channel handling).

**Status:** resolved

- [x] Every effect is set per channel: `MD` vibrato CC1, `OF` offset CC85, `LN` length CC75, `EQ`
      tilt CC74, `R1` small reverb CC91, `R2` large reverb CC92, `CH` chorus CC93, `DL` delay CC94.
      Controllers 2–8 no longer set effects.
- [x] On the acid voice's channel its own controllers win: CC74 is its cutoff and CC75 its decay.
- [x] The Sound Module shows and sends each effect on a chosen channel, and its spec follows.
- [x] The Knobs example sets each effect on the channel it should change, and its test no longer
      says any channel will do.
- [x] The synth's parameter smoother is `Smoothing` (module and constants), and the test named for a
      glide says slide.
- [x] The reference's Built-in Synth section has the controller table with MIDI names: 91 and 93 the
      General MIDI reverb and chorus sends, 94 the delay send as on Roland GS modules, 92 and 85
      Orcvs's own. The `!c` examples in the reference and the console's Function Reference use the
      new controllers.
- [x] The glossary, ADR 0088 and the effect modules' doc comments state the new controllers. ADR
      0088 cites the Roland SC-88 Pro owner's manual ("Effect 4 (Delay Send Level)") for CC94.
- [x] Synth byte tests cover a per-channel send, a controller on one channel leaving another alone,
      and CC74 on `08`.
- [x] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.
