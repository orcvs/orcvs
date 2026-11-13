# 11 — Reset All Controllers follows RP-015

**What to build:** A reset clears performance controllers without wiping the sound a musician set up.

**Blocked by:** 10 (both change controller handling and ADR 0099).

**Status:** ready-for-agent

- [ ] RP-015's published wording is checked first, and the ADR cites it.
- [ ] Reset All Controllers leaves the acid voice's sound controllers and every effect alone.
- [ ] The reference documents how the Built-in Synth answers Reset All Controllers, All Notes Off and
      All Sound Off (which releases rather than cuts), and that the acid voice's accent is fixed with
      no tuning control.
- [ ] ADR 0099 states the new behaviour and says "slide" for the acid voice throughout.
- [ ] A synth byte test turns the acid voice's controls, sends Reset All Controllers, and finds them
      unchanged.
- [ ] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.
