# 06 — Carry each MIDI operand domain in its type

**What to build:** Make each MIDI operand domain a validated type rather than a checked primitive, and declare which domain each operand has in `define_functions!`, so a transposed or unvalidated MIDI operand stops compiling instead of stopping a test.

**Blocked by:** None — can start immediately. It edits `define_functions!`, which `05` also edits; whichever lands second rebases.

**Status:** resolved

**Sources of truth:** ADR 0016 states every MIDI terminal Function's operand domains and says each one validates protocol ranges; `CONTEXT.md` says the output adapter alone assembles the wire message; `lang/src/atom.rs:9` is the `Note` newtype this follows; `03` established the operand role declaration this extends.

- [x] Each MIDI operand *role* has its own type, with a private field and a fallible conversion, in the shape `Note` already uses. One shared public data-byte type does not satisfy this: `controller` and `value`, and `lsb` and `msb`, must not be assignable to one another.
- [x] No `PlayCommand` variant field is a bare `u8` that has a domain.
- [x] The two `debug_assert!` calls at `orcvs/src/midi.rs:156` and `:160` are deleted, because the types carry what they were re-deriving.
- [x] The domains are declared in `define_functions!` and converted during extraction, so no Function body contains a validation call and a new MIDI terminal Function inherits validation from its declaration.
- [x] Extraction still reports every arity and type diagnostic before any domain diagnostic, and a test pins that precedence rather than leaving it to the reading.
- [x] No Function body hands two same-typed operands to a positional call.
- [x] ADR 0028 records that transposing two role names inside the single declaration cannot be compiler-checked, and names what carries it instead.
- [x] Only the domain predicate moves into the new types' constructors; every test asserting which operand reaches which role survives, and `play_carries_each_operand_into_the_role_its_signature_names` is extended to run from Source text, keeping operand values that differ from one another so a transposition changes the answer.
- [x] The coverage gaps named below are closed, and each new type's conversion is tested exhaustively over all 256 inputs.
- [ ] `PlayCommand` is a public type across three crates: the evaluator benchmark is run before and after with its command and results recorded, and the changed signature gets human API review. — benchmark done; human API review outstanding.

## Comments

**The domains are already stated three times and enforced once.** ADR 0016 says channel is a Number in `00`–`0F` and velocity a Number in `00`–`7F`. `CONTEXT.md` repeats it in the Play Function entry and again in the Terminal Output Function entry. `lang/src/functions/mod.rs:12` and `:25` enforce it. But `midi_channel` and `midi_data_byte` each take a `u8`, prove a domain, and return the same `u8`, so the proof dies at the assignment. `lang/src/lib.rs:38` then declares `Raw { channel: u8, velocity: u8, note: u8 }`, and `orcvs/src/midi.rs:156` re-derives both domains with `debug_assert!` because it cannot trust three primitives. The domain is checked once and then thrown away twice.

**`Note` is the precedent, not a new idea.** `lang/src/atom.rs:9` is already a validated newtype whose `TryFrom<u8>` enforces `00`–`7F`, and it is already an operand type in `define_functions!`. `CONTEXT.md` defines it as one MIDI note value. So the operand-type column already holds a validated MIDI domain, and `channel` and `velocity` are the inconsistent entries rather than the novel ones. This ticket makes them match `note`. The shape to follow:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MidiChannel(u8);

impl TryFrom<u8> for MidiChannel {
    type Error = InterpretationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        (value <= 0x0F)
            .then_some(Self(value))
            .ok_or(InterpretationError::MidiChannel(value))
    }
}

impl MidiChannel {
    pub const fn value(self) -> u8 {
        self.0
    }
}
```

**Distinct public types, shared private validation.** Six roles over two predicates would be six near-identical `TryFrom` impls if written by hand, and one public type if collapsed — the first is boilerplate and the second gives up the protection this ticket exists for. Share the machinery privately instead and keep the public types distinct. `InterpretationError::MidiDataByte { role, value }` already carries the role word, so one private constructor taking that word serves every data-byte role, and each public type passes its own. That also keeps `a_rejected_data_byte_names_the_operand_role_that_supplied_it` true without change: the role word moves from an argument at the call site to a property of the type.

**The blast radius is test ergonomics.** `PlayCommand::Raw` appears at 24 sites. Two are production — one construction at `lang/src/functions/mod.rs:50` and one destructure at `orcvs/src/midi.rs:148`. The other 22 are test constructions across `orcvs` and `shell/tests/wasm.rs`, which gain a fallible conversion each.

**Declaring the domain is what makes it survive the family growing.** ADR 0016 commits to Timed Play, Monophonic Play, Control Change `!c channel controller value`, and Pitch Bend `!b channel lsb msb`. Control Change and Pitch Bend each carry two operands over the same `00`–`7F` domain, so a transposition between them is invisible to both the compiler and the domain check — the exact failure `03` removed from Raw Play, arriving twice more. Giving each role its own type over one shared predicate stops those swaps compiling. Declaring the domains in the table rather than in each body is what makes a new terminal Function inherit validation instead of needing a body that remembers to call it.

**Widening the operand-type column is the real cost.** That column currently answers what the parser reads from two Cells. After this it also answers what domain the interpreter accepts, and the mapping from a domain type back to its parser token is where the two meanings meet. This is defensible as one refinement chain from Cells to Number to Channel, and `note` already behaves this way, but it is two questions in one column and should be a deliberate choice rather than a side effect. ADR 0016 defers OSC and UDP output; if a non-MIDI terminal family arrives, prefer naming each type for its domain rather than its protocol.

**Diagnostic order is preserved for free.** `Stack::extract` runs its whole pop loop before calling `from_operands`, so every arity and type diagnostic already fires before any domain bind runs. Making the bind fallible widens the error contract without reordering anything a test observes. Confirm this rather than assume it.

**Transposition inside the table is not solvable and should stop being an open question.** `define_functions!` is the single declaration, so there is no second source for the compiler to check it against: swapping two complete role declarations in one row cannot fail to compile, in principle rather than as a limitation of `macro_rules!`. The types narrow the hole without closing it. A partial swap that moves the names and leaves the types, `[velocity: MidiChannel, channel: Velocity]`, becomes a type error at the `PlayCommand` construction. A complete swap that moves each name with its own type, `[velocity: Velocity, channel: MidiChannel]`, compiles clean: a struct pattern and field init shorthand both match by name, so the body is unchanged and every operand still lands in a legal domain. That is the original defect arriving through the declaration instead of through the body, and only a test catches it. ADR 0028 should say so, so nobody designs for a guarantee that cannot exist.

**Remove the positional handoff rather than typing it.** Each of the nine `*_impl` helpers in `lang/src/functions/math.rs` has one caller and no test caller, so the seam is a pass-through, and it is the one place a body still hands two bare `u8` operands to a positional call one line after naming them. Deleting the helpers removes the site. Newtyping `left` and `right` instead would invent a role vocabulary that no domain justifies, which is the duplication ADR 0028 exists to prevent.

**Coverage. A type carries a domain; a test carries the wiring.** Exactly one existing test relocates: `the_shared_midi_domains_accept_exactly_their_protocol_ranges` exercises `midi_channel` and `midi_data_byte`, which stop existing, and becomes a constructor test on each new type. Everything else stays, including the two that look like domain tests. `play_rejects_channels_outside_the_midi_range` and `play_rejects_velocities_outside_the_midi_data_byte_range` each assert which diagnostic a Function answers for an out-of-range operand, so a complete declaration swap changes which diagnostic fires and fails them; they guard wiring, not just a predicate. `play_carries_each_operand_into_the_role_its_signature_names` is the load-bearing one and must not be deleted: with every operand legal in every domain, it is the only thing separating a correct declaration from a transposed one. Extend it to run from Source text so it covers the parse and evaluation wiring as well as the extraction.

What is missing and matters: the order test at `lang/src/functions/math.rs` extended across every non-commutative Function rather than three of them; an assertion inside `lang` that Raw Play evaluates to a Play interpretation from its Source spelling, which only `orcvs` currently checks; and `TypeError::Numeric` at `lang/src/stack.rs`, which has no test and is reachable from Source as `.^.=0101`. Use exhaustive enumeration, not `proptest`, wherever the space is two bytes wide: the suite already enumerates 65,536 pairs in well under a second, and a sampled 32 cases is strictly weaker. Reserve `proptest` for the structural spaces, as `lang/src/parser.rs` already does.

**Not in scope.** Whether the evaluation dispatch is derived from the table belongs to `04`, which should weigh a generated per-Function evaluation trait as one of its options: that shape would absorb this ticket's positional handoff and carry `05`'s value-or-effect distinction as an associated type, but it is a dispatch decision and does not belong here. The `Stack<16>` bound stays with `pre-split-defects/15`, and so does the test that reaches it. A totality property over parser-accepted Source is the right shape for that ticket's fourth checkbox, which already requires a regression test that evaluation does not panic, but it cannot be added here: it would fail on arrival, and a ticket must not require a test that its own scope forbids it to make pass.

**Resolution note.** Two decisions the checkboxes above did not settle.

*Which roles were minted.* `MidiChannel` and `Velocity` exist; `controller`, `value`, `lsb`, and `msb` do not, because `!c` and `!b` do not. What the ticket asked for structurally is in place: `define_data_byte_roles!` in `lang/src/atom.rs` mints a distinct public type per role over one private predicate, and adding a role is one line with its diagnostic word. Minting the other four now would ship public API for Functions that have no evaluation, and the Control Change value role would have to be named around `lang::Value`, which the Sequence value model already owns. That naming call belongs with the ticket that adds `!c`, alongside the ticket's own open question about naming a type for its domain rather than its protocol.

*The `Eq` derive.* Dropped. The ticket's code block derives it; `Note`, `Atom`, `Function`, and `PlayCommand` in the same files stop at `PartialEq`. A `Velocity: Eq` sitting beside a `Note: !Eq` of identical shape is the mismatch, so the new types match their neighbours. Nothing needs `Eq` today; the first thing that does should add it to `Note` in the same change.

**Verified by mutation, since `cargo-mutants` cannot see macro-generated bodies.** Each mutation was applied with `sed`, run under `cargo nextest run --package lang --locked`, and reverted from a backup copy.

- Complete role swap, `[velocity: Velocity, channel: MidiChannel, note: Note]` — compiles, and fails four tests: the role test, both range tests, and the Source-level domain test.
- Partial role swap, `[velocity: MidiChannel, channel: Velocity, note: Note]` — two type errors at the `PlayCommand::Raw` construction, as the ticket predicted.
- `Modulo => [right: Number, left: Number]` — fails the enumerated order test.
- `subtract`'s body transposed — fails the enumerated order test.
- `MidiChannel` widened to `0x1F`, and the shared data-byte predicate widened to `0xFF` — each fails both domain tests.

**Benchmark.** `cargo bench --package lang --benches --locked -- --output-format bencher`

- before (`40fb1a8`), two runs: `parse 64/64, parse_invalid 42/41, execute 12/12, parse_source 309/309` ns/iter
- after, four runs: `parse 63-66, parse_invalid 41-44, execute 12, parse_source 308-324` ns/iter

Unmoved, as expected of `Copy` newtypes over a byte. Two runs taken while another `cargo bench` was live on the same machine reported `parse 146` and `parse_source 623`, with reported variances of the same order; those are contention on paths this change does not touch, and the quiet runs are what is recorded. The spread above is every quiet run, not the best one.

**Two departures from the coverage list, both deliberate.**

*Two tests relocated, not one.* The ticket expected only `the_shared_midi_domains_accept_exactly_their_protocol_ranges` to move. `a_rejected_data_byte_names_the_operand_role_that_supplied_it` moved with it, because the predicate it exercises moved into `lang/src/atom.rs` beside the types. Its claim is unchanged: the five role words still pin the diagnostic wording, and `Velocity` is asserted to supply its own.

*Every non-commutative arithmetic Function is three of them.* The ticket asked for the order test "extended across every non-commutative Function rather than three of them". Subtract, Divide, and Modulo are the complete list; the other six answer the same for either operand order. The extension delivered is therefore in strength rather than in breadth: three named pairs became the whole 65,536-pair byte square, checked against a reference that names `left` and `right`.

**Carried forward to the Control Change and Pitch Bend tickets.** Because `controller`, `value`, `lsb`, and `msb` are not minted here, nothing in the code stops `!c` arriving with one shared data-byte type for its two same-domain operands — which is the failure this ticket exists to prevent, arriving one ticket later. `define_data_byte_roles!` makes the right answer one line each, but it is a convention, not a compile error. Whichever ticket adds `!c` or `!b` must require a distinct role type per operand as a checkbox of its own.

**Review surface.** The human API review this ticket defers covers four public types, not one: `PlayCommand`'s changed field types, plus `MidiChannel` and `Velocity` newly exported from `lang` and re-exported through `orcvs::source` alongside `Note`.
