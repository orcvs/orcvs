# 03 — Explain zero-based Pick and Stack positions

**What to build:** A musician using Pick or Stack can translate between the zero-based index the
Function takes and the one-based scale degree they think in.

**Blocked by:** 01 (both edit the chord material), 12 (Stack's Role becomes `index`). Owns the
Pick and Stack entries.

**Status:** ready-for-agent

- [ ] Every Pick and Stack example in the Function reference gives the musician's degree beside
      the index (index `04` of C major is G, the 5th degree, V).
- [ ] The glossary calls the zero-based operand `index` in both Pick and Stack, and uses "degree"
      only for the one-based musical explanation; it no longer both avoids "Degree" and describes
      Pick as picking a scale degree.
- [ ] `cargo nextest run --package orcvs --locked` passes (it checks every example, summary and
      heading in the Function reference), and `mise run site_build` and `mise run site_test` pass.

## Comments

Source: `audit.md`, Part 1 section 5 and top finding 6. Pick's octave wrap, Stack's two-position
step and Up/Down's semitones are already documented in the reference; this ticket does not
restate them.
