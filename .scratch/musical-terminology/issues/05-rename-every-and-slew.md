# 05 — Rename Delay to Every and Interpolation to Slew, with `division`

**What to build:** A musician recognises the clock divider and the slew limiter by name, and a
higher `division` reads as slower, as on a clock divider.

**Blocked by:** 01 (the settled names).

**Status:** ready-for-agent

- [ ] Delay `~*` is Every and Interpolation `~>` is Slew; spellings unchanged. The Sound Module's
      echo effect keeps the name Delay.
- [ ] Clock's and Every's `length` Role is `division`, so `length` only means a note's gate length.
- [ ] The Rust variants and functions, the zero-cycle diagnostic's Role name, its error doc, Info,
      the reference, the console's Function Reference, the glossary's Clock, Delay and Interpolation
      entries, the example walk-throughs and the research notes use the new names, and the
      walk-throughs' "rate"/"modulus" operand labels go.
- [ ] Slew's `amount` operand is reviewed against the decisions, and renamed or kept with a reason.
- [ ] The reference glosses Clock as a step counter that holds each step `division` Ticks, Every as
      a clock divider firing every `division × steps` Ticks, with a note that Tidal's `every` means
      something else, and Slew as a slew limiter. The glossary allows "counter" as a gloss for Clock.
- [ ] Runnable reference examples cover Every and Slew under their new names.
- [ ] The glossary's "fixed delay" wording in the Built-in Synth entry no longer adds a third meaning
      of delay.
- [ ] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.
