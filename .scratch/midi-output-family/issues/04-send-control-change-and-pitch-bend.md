# 04 — Send Control Change and Pitch Bend

**What to build:** Implement fixed-arity terminal Functions `!c channel controller value` and
`!b channel lsb msb`, preserving direct MIDI wire bytes.

**Blocked by:** 01 — Generalize Play Commands for MIDI output.

**Status:** resolved

**Tags:** release/v1

- [x] Control Change emits the correct status, controller, and value bytes.
- [x] Pitch Bend emits the correct status, LSB, and MSB bytes.
- [x] Channel accepts `00`–`0F`; every data byte accepts `00`–`7F`.
- [x] `controller` and `value`, and `lsb` and `msb`, each have their own type minted from `define_data_byte_roles!` in `lang/src/atom.rs`, so neither pair is assignable to the other. One shared data-byte type satisfies the domain checkbox above and does not satisfy this one.
- [x] Each operand's domain is declared beside its role in `define_functions!` and converted during extraction, so neither Function body contains a validation call.
- [x] A test per Function carries operand values that differ from one another, so a complete role transposition inside the declaration changes the answer. Asserting exact byte sequences does not cover this: a transposed declaration and a transposed expectation agree.
- [x] Invalid operands diagnose and emit no command.
- [x] No scaling, normalization, wrapping, or clamping occurs.
- [x] Multiple commands retain Tick Plan and output-adapter order.
- [x] Native delivery and in-memory adapter tests assert exact byte sequences.

## Comments

**The domain checkbox alone permits the swap this family is most exposed to.** `evaluation-machine/06` opened with the same wording — "each MIDI operand domain becomes a type" — and `40fb1a8` rewrote it after review, because read literally it permits one shared data-byte type: controller and value share a domain, as do lsb and msb, so a swap between either pair validates cleanly and stays invisible. Raw Play was never exposed to that, since channel and velocity have different domains. `!c` and `!b` are the two Functions where it is reachable, so the requirement matters more here than it did there.

**The machinery is already in place and unused.** `06` built `define_data_byte_roles!` in `lang/src/atom.rs`: a macro that mints a distinct public type per role over one shared private predicate, each carrying its own role word for `InterpretationError::MidiDataByte`. It mints only `Velocity` today. Adding a role is one line plus its arm in `operand_token!`, `operand_type!`, and `operand_bind!`; the generated `declaration_agreement` test catches a disagreement between the first and the third. Nothing enforces that this ticket uses it, which is why the requirement is written down rather than assumed.

**Naming `!c`'s value role needs a decision.** `lang::Value` is taken by the Sequence value model, so the Control Change value role cannot be `Value`. `06` deferred that call here rather than guessing at it. ADR 0016 also defers OSC and UDP output and notes that if a non-MIDI terminal family arrives, each type is better named for its domain than for its protocol.

### What was built

`!c` and `!b` are two rows in `define_functions!` — `[channel: MidiChannel, controller: Controller,
value: ControlValue]` and `[channel: MidiChannel, lsb: BendLsb, msb: BendMsb]` — and
`functions::control_change` and `functions::pitch_bend` destructure each into a `PlayCommand`
variant of its own. Neither body validates anything, for the reason `raw_play` and `timed_play`
do not: the domain is declared beside the role and converted during extraction, so a Function
that diagnoses has constructed no command to discard. `OutputCommand` gains the matching pair,
carried through `TimedNotes::deliver` unresolved exactly as issue 02 said it would be — neither
spelling owns a voice or is due at another Tick — and `MidiOutputAdapter::submit` assembles
`0xB0 | channel` and `0xE0 | channel`, LSB before MSB, in the one place in Orcvs that knows the
wire format.

### Naming the value role

`ControlValue`, with `Controller`, `BendLsb`, and `BendMsb` beside it. `Value` belongs to the
Sequence value model, and `MidiValue` was the obvious second choice and the wrong one: ADR 0016
defers OSC and UDP output and notes a type reads better named for its domain than for its
protocol, so a name whose qualifier is the wire has to be renamed the day a second wire arrives.
The qualifier that survives that is the thing the value belongs to — a control has a value on any
protocol — which is also what pairs it with `Controller` at a glance. `BendLsb` and `BendMsb` took
the same rule: bare `Lsb` and `Msb` name the halves of nothing in particular, and the next
fourteen-bit pair to arrive would have to either share them or explain why not.

The diagnostic words are unchanged and unchanged deliberately: the roles answer `controller`,
`value`, `lsb`, and `msb`, which is what `evaluation-machine/06` pinned in
`a_rejected_data_byte_names_the_operand_role_that_supplied_it` before any of these types existed.
That test now asserts each word through the type that mints it rather than through a hand-built
`InterpretationError`, so a role that inherited another's word is a failure rather than a
diagnostic naming an operand the Source never wrote.

### What the role types buy, and what they do not

They make the swap unrepresentable everywhere a command is read or rebuilt: `functions`
constructing the `PlayCommand`, `TimedNotes::deliver` rebuilding it as an `OutputCommand`, and any
future consumer of either. Writing `controller: value` in one of those does not compile.

They cannot see two things, and both are covered by tests instead. The first is a transposition of
the declaration itself — swap the role names *and* their types in `define_functions!` and every
field still holds its own type, so it compiles and every operand still validates. The second is
the wire assembly, where both bytes are `u8` by the time the adapter has them. So each Function has
a role test carrying three operand values that all differ and are all legal in all three positions
(`01`, `02`, `03`), asserted both through direct extraction and through Source text, and the
adapter has a byte test whose every byte differs from every other. Both were checked by mutation
rather than by assertion: transposing both declarations leaves the crate compiling and fails five
`lang` tests, and transposing the two data bytes in each `submit` arm fails the wire tests. An
exact byte sequence alone proves neither, because a transposed declaration and a transposed
expectation agree with each other.

`every_data_byte_reaches_a_control_change_or_pitch_bend_command_unaltered` enumerates all of
`00`–`7F` in each of the four data-byte positions rather than sampling it, because a scale, a wrap,
or a clamp answers a perfectly legal command and differs only in the value it carries.

### Left open

Nothing clears a controller or a bend. The Playback Engine's stop, disconnect, and destination
change send all-notes-off, which is CC 123 per channel: it silences notes and says nothing about
CC 121 Reset All Controllers or about centring the pitch wheel. A Source that bends a channel and
then stops leaves that bend standing on the device, and the next note sounds bent. ADR 0016 says
nothing either way, and neither does this ticket's checklist, so it is recorded here rather than
decided: whoever revisits the safety action decides whether "silence the device" means notes alone.

A bend is never assembled into its fourteen-bit value anywhere in Orcvs. That is ADR 0016's
"exposing MIDI's wire bytes directly" taken literally, and it is why `!b` needs no scaling rule; a
Source that wants to think in bend units needs a Function that converts, not a change here.

`!c` and `!b` are declared `Scalar`, like every terminal Function today, so a Sequence in any
operand position diagnoses rather than broadcasting — `stack.rs` pins that at every position for
both new operand structs. Extending the family over Sequences is `05`.

The `operand_bind!` arms for the six Number-declared domains now forward to one shared
`@number_domain` arm rather than repeating an eight-line body apiece. Four of those six differ in
nothing but the type they convert to, and a copied arm naming another role's type would still have
compiled; the generated `declaration_agreement` test would have caught a token that disagreed with
its bind, but not a bind that converted to the wrong domain of the same token.
