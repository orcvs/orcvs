# 04 — Document Note spelling

**What to build:** A reader can write any pitch as an Orcvs Note: they know middle C is `C4`,
that lowercase letters are sharps, and how to spell a flat.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] The Function reference states that Notes use scientific pitch notation: `C4` is MIDI note
      60 (middle C), `/` is octave −1 and `G9` is 127.
- [ ] It states that only sharps exist, written as lowercase letters (`c` is C♯), and gives a
      flat-to-sharp lookup (E♭ is `d`, B♭ is `a`, and so on).
- [ ] The glossary's Note entry carries the same facts in brief.
- [ ] `mise run site_build` and `mise run site_test` pass.

## Comments

Source: `audit.md`, Part 1 section 1.
