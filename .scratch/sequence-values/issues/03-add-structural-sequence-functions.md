# 03 — Add structural Sequence Functions

**What to build:** Implement the four structural Sequence Functions — Reverse `:<`, Concatenate
`:&`, Select `:?`, and Replace `:=` — with the exact contracts in ADR 0007.

**Blocked by:** None — 06 and grid-boundedness/01 are resolved.

**Status:** resolved

**Tags:** release/v1

- [x] Reverse changes Atom order but never reverses an Atom encoding.
- [x] Concatenate promotes Atoms, stays flat, and treats empty Sequence as identity.
- [x] Select uses a Number index modulo non-empty Sequence length.
- [x] Replace returns a new same-length Sequence and permits a different replacement Atom type.
- [x] Empty and invalid operands diagnose as ADR 0007 specifies.
- [x] Every Function parses and round-trips through its canonical two-Cell spelling.
- [x] Structural operations preserve a Bang member's type and encoding. (From issue 01, which
      covers Bang only through construction, promotion, and encoding.)
- [x] All four stay generic over Atom type, because none of them reinterprets an Atom.

## Comments

### Scope correction, 2026-09-04

This issue originally also owned Range `:-`. It does not any more. ADR 0023 keeps the structural
Sequence Functions generic precisely because they do not reinterpret their Atoms, while the two
Range Functions are monomorphic and each fixes its own operand and result type. Carrying Range here
put a type-directed Function in the generic issue and left `:#` with no owner at all.

Range `:-` and `:#` are now owned together by `issues/05-add-the-range-functions.md`. The acceptance
line that read "Range handles ascending, descending, equal, Number, Note, and mixed-type bounds" is
not deleted, only moved: issue 05 carries the mixed-type diagnostic explicitly, because that is the
behaviour `orcvs-language-migration/04` settled and it must not be lost in the split.

### ADR 0026: validate the value model against a measured baseline, 2026-09-05

ADR 0026 names this issue as half of its own revisit trigger — the choice can be made "when the
broadcasting Atomic Functions and the structural Sequence Functions have landed". Broadcasting
landed in `sequence-values/02`. When this issue lands, the trigger is complete. Three things for
whoever picks that up.

**The ADR's cost paragraph is out of date on a point of fact.** It records that "Every scalar
allocates, which ... is not a live concern but is unmeasured". Both halves have moved. The scalar
path no longer allocates at all: `Stack::apply` and `Stack::convert` answer a `Shape::Scalar`
operation where they bind it, since `5c2d095`. And the cost is measured now. It is not allocation.
It is that `Value` is 24 bytes, is not `Copy`, and carries a `Drop` implementation because one of
its two variants owns a `Vec`. That drop glue stops the compiler holding the operand buffer in
registers, on every path, whether or not a Sequence exists. Amending the ADR is a call for a human;
this note only records that its premise moved.

**The measured baseline.** `lang` bench `execute` (`.+.+0101.-0A05`, three scalar Functions),
lowest of six interleaved rounds:

| revision | time |
| --- | --- |
| before broadcasting (`469971d`) | 12.1 ns |
| broadcasting, first cut (`46a19be`) | 135.8 ns |
| after the scalar fast path (`5c2d095`) | 56.6 ns |

The remaining 4.7x over the pre-broadcast base is the seam itself: the 24-byte non-`Copy` `Value`
move, `broadcast()` returning the whole structure by value with no inline attribute, and three reads
of the operands where the base did one. It was left standing deliberately. Reshaping `Broadcast` to
close it is the change most likely to make ADR 0026 harder to adopt later, and the uniform model
deletes the cause rather than working around it.

**What to validate, and against what.** Re-run the same benchmark and compare against 12.1 ns, not
against the current figure.

Do not assume the drop glue is the cost. That was the first hypothesis here and a measurement
refuted it. Backing `Sequence` with `ManuallyDrop<Vec<Atom>>` removes the `Drop` implementation from
`Value` entirely — a probe that leaks, and is only a probe — and it moved the bench from 40.7 ns to
38.5 ns against a 12.4 ns pre-broadcast base. Two nanoseconds of twenty-six. Ownership is not what
this costs.

Be careful what that refutes, because it is narrower than it first looks. The probe removes the
`Drop` implementation and nothing else: a `ManuallyDrop<Vec<Atom>>` Sequence is still 24 bytes and
still travels through the same two-variant `Value` enum, discriminant and all. What ADR 0026 removes
is the enum itself. The probe never tested that, so the uniform model's payoff is **unquantified,
not refuted**. Do not carry a number for it — including this one — until something measures a
one-shape representation directly.

Where the 26 ns sits is likewise not attributed. The candidates are the size of the `Broadcast`
structure, about 120 bytes moved for each operation whether or not it is dropped; the two-pass walk,
which validates every operand before it binds any; and the `ArrayVec<Atom, 4>` that `element()`
builds for each bind. The uniform model plausibly moves the first, which is a size argument rather
than an ownership one — but plausibly is the honest word. Attribute the 26 ns before designing
against it, and beat 12.4 ns.

Do not answer it by inlining the members. Backing `Sequence` with `ArrayVec<Atom, 256>` in place of
`Vec<Atom>` was measured on the same bench at 1581.8 ns, 125x the pre-broadcast base: `Value` must
be as large as its largest variant, so the operand buffer becomes about 8 KB and moves on every
operation. `Vec` is 24 bytes and is the smaller of the two.

**Method.** `cargo bench` compiles and links immediately before it measures, and the readings are
badly contaminated — one round of it read 53.8 ns for the base and 778 ns for the first cut. Build
the bench binary first, run the binary directly, alternate the revisions, and take the lowest
result.

### Verification, 2026-09-14

Verified all eight items against `main` (`87fc793`), per item, by reading the code that decides the
behaviour and then confirming a test drives it. Landed in `fcb1a40` with review follow-up in
`651f9d6`. ADR 0026's revisit trigger (broadcasting + these four Functions) is therefore complete;
the cost paragraph above is unchanged.

- **Reverse order, not encoding.** `reverse` copies members and reverses the vec
  (`lang/src/functions/sequence.rs`). Covered by `reverse_preserves_atom_type_and_encoding`, which
  round-trips Number / Note / Bang and Source `:<:-0003` → `03 02 01 00`.
- **Concatenate promote, flat, empty identity.** `AtomOrSequence` binding promotes
  (`lang/src/stack.rs`); `concatenate` extends one vec. Covered by
  `concatenate_promotes_atoms_stays_flat_and_treats_empty_as_identity`. Nested Sequences are
  unrepresentable (`Sequence` is flat), so flatness is the type.
- **Select modulo length.** `wrapping_index` is `usize::from(index) % length`. Covered by
  `select_uses_a_wrapping_number_index_and_preserves_the_chosen_atom` (`04` on length 3 → member 1).
- **Replace same length, different type.** Copies the vec, writes one slot, returns a new Sequence;
  the input is left unchanged. Covered by
  `replace_returns_a_new_same_length_sequence_and_allows_a_different_replacement_type`. Wrapping
  shares `wrapping_index` with Select; Replace has no independent wrap fixture.
- **Empty and invalid operands.** Select/Replace raise `SequenceError::EmptyNotAllowed`; Reverse
  leaves empty unchanged (ADR 0007). Non-Number index, Sequence where an Atom is required, Atom
  where a Sequence is required, and arity are covered by
  `select_diagnoses_empty_sequences_and_non_number_indices`,
  `replace_diagnoses_empty_sequences_non_number_indices_and_sequence_replacements`,
  `reverse_and_concatenate_diagnose_atoms_where_sequences_are_required`, and
  `structural_functions_diagnose_missing_operands`.
- **Parse and round-trip.** `define_functions!` rows `:<`, `:&`, `:?`, `:=`. Covered by
  `every_sequence_function_parses_and_round_trips_its_spelling` and
  `select_and_replace_parse_round_trips_execute_through_nested_operands`. `:&` is parsed as a nested
  Expression in that test rather than as a bare `TryFrom` spelling; Display still answers `:&`.
- **Bang type and encoding.** Reverse and Replace are driven by
  `structural_operations_preserve_bang_type_and_encoding`; Select by
  `select_declares_that_it_can_emit_bang_and_returns_a_bang_member` (and the Source Grid
  `a_select_bang_activates_an_aligned_terminal_root`). Concatenate copies members and so preserves
  Bang the same way Reverse does; nothing concatenates a Bang member as a fixture.
- **Generic over Atom type.** Signatures name `Sequence` / `Atom` / `AtomOrSequence`, not Number or
  Note. Reverse and Replace fixtures mix Number, Note, and Bang; Concatenate's happy path is Numbers
  only, plus a refused effect-Function member.

Source projection of these results is issue 06's, already resolved:
`live_a_declared_reverse_result_reaches_its_destination_cells` and the Concatenate / Replace
siblings in `orcvs/src/source/tick.rs`.
