# 02 — Broadcast Atomic Functions over Sequences

**What to build:** Apply compatible stateless Atomic Functions pervasively across Sequence operands
using ADR 0007's scalar and equal-length rules.

**Blocked by:** 01 — Add the Sequence language value; orcvs-language-migration/05; orcvs-language-migration/07.

**Status:** resolved

**Tags:** release/v1

- [x] Atom/Atom evaluates once and returns the ordinary result.
- [x] Atom/Sequence and Sequence/Atom repeat the scalar across every element.
- [x] Equal-length Sequences pair element-wise in order.
- [x] Unequal non-scalar lengths diagnose and return no partial Sequence.
- [x] Per-element type or evaluation failure diagnoses the complete operation.
- [x] Unary `.v` and `.^` conversions broadcast atom-wise, preserve order, and return no partial
      Sequence when any element fails conversion.
- [x] Equality remains ADR 0011's whole-value predicate: it produces one scalar Bang only when
      every broadcast pair is equal, otherwise no value; it never creates absent Sequence elements.
- [x] Increment and Interpolation are not accidentally broadcast.

## Comments

The broadcast seam is `Stack::broadcast` in `lang/src/stack.rs`. It pops the operands a Function
declares, decides one shape for the whole operation, and hands out the operands for each element.
`extract`, `apply`, `predicate`, and `convert` are four thin layers over that one decision rather
than four mechanisms: the arithmetic Functions and the numeric conversions differ only above it, in
the type layer, because ADR 0021 excludes `.v` and `.^` from the single `Token` their literal
signature declares and has them read a `NumericValue` at evaluation instead. A second broadcast for
the conversions would have been free to disagree with the first about lengths, about ordering, and
about what a partial failure leaves behind.

Broadcasting is declared per Function rather than assumed of every value Function. `define_functions!`
carries a third column, `Pervasive` or `Scalar`, and `Stack::broadcast` asks it before it widens
anything. `is_terminal` can be read off the `!` family prefix, and pervasion cannot: ADR 0012 makes
Increment `~+` and Interpolation `~>` scalar exceptions that will share a family prefix, a kind, and
a signature with Functions that do broadcast. An exception that arrived by omission would therefore
be silent, so the pervasion is declared beside every other property of a Function and
`every_function_declares_whether_it_extends_over_a_sequence` matches exhaustively over `Function`
with no wildcard: a Function added later has to be classified in the table and again in the test.

Equality is not written as a map. ADR 0011 makes it a whole-value predicate, so it goes through
`Stack::predicate`: it uses ordinary broadcasting to find its comparison pairs and answers one Bang
when every pair is equal and the Absence Marker when any pair is not. A map would need something to
write at a position where a pair disagreed, and the only Atom meaning nothing is the Absence Marker,
which `Sequence::new` refuses because it has no Source encoding. A comparison of no pairs is
vacuously all-equal, which is why an empty Sequence operand answers one Bang there where an
arithmetic Function answers the empty Sequence.

Narrowed, deferred, or only partly witnessed:

- "Increment and Interpolation are not accidentally broadcast" is closed by the mechanism rather
  than by those two Functions, which no issue has built yet. What this issue can hold is that they
  cannot become pervasive by omission when they arrive, and that is what the declared column and
  the exhaustive pervasion test hold. The Functions themselves belong to whichever issue adds them.
- The stated diagnostic order is arity, then shape, then per-element type, then per-element domain,
  then evaluation. Three of its four adjacencies have a witness on the pervasive path — arity before
  shape, shape before type, and type before evaluation. The type-before-domain and
  domain-before-evaluation pairs have no pervasive witness, because every operand domain narrower
  than its `Token` (`MidiChannel`, `Velocity`, `Length`) belongs to a Scalar Terminal Output
  Function; those two pairs are pinned on the scalar path by
  `every_arity_and_type_diagnostic_precedes_every_domain_diagnostic` instead. `checked` still runs
  every operand's type check before any element binds, so the ordering holds for the first pervasive
  Function that declares a narrowed domain rather than being retrofitted for it.
- Nothing here is reachable from Source text. No Source-parseable Function produces a Sequence
  operand until issue 03 adds the structural Functions and issue 05 the Range Functions, so
  broadcasting is exercised through `Stack` and `Context` directly, the way issue 01 exercised the
  value. Every Source-level test is unchanged.
- Review found the empty-Sequence shape got this wrong once, and it is worth recording. The type
  check first walked elements rather than operands, so a width of zero ran no check at all: at
  `.= C4 <empty>` and `.+ ** <empty>` the scalar operand was never examined and the operation
  answered a vacuous Bang or the empty Sequence instead of diagnosing, contradicting ADR 0011's
  "accepts only Number operands". A scalar operand belongs to the operation's type even where the
  shape makes no element for it to be repeated into, so `checked` now walks operands in signature
  order and, within a Sequence operand, its members in order. That also fixes which of two faulty
  operands answers, and the operand order is the right one because the diagnostic carries the
  offending Atom rather than its index: the operand the Source wrote first is the only ordering a
  reader can follow. The unary conversions never had the hole — with one operand, a width of zero
  means that operand is itself the empty Sequence — and now carry a `UnaryOperands` bound so the
  arity comes from the same table row as the roles.
- `Stack::try_pop`, its arity remapping, and the two `MaybeAtom` conversions that were its only
  callers are gone; `NumericValue` now converts from an `Atom`, which is what a broadcast element
  is. `ArgumentError::Expected` went with them, having no remaining constructor.
  `Stack::pop` stays as the scalar pop offered outside Function evaluation, and keeps raising
  `SequenceError::ExpectedAtom` under the same rule `broadcast` applies to a Scalar Function.
- The broadcast seam has no panic path. `extract` answers `ExpectedAtom` for a widened shape rather
  than asserting, because binding element 0 would read one member of a Sequence and discard the
  rest silently inside a Tick; `convert` takes `UnaryOperands` rather than a `Function` argument
  unrelated to its closure, so a two-operand Function is a type error at the call site instead of a
  run-time surprise; and `assemble` defaults an impossible scalar shape to the absence marker,
  which plans no write, rather than panicking under the Source write guard as ADR 0028 forbids.
