# 02 — Give Voicings their standard names

**What to build:** A reader of the Voicing documentation learns each Voicing by the name jazz
texts use, so `D5` and `D6` read as drop 2&3 and drop 2&4, and the less standard Voicings say
exactly which tones they keep.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] The glossary and the Function reference introduce `D5` as drop 2&3 and `D6` as drop 2&4,
      leading with the standard name and explaining that the digit is the sum of the dropped
      positions counted from the top.
- [ ] `SH` is documented as keeping the first, second and fourth tones (root, 3rd and 7th on a
      seventh chord), and that the R-7-3 shell is not available.
- [ ] `RL` is documented as removing only the root, giving the A-form rootless voicing on ninth
      chords and the B form with inversion `02`, and a 3-5 dyad on a triad.
- [ ] The reference notes that `OP` on a four-tone chord produces the same shape as `D6` an
      octave higher.
- [ ] The reference notes that drop Voicings on five- and six-tone chords count positions across
      all tones, extensions included.
- [ ] Spellings and behaviour are unchanged; renaming `D5`/`D6` is out of scope.
- [ ] `mise run site_build` and `mise run site_test` pass, and the reference's example test passes.

## Comments

Source: `audit.md`, Part 1 section 4.
