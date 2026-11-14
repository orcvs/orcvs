# 03 — Rename the drop 2&3 and drop 2&4 Voicings

**What to build:** A musician finds the drop voicings under the voices they drop.

**Blocked by:** 01 (the settled names).

**Status:** resolved

- [x] `D5` → `23` (drop 2&3) and `D6` → `24` (drop 2&4); `CL`, `OP`, `D2`, `D3`, `SH` and `RL`
      are unchanged. The Rust variants are named for the dropped positions, and the digit-sum
      explanation is gone from code and docs.
- [x] The reference gives each Voicing its standard name, says the two-Cell limit is why `23` and
      `24` carry no `D`, notes that `OP` on a four-tone chord equals `24` an octave up, what `SH`
      and `RL` keep, that `D3` on a triad is not a standard voicing, and that an Inversion at or
      above the tone count moves the chord up octaves.
- [x] Runnable reference examples play `23` and `24`.
- [x] The glossary's Voicing entry and ADR 0105 match.
- [x] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.

## Answer

Resolved on 14 November 2026. The Voicing table in `lang/src/atom.rs` spells `23` and `24`, with
the variants `DropTwoAndThree` and `DropTwoAndFour`; `lang`'s literal test refuses `D5` and `D6`
and reads `23` and `24` as a Voicing or a Number by position. `docs/functions.md` gains a Voicings
section giving each Voicing its standard name, why `23` and `24` carry no `D`, that `OP` on a
four-tone chord plays `24` an octave up, what `SH` and `RL` keep, that `D3` on a triad is not a
standard voicing, and that each step of an inversion raises the lowest tone, which moves a chord
within an octave up an octave per whole round but re-sorts a wider Interval Set; runnable
examples play `C4M7` voiced `24` and, inverted once, `23`. `CONTEXT.md`'s Voicing entry and a dated
amendment to ADR 0105 state the new names, and ADR 0103's Status line points to that amendment.
