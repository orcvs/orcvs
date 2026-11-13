# 06 — Gloss the timing Functions in musician terms

**What to build:** A musician reading the Tick section of the Function reference recognises each
timing Function by the sequencer or modular word they already know, and the research notes stop
making claims the code contradicts.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Delay is glossed as a clock divider (one Bang every `length × steps` Ticks from Tick 0), and
      the reference says it is not the Sound Module's echo effect of the same name.
- [ ] Interpolation is glossed as a slew limiter (moves toward `target` by at most `amount` per
      Tick).
- [ ] Clock is glossed as a step counter driven by a divided clock, answering a Number, not a pulse.
- [ ] Increment is glossed as a wrapping counter and Random as a seeded sample and hold that
      changes every Tick.
- [ ] Where the glossary's avoid lists ban the musician word (counter, slew), the entry still
      allows it as an explanatory gloss.
- [ ] The Euclidean entry says its pattern matches Bjorklund's up to rotation, and the timing
      research notes no longer call it Bjorklund's distribution.
- [ ] The timing research notes use the operand names the Function table uses.
- [ ] Function names are unchanged; renaming Delay or Interpolation is out of scope.
- [ ] `mise run site_build` and `mise run site_test` pass.

## Comments

Source: `audit.md`, top findings 2 and 7, and Part 2.
