# 08 — Assert the Cell invariant the Source actually has, on the Tick path

**What to build:** The one assertion standing between an interpreted result and the Source states
the rule a Cell really has. Every other statement of that rule is the printable ASCII range
`0x20..=0x7e`; the Tick path asserts only `is_ascii`.

**Blocked by:** None.

**Status:** needs-triage

**Tags:** release/v1

- [ ] A Tick's committed result is held to the same rule a typed Cell is: one printable
      single-byte ASCII character.
- [ ] The rule is stated once, where a Cell's content is defined, rather than a third time on the
      Tick path.
- [ ] The existing suite passes unchanged; no Source that ticks today ticks differently.

## Comments

`Source::check_content` accepts `0x20..=0x7e`, the persistence `Deserialize` validates
`0x20..=0x7e`, and `LanguageMap::derive` refuses anything outside `0x20..=0x7e`. The Tick path
asserts `encoded.is_ascii()` in `tick::emit_expression_root`, which admits control bytes.

Nothing produces one today — every Interpreter result is a hexadecimal Number, a Note name, or a
Char echoing a Cell that was already validated — so this is a latent gap, not a live defect, and
it is not a soundness gap either: `set_source`'s unsafe block needs UTF-8 validity, and any ASCII
byte gives it that. What a control byte would break is the Source's own round trip, since
persistence refuses to read back what the Tick was allowed to write.

Found while rewriting that SAFETY comment to name the assertion it actually rests on. The comment
now records the discrepancy; this ticket removes it.

Worth pairing with the observation that `CellWrite.content` is a `char` and `commit_tick` narrows
it with `as u8`. The field is wider than the invariant, and the narrowing is lossy for anything
outside ASCII. Unreachable today — `execute` builds the only plans `commit_tick` sees — but the
type says less than the code assumes.
