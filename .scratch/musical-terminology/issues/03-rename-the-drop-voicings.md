# 03 — Rename the drop 2&3 and drop 2&4 Voicings

**What to build:** A musician finds the drop voicings under the voices they drop.

**Blocked by:** 01 (the settled names).

**Status:** ready-for-agent

- [ ] `D5` → `23` (drop 2&3) and `D6` → `24` (drop 2&4); `CL`, `OP`, `D2`, `D3`, `SH` and `RL`
      are unchanged. The Rust variants are named for the dropped positions, and the digit-sum
      explanation is gone from code and docs.
- [ ] The reference gives each Voicing its standard name, says the two-Cell limit is why `23` and
      `24` carry no `D`, notes that `OP` on a four-tone chord equals `24` an octave up, what `SH`
      and `RL` keep, that `D3` on a triad is not a standard voicing, and that an Inversion at or
      above the tone count moves the chord up octaves.
- [ ] Runnable reference examples play `23` and `24`.
- [ ] The glossary's Voicing entry and ADR 0105 match.
- [ ] The crate gates for every crate the change reaches pass, including `console` wherever the
      console compiles what changed (the examples or the Function Reference asset), and the site
      build and tests where the site renders it.
