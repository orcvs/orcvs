# 05 — Rename Delay to Every and Interpolation to Slew, with `division`

**What to build:** A musician recognises the clock divider and the slew limiter by name, and a
higher `division` reads as slower, as on a clock divider.

**Blocked by:** 01 (the settled names).

**Status:** resolved

- [x] Delay `~*` is Every and Interpolation `~>` is Slew; spellings unchanged. The Sound Module's
      echo effect keeps the name Delay.
- [x] Clock's and Every's `length` Role is `division`, so `length` only means a note's gate length.
- [x] The Rust variants and functions, the zero-cycle diagnostic's Role name, its error doc, Info,
      the reference, the console's Function Reference, the glossary's Clock, Delay and Interpolation
      entries, the example walk-throughs and the research notes use the new names, and the
      walk-throughs' "rate"/"modulus" operand labels go.
- [x] Slew's `amount` operand is reviewed against the decisions, and renamed or kept with a reason.
- [x] The reference glosses Clock as a step counter that holds each step `division` Ticks, Every as
      a clock divider firing every `division × steps` Ticks, with a note that Tidal's `every` means
      something else, and Slew as a slew limiter. The glossary allows "counter" as a gloss for Clock.
- [x] Runnable reference examples cover Every and Slew under their new names.
- [x] The glossary's "fixed delay" wording in the Built-in Synth entry no longer adds a third meaning
      of delay.
- [x] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.

## Answer

Resolved on 14 November 2026. The Function table in `lang/src/atom.rs` names `~*` Every and `~>`
Slew, with the variants `Function::Every` and `Function::Slew` and the bodies `tick::every` and
`tick::slew`, and names Clock's and Every's first operand `division`, so Info, the console's
Function Reference and the zero-cycle diagnostic ("cannot count a cycle with zero division") use
it. `docs/functions.md` glosses Clock as a step counter, Every as a clock divider with a note on
Tidal's `every`, and Slew as a slew limiter, and a new runnable example has `~*0401` Bang once a
beat. `CONTEXT.md`'s Clock, Every and Slew entries match, Clock no longer avoids "counter", and the
Built-in Synth entry says a fixed offset rather than a fixed delay. The example walk-throughs drop
their "rate"/"modulus" labels, the research notes use the shipped names, and a dated amendment to
ADR 0012 records the renames.

Slew's `amount` is kept. It holds the most one Tick moves the value, the same change per Tick that
Increment's `amount` holds, so one name means one quantity across the two feedback Functions. The
slew-limiter word for it is "rate", which decisions.md retires because Orca's `rate` read as
slower on Clock and faster on Interpolation; `amount` says how much without saying which way.
