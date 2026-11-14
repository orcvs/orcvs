# 06 — Explain Ticks, Notes and numbers

**What to build:** A musician can count bars, spell any note, and move between the numbers their
DAW shows and the hex they type.

**Blocked by:** 05 (it glosses the timing Functions under their new names).

**Status:** resolved

- [x] The reference's reading guide and the glossary's Tick entry say a Tick is a sixteenth note,
      four to a beat at the displayed BPM, and that a DAW's "tick" is a much finer subdivision.
- [x] The reference explains Note spelling: scientific pitch notation with middle C as `C4` (MIDI 60),
      that Ableton, Logic, Cubase and Bitwig show MIDI 60 as C3, lowercase letters as sharps, and a
      lookup from each flat (including C♭, F♭, E♯, B♯) to its Orcvs spelling.
- [x] Prose names controller numbers, velocities and interval sizes in decimal with the Source hex
      beside them ("CC74 (`4A`)"), and the reference carries a table of common values (octave
      `0C`, fifth `07`, velocity 100 `64`). The glossary's Built-in Synth entry no longer mixes
      bases in one sentence.
- [x] The headline arithmetic example is replaced by one a musician cannot read as an octave jump,
      and points to Up for transposition.
- [x] The reference glosses Increment as a wrapping counter, Random as a reproducible random value
      per Tick and Position that holds nothing between activations, and Track, Push, Bang, Halt and
      Raw Play in musician terms (Raw Play: you send the Note Off).
- [x] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.

## Answer

Resolved on 14 November 2026. `docs/functions.md`'s reading guide gains **Ticks**, **Notes** and
**Numbers** paragraphs: a Tick is a sixteenth note, four to a beat at the displayed BPM, unlike a
DAW's tick; Notes are scientific pitch notation with middle C, MIDI 60 (`3C`), as `C4`, which
Ableton Live, Logic Pro, Cubase and Bitwig Studio show as C3, lowercase letters are sharps, and a
table spells each flat, C♭, F♭, E♯ and B♯ included; and a table of common values gives each in
decimal beside its hex. The nested-arithmetic example is `.+.x020301`, answering `07`, followed by
a pointer to Up and Down for transposition (`#+C407` answers `G4`). Increment, Random, Track, Push,
Bang, Halt and Raw Play gain musician glosses, and the Pick, Control Change, Pitch Bend and Raw Play
prose names its numbers in decimal beside the hex. `CONTEXT.md`'s Tick entry states the sixteenth
note, its Built-in Synth entry names channels, controllers and the accent velocity in decimal
beside the hex, and Increment may be glossed as a wrapping counter.
