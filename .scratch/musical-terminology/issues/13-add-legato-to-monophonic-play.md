# 13 — Add legato to Monophonic Play

**What to build:** A musician plays legato on a mono synth: the acid voice and external mono synths
slide.

**Blocked by:** 12 (the operand type).

**Status:** resolved

- [x] `!%` takes an optional last operand, `legato`, a Number: blank or `00` retriggers (Note Off
      before Note On, as today); anything else sends the new Note On before the previous Note Off.
- [x] A legato note at the pitch it replaces keeps sounding to its new end, with no Note Off in
      between.
- [x] The reference documents `legato` and says an external synth slides only with its portamento on.
- [x] The glossary's Monophonic Play entry and signature, ADRs 0016 and 0099, and the scheduler's
      comments state the new behaviour.
- [x] Runnable reference examples cover a retrigger, a slide, and two consecutive legato notes at the
      same pitch.
- [x] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.

A legato note on a silent channel starts as any note does, and velocity `00` or length `00` stops
the channel's note whatever the `legato`. `legato` takes two Cells east of `length`, so a `!%` now
needs twelve Cells; on the one Grid shape that matters only at its east edge, and the tests that
used ten-column Grids widened to twelve. The row declares `legato: Defaulted<Number, 0x00>`
directly, through the row grammar ticket 12 widened.
