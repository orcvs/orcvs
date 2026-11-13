# 01 — Describe Interval Sets in chord-quality terms

**What to build:** A musician reading the glossary or the Function reference recognises an
Interval Set as a chord quality or a scale, stored as its semitone formula above the root, and
can read every two-Cell name as the chord or scale they already know.

**Blocked by:** 09 (the names this ticket documents). Owns the Interval Set entries in the
glossary and the Function reference's operand-type table.

**Status:** ready-for-agent

- [ ] The glossary's Interval Set entry defines it as "a chord quality or a scale, stored as its
      semitone formula above the root". Its avoid list is otherwise unchanged.
- [ ] The Function reference's operand-type table uses the same wording and only the names ticket
      09 settles.
- [ ] The reference lists each chord name with the chord as lead sheets write it (for example
      `MA` is C, `h7` is Cø7 / Cm7♭5, `mM` is Cm(maj7)). The glossary avoids "chord symbol" as
      a name for an Interval Set; this column is an explanatory gloss, and the reference says so.
- [ ] `b9` is documented as the dominant seventh flat nine (C7♭9), not a bare ♭9.
- [ ] The reference says the three 13th chords (`13`, `MD`, `mD`) leave out the 11th.
- [ ] The reference explains the naming: triads from maj/min/dim/aug (`MA mi di au`), the modes by
      name (`io` Ionian, `ae` Aeolian), and `pM`/`pm` as major and minor pentatonic.
- [ ] `mm` is documented as the ascending (jazz) melodic minor and `bl` as the minor blues scale.
- [ ] `cargo nextest run --package orcvs --locked` passes (it checks every example, summary and
      heading in the Function reference), and `mise run site_build` and `mise run site_test` pass.

## Comments

Source: `audit.md`, "Is chord type the right term?" and Part 1 sections 2 and 3; the names come
from `decisions.md`.
