# 06 — Gloss the timing and sequencing Functions in musician terms

**What to build:** A musician reading the Function reference recognises each timing and
sequencing Function by the sequencer or modular word they already know, and the research notes
stop making claims the code contradicts.

**Blocked by:** 05 (it owns the `length` and `rate` glosses), 11 (Every and Slew), 16 (the
Euclidean rotation this ticket describes).

**Status:** ready-for-agent

- [ ] Every is glossed as a clock divider: one Bang every `rate × steps` Ticks from Tick 0.
- [ ] The glossary's Built-in Synth entry no longer uses "delay" for its scheduling latency, so the
      word means only the Sound Module's echo effect.
- [ ] Slew is glossed as a slew limiter: it moves toward `target` by at most `amount` per Tick.
- [ ] Clock is glossed as a step counter driven by a divided clock, answering a Number, not a pulse.
- [ ] Increment is glossed as a wrapping counter. The glossary avoids "Counter" as a name for Clock
      and Increment; the entries allow it as an explanatory gloss and say so.
- [ ] Random is glossed as a reproducible random value per Tick and Position: it holds nothing
      between activations, and two Randoms with the same seed at different Positions are
      independent.
- [ ] Track and Push are glossed as a step sequencer's read and write, Bang as a trigger, Halt as a
      per-Tick mute of the root below, and Raw Play as "Note On; you send the Note Off".
- [ ] The Euclidean entry describes the Bjorklund rotation ticket 16 settles, with an example.
- [ ] The timing research notes use the Function and operand names the Function table uses.
- [ ] `cargo nextest run --package orcvs --locked` passes (it checks every example, summary and
      heading in the Function reference), and `mise run site_build` and `mise run site_test` pass.

## Comments

Source: `audit.md`, top findings 2 and 7, and Part 2.
