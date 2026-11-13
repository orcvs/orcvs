# 13 — Add legato to Monophonic Play

**What to build:** A musician plays legato on a mono synth: the acid voice and external mono synths
slide.

**Blocked by:** 12 (the operand type).

**Status:** ready-for-agent

- [ ] `!%` takes an optional last operand, `legato`, a Number: blank or `00` retriggers (Note Off
      before Note On, as today); anything else sends the new Note On before the previous Note Off.
- [ ] A legato note at the pitch it replaces keeps sounding to its new end, with no Note Off in
      between.
- [ ] The reference documents `legato` and says an external synth slides only with its portamento on.
- [ ] The glossary's Monophonic Play entry and signature, ADRs 0016 and 0099, and the scheduler's
      comments state the new behaviour.
- [ ] Runnable reference examples cover a retrigger, a slide, and two consecutive legato notes at the
      same pitch.
- [ ] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.
