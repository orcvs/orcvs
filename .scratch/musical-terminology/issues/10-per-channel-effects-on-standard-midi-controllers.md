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
      Beyond what this asked, three additions keep it usable per channel: until the reader picks
      a channel, the chosen channel follows each Control Change heard; the channel buttons take
      the keyboard (Enter or Space chooses, the arrow keys move between them); and on `08` the
      `EQ` and `LN` columns read `CU` and `DC`, the acid voice's cutoff and decay.
      The synth also skips a silent channel whose effects rest, which holds because two fast
      paths pass the channel unchanged: a vibrato at zero swing returns the sample it was given,
      and a flat tilt returns its input.
- [x] The Knobs example sets each effect on the channel it should change, and its test no longer
      says any channel will do. Its bassline on `08` is the only part playing, and there CC74 and
      CC75 are the cutoff and decay its `CUT` and `DEC` knobs already turn, so it keeps six effect
      knobs (`MD`, `OF`, `R1`, `R2`, `CH`, `DL`) and has none for `EQ` or `LN`.
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
