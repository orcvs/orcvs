# 05 — Exhaustive arithmetic and Note conversion tests

**What to build:** Test the byte arithmetic and the MIDI Note conversions exhaustively, with loops
rather than properties. The domains are small enough to cover completely.

**Blocked by:** 01 — Add proptest for native targets.

**Status:** ready-for-agent

- [ ] `add`, `subtract`, and `multiply` are checked over all 65,536 `u8` pairs and always wrap.
- [ ] `divide` errors for every zero divisor and returns a value for every non-zero divisor.
- [ ] `midi_number_to_note` and `midi_note_to_number` round-trip over every value from `00` to `7F`.
- [ ] Every value above `7F` is rejected as a Note.
- [ ] `str_to_num` accepts exactly the two-character uppercase hexadecimal spellings, over all 256 values.
- [ ] The tests are fast enough for the pull-request tier.

## Comments

Do not use proptest here. Two `u8` operands is 65,536 pairs and one MIDI value is 128 cases. An
exhaustive loop proves the law; a 32-case or 256-case sample only fails to disprove it.

ADR 0011 is the claim under test: "General arithmetic wraps within this byte range." `add_impl`,
`subtract_impl`, and `multiply_impl` use the wrapping operations, and `divide_impl` returns
`DivisionByZero`. Division is the asymmetry, so state it in the test names rather than leaving a
reader to notice.

ADR 0021 defines the `.v` and `.^` identity laws. Those Functions do not exist yet, so this issue
covers only the conversion helpers in `lang/src/lib.rs`. Add the Function-level laws when
`orcvs-language-migration/05` lands.
