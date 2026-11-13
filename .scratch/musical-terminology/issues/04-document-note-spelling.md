# 04 — Document Note spelling

**What to build:** A reader can write any pitch as an Orcvs Note: they know middle C is `C4` and
how to spell a flat.

**Blocked by:** None — can start immediately. Owns the Note row of the operand-type table and the
glossary's Note entry.

**Status:** ready-for-agent

- [ ] The Function reference states that Notes follow scientific pitch notation: `C4` is MIDI
      note 60 (middle C), `/` is octave −1 and `G9` is 127.
- [ ] It gives a flat-to-sharp lookup (E♭ is `d`, B♭ is `a`, and so on), building on the
      existing statement that lowercase letters are sharps.
- [ ] The glossary's Note entry names scientific pitch notation and middle C.
- [ ] `cargo nextest run --package orcvs --locked` passes (it checks every example, summary and
      heading in the Function reference), and `mise run site_build` and `mise run site_test` pass.

## Comments

Source: `audit.md`, Part 1 section 1.
