# 04 — Send Control Change and Pitch Bend

**What to build:** Implement fixed-arity terminal Functions `!c channel controller value` and
`!b channel lsb msb`, preserving direct MIDI wire bytes.

**Blocked by:** 01 — Generalize Play Commands for MIDI output.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Control Change emits the correct status, controller, and value bytes.
- [ ] Pitch Bend emits the correct status, LSB, and MSB bytes.
- [ ] Channel accepts `00`–`0F`; every data byte accepts `00`–`7F`.
- [ ] `controller` and `value`, and `lsb` and `msb`, each have their own type minted from `define_data_byte_roles!` in `lang/src/atom.rs`, so neither pair is assignable to the other. One shared data-byte type satisfies the domain checkbox above and does not satisfy this one.
- [ ] Each operand's domain is declared beside its role in `define_functions!` and converted during extraction, so neither Function body contains a validation call.
- [ ] A test per Function carries operand values that differ from one another, so a complete role transposition inside the declaration changes the answer. Asserting exact byte sequences does not cover this: a transposed declaration and a transposed expectation agree.
- [ ] Invalid operands diagnose and emit no command.
- [ ] No scaling, normalization, wrapping, or clamping occurs.
- [ ] Multiple commands retain Tick Plan and output-adapter order.
- [ ] Native delivery and in-memory adapter tests assert exact byte sequences.

## Comments

**The domain checkbox alone permits the swap this family is most exposed to.** `evaluation-machine/06` opened with the same wording — "each MIDI operand domain becomes a type" — and `40fb1a8` rewrote it after review, because read literally it permits one shared data-byte type: controller and value share a domain, as do lsb and msb, so a swap between either pair validates cleanly and stays invisible. Raw Play was never exposed to that, since channel and velocity have different domains. `!c` and `!b` are the two Functions where it is reachable, so the requirement matters more here than it did there.

**The machinery is already in place and unused.** `06` built `define_data_byte_roles!` in `lang/src/atom.rs`: a macro that mints a distinct public type per role over one shared private predicate, each carrying its own role word for `InterpretationError::MidiDataByte`. It mints only `Velocity` today. Adding a role is one line plus its arm in `operand_token!`, `operand_type!`, and `operand_bind!`; the generated `declaration_agreement` test catches a disagreement between the first and the third. Nothing enforces that this ticket uses it, which is why the requirement is written down rather than assumed.

**Naming `!c`'s value role needs a decision.** `lang::Value` is taken by the Sequence value model, so the Control Change value role cannot be `Value`. `06` deferred that call here rather than guessing at it. ADR 0016 also defers OSC and UDP output and notes that if a non-MIDI terminal family arrives, each type is better named for its domain than for its protocol.
