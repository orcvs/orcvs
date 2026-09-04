# 06 — Carry each MIDI operand domain in its type

**What to build:** Make each MIDI operand domain a validated type rather than a checked primitive, and declare which domain each operand has in `define_functions!`, so a transposed or unvalidated MIDI operand stops compiling instead of stopping a test.

**Blocked by:** None — can start immediately. It edits `define_functions!`, which `05` also edits; whichever lands second rebases.

**Status:** ready-for-agent

**Sources of truth:** ADR 0016 states every MIDI terminal Function's operand domains and says each one validates protocol ranges; `CONTEXT.md` says the output adapter alone assembles the wire message; `lang/src/atom.rs:9` is the `Note` newtype this follows; `03` established the operand role declaration this extends.

- [ ] Each MIDI operand domain is a type constructed only through a fallible conversion, in the shape `Note` already uses.
- [ ] No `PlayCommand` variant field is a bare `u8` that has a domain.
- [ ] The two `debug_assert!` calls at `orcvs/src/midi.rs:156` and `:160` are deleted, because the types carry what they were re-deriving.
- [ ] The domains are declared in `define_functions!`, so a new MIDI terminal Function inherits validation from its declaration rather than from a body someone remembers to write.
- [ ] Extraction still reports every arity and type diagnostic before any domain diagnostic.
- [ ] No Function body hands two same-typed operands to a positional call.
- [ ] ADR 0028 records that transposing two role names inside the single declaration cannot be compiler-checked, and names what carries it instead.
- [ ] The tests the types make unrepresentable are removed rather than left passing, and the coverage gaps listed below are closed.
- [ ] `PlayCommand` is a public type across three crates: the evaluator benchmark is run before and after with its command and results recorded, and the changed signature gets human API review.

## Comments

**The domains are already stated three times and enforced once.** ADR 0016 says channel is a Number in `00`–`0F` and velocity a Number in `00`–`7F`. `CONTEXT.md` repeats it in the Play Function entry and again in the Terminal Output Function entry. `lang/src/functions/mod.rs:12` and `:25` enforce it. But `midi_channel` and `midi_data_byte` each take a `u8`, prove a domain, and return the same `u8`, so the proof dies at the assignment. `lang/src/lib.rs:38` then declares `Raw { channel: u8, velocity: u8, note: u8 }`, and `orcvs/src/midi.rs:156` re-derives both domains with `debug_assert!` because it cannot trust three primitives. The domain is checked once and then thrown away twice.

**`Note` is the precedent, not a new idea.** `lang/src/atom.rs:9` is already a validated newtype whose `TryFrom<u8>` enforces `00`–`7F`, and it is already an operand type in `define_functions!`. `CONTEXT.md` defines it as one MIDI note value. So the operand-type column already holds a validated MIDI domain, and `channel` and `velocity` are the inconsistent entries rather than the novel ones. This ticket makes them match `note`.

**The blast radius is test ergonomics.** `PlayCommand::Raw` appears at 24 sites. Two are production — one construction at `lang/src/functions/mod.rs:50` and one destructure at `orcvs/src/midi.rs:148`. The other 22 are test constructions across `orcvs` and `shell/tests/wasm.rs`, which gain a fallible conversion each.

**Declaring the domain is what makes it survive the family growing.** ADR 0016 commits to Timed Play, Monophonic Play, Control Change `!c channel controller value`, and Pitch Bend `!b channel lsb msb`. Control Change and Pitch Bend each carry two operands over the same `00`–`7F` domain, so a transposition between them is invisible to both the compiler and the domain check — the exact failure `03` removed from Raw Play, arriving twice more. Giving each role its own type over one shared predicate stops those swaps compiling. Declaring the domains in the table rather than in each body is what makes a new terminal Function inherit validation instead of needing a body that remembers to call it.

**Widening the operand-type column is the real cost.** That column currently answers what the parser reads from two Cells. After this it also answers what domain the interpreter accepts, and the mapping from a domain type back to its parser token is where the two meanings meet. This is defensible as one refinement chain from Cells to Number to Channel, and `note` already behaves this way, but it is two questions in one column and should be a deliberate choice rather than a side effect. ADR 0016 defers OSC and UDP output; if a non-MIDI terminal family arrives, prefer naming each type for its domain rather than its protocol.

**Diagnostic order is preserved for free.** `Stack::extract` runs its whole pop loop before calling `from_operands`, so every arity and type diagnostic already fires before any domain bind runs. Making the bind fallible widens the error contract without reordering anything a test observes. Confirm this rather than assume it.

**Transposition inside the table is not solvable and should stop being an open question.** `define_functions!` is the single declaration, so there is no second source for the compiler to check it against: swapping two role names in one row cannot fail to compile, in principle rather than as a limitation of `macro_rules!`. A test that names both roles is the only thing that carries it. ADR 0028 should say so, so nobody designs for a guarantee that cannot exist.

**Remove the positional handoff rather than typing it.** Each of the nine `*_impl` helpers in `lang/src/functions/math.rs` has one caller and no test caller, so the seam is a pass-through, and it is the one place a body still hands two bare `u8` operands to a positional call one line after naming them. Deleting the helpers removes the site. Newtyping `left` and `right` instead would invent a role vocabulary that no domain justifies, which is the duplication ADR 0028 exists to prevent.

**Coverage.** Three current tests become dead weight once the types land: `play_rejects_channels_outside_the_midi_range` and `play_rejects_velocities_outside_the_midi_data_byte_range` collapse into constructor tests on the new types, and `play_carries_each_operand_into_the_role_its_signature_names` cannot fail to compile any more and should go rather than remain as a passing test of nothing. What is missing and matters: a totality property over parser-accepted Source, which is expected to fail immediately because it reaches the `Stack<16>` overflow that `pre-split-defects/15` owns and nothing currently exercises; an order test extended across every non-commutative Function rather than three of them; an assertion inside `lang` that Raw Play evaluates to a Play interpretation from its Source spelling, which only `orcvs` currently checks; and `TypeError::Numeric` at `lang/src/stack.rs`, which has no test and is reachable from Source as `.^.=0101`. Use exhaustive enumeration, not `proptest`, wherever the space is two bytes wide: the suite already enumerates 65,536 pairs in well under a second, and a sampled 32 cases is strictly weaker. Reserve `proptest` for the structural spaces, as `lang/src/parser.rs` already does.

**Not in scope.** Whether the evaluation dispatch is derived from the table belongs to `04`, which should weigh a generated per-Function evaluation trait as one of its options: that shape would absorb this ticket's positional handoff and carry `05`'s value-or-effect distinction as an associated type, but it is a dispatch decision and does not belong here. The `Stack<16>` bound stays with `pre-split-defects/15`; this ticket adds the test that reaches it and must not restate that ticket's proof.
