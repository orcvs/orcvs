# 03 — Explain zero-based Pick and Stack positions

**What to build:** A musician using Pick or Stack can translate between the zero-based index the
Function takes and the one-based scale degree they think in, and the glossary describes both
Functions consistently.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Every Pick and Stack example in the Function reference gives the musician's degree beside
      the index (index `04` of C major is G, the 5th degree, V).
- [ ] The glossary no longer both avoids "Degree" and describes Pick as picking a scale degree;
      one consistent rule covers Pick's `index` and Stack's `degree` Roles.
- [ ] The reference documents that Pick wraps up one octave per pass through the set rather than
      continuing to rise, including for sets wider than an octave.
- [ ] The reference documents that Stack takes every other entry of the set, which is stacking in
      thirds on seven-note scales but not on pentatonics or chords.
- [ ] Up and Down are described as transposition by semitones.
- [ ] Role names are unchanged; renaming them is out of scope.
- [ ] `mise run site_build` and `mise run site_test` pass, and the reference's example test passes.

## Comments

Source: `audit.md`, Part 1 section 5 and top finding 6.
