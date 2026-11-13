# 08 — Move the acid voice to `08` and the kit to `09`–`0F`

**What to build:** A General MIDI drum pattern sent to MIDI channel 10 plays drums, and a musician
can read which channel plays what.

**Blocked by:** 01 (the settled layout).

**Status:** resolved

- [x] The Built-in Synth plays the acid voice on `08` and the kit on `09`–`0F`; tones stay on
      `00`–`07`.
- [x] Shipped examples and site lessons send the bassline on `08` and drums on `09`, and the
      playback, site and console MIDI tests that pin channels follow.
- [x] The reference gains a Built-in Synth section whose channel table gives each channel in Orcvs
      and DAW numbering, explains that `00`–`0F` are MIDI channels 1–16, and notes that `09` is a
      drum channel as in General MIDI.
- [x] The glossary's Built-in Synth entry, ADRs 0087 and 0099, and the synth's doc comments state the
      new channels.
- [x] Synth byte tests cover the acid voice on `08` and the kit on `09`.
- [x] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset).
- [x] The site build and tests pass where the site renders it. CI's Site workflow carries this
      gate (`mise run site_build`, `mise run site_test`, against the commit's own web console); it
      was not run locally.
