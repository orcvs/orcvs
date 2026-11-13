# 01 — Describe Interval Sets in chord-quality terms

**What to build:** A musician reading the glossary or the Function reference recognises an
Interval Set as a chord quality or a scale, stored as its semitone formula above the root, and
can read every two-Cell name as the chord they already know.

**Blocked by:** None — can start immediately. Owns the Interval Set entries in the glossary and
the Function reference's operand-type table.

**Status:** ready-for-agent

- [ ] The glossary's Interval Set entry defines it as "a chord quality or a scale, stored as its
      semitone formula above the root". Its avoid list is otherwise unchanged.
- [ ] The Function reference's operand-type table uses the same wording.
- [ ] The reference lists each chord name with the chord as lead sheets write it (for example
      `h7` is Cø7 / Cm7♭5, `mM` is Cm(maj7)). The glossary avoids "chord symbol" as a name for
      an Interval Set; this column is an explanatory gloss of how musicians write the chord, and
      the reference says so.
- [ ] `b9` is documented as the dominant seventh flat nine (C7♭9), not a bare ♭9.
- [ ] The reference says the three 13th chords (`13`, `MD`, `mD`) leave out the 11th.
- [ ] `p5` and `p3` state what the names stand for, or the reference gives them a stated mnemonic.
- [ ] `mm` is documented as the ascending (jazz) melodic minor and `bl` as the minor blues scale.
- [ ] `cargo nextest run --package orcvs --locked` passes (it checks every example, summary and
      heading in the Function reference), and `mise run site_build` and `mise run site_test` pass.

## Comments

Source: the musical terminology audit in `audit.md`, "Is chord type the right term?" and Part 1
sections 2 and 3. No semitone content changes; every list was checked correct. The renames of the
filler-`_` names and the major and natural minor scales are a separate decision and are not part
of this ticket.
