# 05 — Exhaustive arithmetic and Note conversion tests

**What to build:** Test the byte arithmetic and the MIDI Note conversions exhaustively, with loops
rather than properties. The domains are small enough to cover completely.

**Blocked by:** property-testing/07 — Make Number and Note Source encodings canonical; orcvs-language-migration/05 — Add explicit Number and Note conversions; orcvs-language-migration/07 — Complete the numeric Function family.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Addition, ordered Subtraction, Absolute Difference, Multiplication, Modulo, Minimum, Maximum,
      and Equality are checked over all 65,536 Number pairs against their accepted laws.
- [ ] `divide` errors for every zero divisor and returns a value for every non-zero divisor.
- [ ] Modulo errors for every zero divisor; Equality proves its scalar Bang/no-value behavior.
- [ ] `midi_number_to_note` and `midi_note_to_number` round-trip over every value from `00` to `7F`.
- [ ] Every value above `7F` is rejected as a Note.
- [ ] Evaluator-level `.v` identity and Note-to-Number conversion cover every valid MIDI value, and
      `.v Number` covers every byte.
- [ ] Function-level `.^` identity and Number-to-Note conversion cover `00`–`7F`, while every Number
      from `80` through `FF` diagnoses and produces no result.
- [ ] `str_to_num` accepts exactly the two-character uppercase hexadecimal spellings, over all 256 values.
- [ ] The tests are fast enough for the pull-request tier.

## Comments

Do not use proptest here. Two `u8` operands is 65,536 pairs and one MIDI value is 128 cases. An
exhaustive loop proves the law; a 32-case or 256-case sample only fails to disprove it.

ADR 0011 is the claim under test: "General arithmetic wraps within this byte range." `add_impl`,
`subtract_impl`, and `multiply_impl` use the wrapping operations, and `divide_impl` returns
`DivisionByZero`. Division is the asymmetry, so state it in the test names rather than leaving a
reader to notice.

ADR 0021 defines the `.v` and `.^` identity laws. The Functions landed in
`orcvs-language-migration/05`; this issue now owns preserving their exhaustive Function-level laws
alongside the lower-level conversion-helper coverage.
