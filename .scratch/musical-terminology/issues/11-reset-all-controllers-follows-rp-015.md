# 11 — Reset All Controllers follows RP-015

**What to build:** A reset clears performance controllers without wiping the sound a musician set up.

**Blocked by:** 10 (both change controller handling and ADR 0099).

**Status:** resolved

- [x] RP-015's published wording is checked first, and the ADR cites it. The MIDI Association
      publishes RP-015 to members only, so its text was not read: the ADR cites the document by
      title and location and takes its reset list from receivers that implement it, and says so.
      It resets Modulation, so `MD` clears on Reset All Controllers while every other effect stays.
- [x] Reset All Controllers leaves the acid voice's sound controllers and every effect alone.
- [x] The reference documents how the Built-in Synth answers Reset All Controllers, All Notes Off and
      All Sound Off (which releases rather than cuts), and that the acid voice's accent is fixed with
      no tuning control.
- [x] ADR 0099 states the new behaviour and says "slide" for the acid voice throughout.
- [x] A synth byte test turns the acid voice's controls, sends Reset All Controllers, and finds them
      unchanged.
- [x] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.
