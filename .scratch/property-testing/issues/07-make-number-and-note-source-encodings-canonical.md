# 07 — Make Number and Note Source encodings canonical

**What to build:** Give every Number and MIDI Note exactly one complete two-Cell Source encoding
that renders and parses in both directions, including the documented octave below zero.

**Blocked by:** None — can start immediately.

**Status:** resolved

**Tags:** release/v1

- [x] Number parsing accepts exactly two uppercase hexadecimal Cells for every value `00`–`FF`.
- [x] Single-Cell, lowercase, signed, empty, and over-length Number spellings diagnose.
- [x] Note conversion derives one arithmetic mapping for all values `00`–`7F`, from `C/` through
      `G9`, without inverse lookup tables.
- [x] Every valid Note renders as exactly two Cells and round-trips through contextual Note parsing.
- [x] Every value above `7F` is rejected as a Note and cannot render as non-canonical generated
      Source.
- [x] Exhaustive loops cover all 256 Numbers, all 128 Notes, and both conversion directions without
      adding a property-testing dependency.

## Comments

The exhaustive wrapping-arithmetic coverage already landed with the byte-arithmetic migration and
is not repeated here. This ticket has no dependency on the separate property-testing setup: these
finite domains are small enough to enumerate completely.

Implemented by `2d86fcd` and `2b5a78a`. Freshly compiled exhaustive tests cover all 256 Number
encodings, all 128 Note encodings, contextual Note parsing, both Note conversion directions, and
rejection of non-canonical or out-of-domain inputs. The scoped `lang` and `orcvs` gates and
`mise run check_wasm` pass.
