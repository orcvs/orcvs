# 02 — Rename Interval Sets

**What to build:** A musician writes chords and scales by the names lead sheets and mode tables use,
and reads in the reference what each name is.

**Blocked by:** 01 (the settled names).

**Status:** resolved

- [x] The filler is gone: triads `MA mi di +5`, diminished seventh `d7`, scales `io ae` and `pM pm`,
      dominants `07 09` beside `13`. Every other name is unchanged, and the uniqueness check across
      Interval Sets, Voicings and Functions still holds. No old name is kept as an alias.
- [x] `MA` and `mi` change meaning from scales to triads and nothing diagnoses a stale use, so
      every scale use of them in shipped examples, the console's Function Reference, the reference,
      the glossary and tests moves to `io` or `ae`; each moved use is checked by hand.
- [x] The Function reference describes every Interval Set in chord-quality terms, beside how a lead
      sheet writes the chord ("chord symbol" stays avoided as a name for an Interval Set), and lists
      the accepted near pairs: `mM`/`mm`, `pM`/`pm`, `io`/`lo`, `di`/`do`.
- [x] Runnable reference examples play a renamed triad, `d7` and `io`.
- [x] The glossary's Interval Set entry and ADR 0105 state the new names and rules.
- [x] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.

## Answer

Resolved on 13 November 2026. The Interval Set table in `lang/src/atom.rs` spells the new names;
the variants for the two scales are `IonianScale` and `AeolianScale`, beside the other modes.
`lang`'s literal test refuses every removed spelling, so no alias survives. Every scale use of `MA`
and `mi` in tests, the Function reference and the console's Function Reference asset was read by
hand and moved to `io` or `ae`; the shipped examples use no Interval Set. `docs/functions.md` gains
an Interval Sets section with each name's chord quality or scale, its lead-sheet spelling on C,
its semitones and the near pairs, and runnable examples play `C4MA`, `B3d7` and `C4io`.
`docs/research/chords-and-patterns.md` keeps the names it proposed, since it records a proposal.
