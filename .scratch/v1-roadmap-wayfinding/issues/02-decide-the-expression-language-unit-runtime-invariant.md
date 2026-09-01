# Decide the Expression, Language Unit, and runtime value invariant

Type: grilling

Blocked by: None — can start immediately.

Status: resolved

## Question

What invariant relates Language Units, Expression syntax expectations, and runtime values across
valid Source, incomplete Live Edits, invalid Source, diagnostics, Comments, Bangs, Activation
Characters, and future structural syntax? Decide whether every Expression entry necessarily has
one runtime Atom before choosing paired storage, and state the representation constraints that
replace the premature tuple assumption in the current foundation ticket.

## Answer

The one-to-one invariant is deliberately narrower than either Language Unit recognition or Source
analysis: every **successfully parsed, evaluable Expression entry** pairs exactly one syntax
expectation with exactly one real runtime value. This closed category currently covers a Function
and each context-typed Operand Literal consumed by its fixed signature. It does not imply that
every Language Unit, Language Map record, or piece of Source syntax has a runtime Atom.

Incomplete and invalid Live Edits remain character Source and produce explicit analysis or
diagnostic records. They do not form valid Language Units and must not acquire `Atom::Empty`,
`Function::Empty`, or any other placeholder runtime value. Comments are excluded from Language
Units and Expressions and likewise have no runtime value. Diagnostics are derived outcomes tied to
Positions and Footprints from the same Source revision, not language values.

Bang is both a Language Unit and a runtime Atom, including when preserved by compatible Sequence
structure, but a standalone Bang is not thereby an Expression entry. Activation Characters retain
their accepted observable behavior: direction persists in Source, each advances once per Tick,
and collision produces Bang. Their implementation classification remains open pending a focused
prototype comparing a distinct spatial Language Unit with a self-reproducing Function/value model.
That prototype may reuse an internal source-encodable representation, but must not accidentally
grant Expression-operand or Sequence behavior. This open representation choice does not block the
valid Expression-entry invariant.

Future structural Expression syntax that has no runtime value must be represented explicitly and
must never receive a fake value. Whether it remains outside evaluable entries or later introduces
an explicit structural variant is deferred until such syntax exists.

These decisions replace the premature universal tuple assumption with representation constraints:

- Pair syntax expectation and runtime value atomically only for successfully parsed evaluable
  entries; bounded insertion cannot commit half an entry.
- Give an Expression one authoritative entry count; do not mask divergence with parallel lengths.
- Keep lexical Language Units, Expression membership, permissive analysis, diagnostics, and runtime
  values distinct behind the Language Map's deep interface.
- Strict parsing returns only complete evaluable entries or an error. Permissive Live Edit analysis
  preserves complete recognized units plus explicit incomplete and invalid records.
- Positions, Footprints, entries, and diagnostics come from one immutable Language Map revision;
  Source remains the only stored program state and Grid the only coordinate system.
- The paired entry is an internal representation, not a second mutable Source model or a new public
  seam. Callers obtain syntax, values, roots, Footprints, and diagnostics through the Language Map.

This requires correcting `lang-foundations/07`: its paired storage applies to valid evaluable
entries, while its valid, incomplete, and invalid outcomes must no longer imply that every outcome
has a pair. The current parallel `tokens`/`atoms`, maximum-length fallback, and sentinel recovery
are migration inputs rather than language invariants.
