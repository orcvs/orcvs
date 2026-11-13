# 02 — Give Voicings their standard names

**What to build:** A reader of the Voicing documentation learns each Voicing by the name jazz
texts use, and the edge cases a musician would not expect are stated.

**Blocked by:** 01 (both edit the chord material), 10 (the drop Voicing names). Owns the Voicing
and Inversion entries.

**Status:** ready-for-agent

- [ ] The glossary and the Function reference describe `23` as drop 2&3 and `24` as drop 2&4,
      using only the names ticket 10 settles.
- [ ] `SH` notes that on a seventh chord the kept tones are the root, 3rd and 7th, and that the
      R-7-3 shell is not available.
- [ ] `RL` notes that it gives the A-form rootless voicing on ninth chords, the B form with
      inversion `02`, and a 3-5 dyad on a triad.
- [ ] The reference notes that `OP` on a four-tone chord produces the same shape as `24` an
      octave higher.
- [ ] The reference notes that drop Voicings on five- and six-tone chords count positions across
      all tones, extensions included, and that `D3` on a triad is not a standard voicing.
- [ ] The Inversion entry states that an inversion at or above the tone count moves the whole
      chord up octaves.
- [ ] `cargo nextest run --package orcvs --locked` passes (it checks every example, summary and
      heading in the Function reference), and `mise run site_build` and `mise run site_test` pass.

## Comments

Source: `audit.md`, Part 1 section 4; names from `decisions.md`.
