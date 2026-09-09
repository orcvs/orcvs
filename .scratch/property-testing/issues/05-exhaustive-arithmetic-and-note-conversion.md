# 05 — Exhaustive arithmetic and Note conversion tests

**What to build:** Test the byte arithmetic and the MIDI Note conversions exhaustively, with loops
rather than properties. The domains are small enough to cover completely.

**Blocked by:** None — every listed blocker is resolved.

**Status:** resolved

**Tags:** release/v1

- [x] Addition, ordered Subtraction, Absolute Difference, Multiplication, Modulo, Minimum, Maximum,
      and Equality are checked over all 65,536 Number pairs against their accepted laws.
- [x] `divide` errors for every zero divisor and returns a value for every non-zero divisor.
- [x] Modulo errors for every zero divisor; Equality proves its scalar Bang/no-value behavior.
- [x] `midi_number_to_note` and `midi_note_to_number` round-trip over every value from `00` to `7F`.
- [x] Every value above `7F` is rejected as a Note.
- [x] Evaluator-level `.v` identity and Note-to-Number conversion cover every valid MIDI value, and
      `.v Number` covers every byte.
- [x] Function-level `.^` identity and Number-to-Note conversion cover `00`–`7F`, while every Number
      from `80` through `FF` diagnoses and produces no result.
- [x] `str_to_num` accepts exactly the two-character uppercase hexadecimal spellings, over all 256 values.
- [x] The tests are fast enough for the pull-request tier.

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

2026-09-09 — Resolved. The checklist is delivered by exhaustive loops; no proptest strategy was
added, and none of the existing ones was extended to cover any of it.

Already in place and re-read against the checklist rather than rewritten: Add, Subtract and
Multiply over all 65,536 pairs in `interpreter::test::general_arithmetic_wraps_for_every_pair_of_bytes`
against `u16`/`i16` laws that never name a `wrapping_*` call; Absolute Difference, Modulo for every
non-zero divisor, Modulo for every zero divisor, Minimum, Maximum and Equality's scalar Bang and
`Atom::Empty` answers, each over the whole byte square; the `00`–`7F` Note round trip and the
`80`–`FF` rejection in `lib`; and `Note::try_from` refusing every value above `7F` in `atom`.

Added:

- `interpreter::test::division_is_the_asymmetry_that_diagnoses_every_zero_divisor` — a diagnostic
  for all 256 zero divisors and a Number for all 65,280 pairs that have a quotient, with the
  expected quotient counted by repeated subtraction so the assertion is floor division's definition
  rather than a second spelling of `left / right`.
- `interpreter::test::explicit_numeric_conversions_have_fixed_result_types` now runs `.v Number`
  over the whole byte domain (`00`–`FF`), not only the MIDI part of it; the typed conversions keep
  their `00`–`7F` sweep.
- `functions::numeric_conversion::test::conversion_to_note_is_the_identity_over_every_note_and_refuses_every_number_above_the_range`
  — Function-level `.^` identity over all 128 Notes and a `NoteConversion` diagnostic with no
  result for every Number `80`–`FF`.
- `test::str_to_num_accepts_exactly_the_uppercase_two_cell_hexadecimal_spellings` and
  `test::midi_note_to_number_accepts_exactly_the_pitch_spellings_inside_the_midi_range` — sweeps of
  all 16,384 two-character ASCII spellings, asserting exactly 256 and exactly 128 accepted. The
  Note sweep is what refuses `A9` and `B9`, which are spelled like every other Note and name 129
  and 131.

Runtime: the whole `lang` suite is 0.36 s wall. The slowest exhaustive test is
`general_arithmetic_wraps_for_every_pair_of_bytes` at 0.046 s; the new division sweep is 0.022 s,
the `str_to_num` sweep 0.019 s, the conversion tests 0.016 s and 0.010 s. Nothing here is near the
pull-request tier's budget, and no case count was sampled down.
