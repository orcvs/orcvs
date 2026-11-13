# 02 — Rename Interval Sets

**What to build:** A musician writes chords and scales by the names lead sheets and mode tables use,
and reads in the reference what each name is.

**Blocked by:** 01 (the settled names).

**Status:** ready-for-agent

- [ ] The filler is gone: triads `MA mi di +5`, diminished seventh `d7`, scales `io ae` and `pM pm`,
      dominants `07 09` beside `13`. Every other name is unchanged, and the uniqueness check across
      Interval Sets, Voicings and Functions still holds. No old name is kept as an alias.
- [ ] `MA` and `mi` change meaning from scales to triads and nothing diagnoses a stale use, so
      every scale use of them in shipped examples, the console's Function Reference, the reference,
      the glossary and tests moves to `io` or `ae`; each moved use is checked by hand.
- [ ] The Function reference describes every Interval Set in chord-quality terms, beside how a lead
      sheet writes the chord ("chord symbol" stays avoided as a name for an Interval Set), and lists
      the accepted near pairs: `mM`/`mm`, `pM`/`pm`, `io`/`lo`, `di`/`do`.
- [ ] Runnable reference examples play a renamed triad, `d7` and `io`.
- [ ] The glossary's Interval Set entry and ADR 0105 state the new names and rules.
- [ ] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.
