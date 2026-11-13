# 01 — Describe Interval Sets in chord-quality terms

**What to build:** A musician reading the glossary or the Function reference recognises an
Interval Set as a chord quality or a scale, stored as its semitone formula above the root, and
can read every two-Cell name as the chord symbol they already know.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] The glossary's Interval Set entry defines it as "a chord quality or a scale, stored as its
      semitone formula above the root", and "chord quality" is no longer on its avoid list.
      "Chord type" may stay avoided as the informal synonym.
- [ ] The Function reference's operand-type table uses the same wording.
- [ ] The reference lists each chord name with its standard chord symbol (for example `h7` is
      Cø7 / Cm7♭5, `mM` is Cm(maj7)).
- [ ] `b9` is documented as the dominant seventh flat nine (C7♭9), not a bare ♭9.
- [ ] The reference says the three 13th chords (`13`, `MD`, `mD`) leave out the 11th.
- [ ] `p5` and `p3` state what the names stand for, or the reference gives them a stated mnemonic.
- [ ] `mm` is documented as the ascending (jazz) melodic minor and `bl` as the minor blues scale.
- [ ] `mise run site_build` and `mise run site_test` pass, and the reference's example test passes.

## Comments

Source: the musical terminology audit in `audit.md`, "Is chord type the right term?" and Part 1
sections 2 and 3. No semitone content changes; every list was checked correct.
