# Audit of “Orcvs lang Findings”

## Purpose

This document audits the Claude artifact `e4283a16-99e8-4deb-90ce-4d3677b58434`, titled
“Orcvs lang Findings”. The artifact reviewed the `lang` crate at commit `c889af5` and reported
twelve design findings. This audit checks those findings against the cited source, the repository's
design documents, callers, manifests, native tests, WASM build, and dependency policy.

The artifact is useful and mostly evidence-based, but some conclusions need narrower wording. At
the reviewed commit, eight findings are substantially confirmed and four require correction or
additional design or performance evidence.

## Overall assessment

| Finding | Assessment | Recommendation |
| --- | --- | --- |
| F-01 | Confirmed at `c889af5`; resolved on the audited current branch | Keep the corrected issue language and treat this item as closed |
| F-02 | Core synchronization risk confirmed; compiler-silence claim overstated | Centralize spelling and signatures, make behavior matches exhaustive, and add round-trip tests |
| F-03 | Confirmed | Replace inverse MIDI tables with checked conversion logic and exhaustive boundary/round-trip tests |
| F-04 | Confirmed semantic boundary defect | Introduce typed operand extraction and enforce Raw Play domains inside interpretation |
| F-05 | Attribute count confirmed; performance effects unmeasured | Remove unjustified attributes as maintenance work or benchmark before making performance claims |
| F-06 | Redundant reconstruction confirmed; “three copies” is incorrect | Remove the two redundant `collect` operations without claiming a measured speedup |
| F-07 | Confirmed | Change `Parser::from` to accept `&str` and simplify callers |
| F-08 | Invariant risk confirmed; proposed tuple representation not yet justified | Confirm the domain invariant before choosing a paired representation |
| F-09 | Confirmed dead placeholder | Remove `Portal` until Tick planning requires a concrete internal representation |
| F-10 | Confirmed dependency-placement problem | Gate the tracing helper for tests and move `tracing-subscriber` to dev-dependencies |
| F-11 | Confirmed unused conversions | Remove them and reintroduce typed conversions only when a Function needs them |
| F-12 | Confirmed architectural gap | Implement the lexical Language Unit pass as part of language-map issue 01 |

## Detailed recommendations

### F-01 — Keep the Language Map issue aligned with the language design

At `c889af5`, the issue incorrectly described Comments as beginning with `#` and required lexical
Number/Note identity. ADR 0023 requires `##`, while ADR 0021 and `CONTEXT.md` make Operand Literal
typing contextual to a consuming Function's signature.

The current issue text has already been corrected. Keep this item closed and preserve these rules:

- A Comment begins with `##`; one `#` is incomplete or invalid Source.
- Lexing establishes a complete two-Cell Operand Literal, not an intrinsic Number or Note.
- Parsing against the consuming Function's signature assigns the Atom type.

### F-02 — Make Function definitions compiler-checked

The underlying risk is real: a Function's spelling, parsing, operand signature, and evaluation are
distributed across several files. However, the artifact's claim that adding a variant immediately
“compiles with no error” is inaccurate. `Display for Function` is exhaustive, so the compiler first
requires a display arm. After that arm is added, wildcard branches in signature selection and
interpretation can silently supply empty behavior.

Recommended change:

- Define one canonical `Function::spelling()` method.
- Define one exhaustive `Function::signature()` method.
- Make `Display` delegate to `spelling()`.
- Make `TryFrom<&str>` use the same canonical definitions rather than a separate spelling table.
- Add `Function::ALL` for exhaustive tests and iteration.
- Remove wildcard branches that turn an unimplemented Function into zero operands or
  `Function::Empty`.
- Keep evaluation dispatch exhaustive, or make unsupported behavior return an explicit error.
- Test that every Function spelling parses back to the same Function.
- Test that spellings are unique.
- Test every declared Function has an intentional signature and evaluation classification.

Do this before adding the larger planned Function set.

### F-03 — Replace the MIDI tables carefully

The two 107-entry tables are inverse hand-written mappings from A0 through G9. They omit MIDI
values 0 through 20 even though the target language permits Notes from C/ through G9.

Recommended change:

- Replace the tables with checked conversion logic based on MIDI octave and semitone arithmetic.
- Keep Source spelling canonical, including lowercase pitch letters for sharps and `/` for the
  octave below zero.
- Reject malformed spellings rather than normalizing them silently.
- Test every MIDI byte from 0 through 127 in both directions.
- Assert `number -> spelling -> number` for the full Note domain.
- Assert `spelling -> number -> spelling` for every canonical spelling.
- Add explicit boundary tests for C/, G9, values 0 and 127, and invalid values above 127.

The formula removes duplication, but the tests—not the formula alone—should establish the language
contract.

### F-04 — Enforce Raw Play operand types at the interpreter boundary

The parser declares Raw Play operands as channel Number, velocity Number, and Note. The interpreter
currently extracts all three through `TryFrom<MaybeAtom> for u8`, which accepts Number, Note, and
Char. Because `Interpreter::execute` is public, parser validation cannot be treated as the only
boundary.

Recommended change:

- Use `NumberValue` for channel and velocity extraction.
- Introduce a `NoteValue` newtype that accepts only `Atom::Note`.
- Validate channel as `00` through `0F`.
- Validate velocity as `00` through `7F`.
- Validate Note as a typed Note in the MIDI range.
- Return typed errors for wrong Atom type and out-of-domain values.
- Test direct `Interpreter::execute` calls with swapped Number/Note operands, Char operands, and
  channel/velocity boundary values.

This should be coordinated with centralized typed operand extraction rather than adding more loose
`u8` conversions.

### F-05 — Separate maintainability cleanup from performance claims

The crate contains 37 `#[inline(always)]` attributes and no benchmark demonstrating their value.
`char_to_num` also creates a `String` to parse one hexadecimal character. Those observations are
correct. The artifact did not measure compile time, code size, or runtime, so it cannot conclude
that the attributes increase costs or that removing them improves performance.

Recommended change:

- Remove `#[inline(always)]` where it has no documented, measured need.
- Prefer ordinary `#[inline]` only at a justified cross-crate abstraction boundary.
- Let the optimizer decide for small internal functions by default.
- Replace the `char_to_num` string conversion with direct ASCII hexadecimal decoding.
- Do not describe these changes as performance improvements without a reproducible benchmark or
  profile.
- If parser or Tick performance becomes important, benchmark the complete representative path
  rather than isolated helpers.

Treat this primarily as signal-to-noise and maintainability work.

### F-06 — Remove two redundant reconstructions, not “three copies”

`mem::take` transfers ownership of the `ArrayVec`; it does not copy its contents. The current path
then reconstructs an `ArrayVec` once in `Expression::take_atoms` and again in
`Parser::try_parse`. The artifact's three-copy count is therefore incorrect.

Recommended change:

- Return the `ArrayVec` obtained from `mem::take` directly from `take_atoms`.
- Return that `Atoms` value directly from `Parser::try_parse`.
- Apply the same reasoning to token handoff where the desired return type permits it.
- Characterize this as removal of redundant iteration and reconstruction.
- Do not assign a performance severity until profiling shows this path is material.

### F-07 — Remove the unnecessary mutable parser input

`Parser::from` accepts `&mut str` but stores and reads it as `&str`. This needlessly requires callers
to own mutable strings and causes avoidable cloning in the Language Map.

Recommended change:

- Change `Parser::from` to accept `&str`.
- Pass Source slices directly where lifetimes allow.
- Remove caller-side mutable bindings and clones that exist only to satisfy this signature.
- Preserve tests for strict parsing, diagnostic parsing, nested Functions, and incomplete Source.

### F-08 — Encode the Expression invariant only after confirming it

`Expression` currently keeps parallel Token and Atom arrays, and `add` relies on them staying in
step. This is a real invariant that the type does not express. A paired array is attractive, but
the audit does not establish that every future Language Unit must always correspond to exactly one
runtime Atom.

Recommended process:

1. Confirm whether incomplete operands, diagnostics, Comments, Bangs, Activation Characters, and
   future structural syntax all have a one-to-one Token/Atom relationship.
2. If the invariant is universal, store a named pair type or `ArrayVec<(Token, Atom), EXP_LEN>`.
3. If syntax can exist without a runtime Atom, model that distinction explicitly rather than
   forcing placeholder Atoms into a tuple.
4. Preserve the separate consumer needs: tokens drive footprints and Glyphs, while atoms drive
   interpretation.
5. Add invariant tests around partial/incomplete parsing and expression-capacity errors.

Do not make the representation change solely because parallel arrays look suspicious; choose it
from the language model.

### F-09 — Remove the unused Portal value

The exported `Portal` struct is unused and conflicts with ADR 0009's description of Portal
resolution as Tick-planning behavior rather than a public language value.

Recommended change:

- Delete `lang/src/portal.rs` and its public re-export.
- Review downstream compilation to confirm no external workspace consumer relies on it.
- Introduce a new internal planning representation only when Sequence/Portal behavior is
  implemented and its invariants are known.

Because this changes a public export, perform a human API review even though the application has no
public compatibility contract.

### F-10 — Keep test tracing out of the shipped library graph

The global tracing subscriber helper is used only by tests, but `tracing-subscriber` is a normal
`lang` dependency. Dependency inspection confirms that it is present in the normal `lang`
dependency graph, including WASM compilation.

Recommended change:

- Put the tracing initialization helper and its `Once` static behind `#[cfg(test)]`.
- Move `tracing-subscriber` from `[dependencies]` to `[dev-dependencies]`.
- Consider a shared test helper that tolerates an already-installed global subscriber rather than
  panicking.
- Run the native crate gate, `mise run check_wasm`, and `mise run audit_deps` after the manifest
  change.

### F-11 — Remove unused broad conversions

The `MaybeAtom -> String` and `MaybeAtom -> Function` conversions have no Rust callers. Broad
conversion implementations also make the operand contract harder to read than Function-specific
typed extraction.

Recommended change:

- Delete both unused implementations.
- Add future extraction through semantic newtypes such as `NumberValue` and `NoteValue`.
- Prefer errors that name the expected operand domain.
- Reintroduce a conversion only when a concrete Function requires it.

### F-12 — Implement a lexical Language Unit pass at the Language Map boundary

ADR 0018 requires a row-local, non-overlapping Language Unit partition. The current parser handles
Bang and Activation spellings as special cases and has a `check` mode that changes failure
behavior. `LanguageMap` explicitly records that Language Unit queries await lexical prerequisites.

Recommended change:

- Implement the lexical partition as part of language-map issue 01, after preserving the corrected
  Comment and contextual Operand Literal rules.
- Keep row boundaries explicit and prevent any unit footprint from crossing them.
- Recognize complete Functions, Operand Literals, Bangs, and Activation Characters.
- Exclude `##` through row end as Comment text.
- Diagnose invalid or incomplete Source while continuing at the defined recovery Cell.
- Keep lexical recognition separate from contextual Number/Note interpretation.
- Replace the parser's boolean `check` mode with explicit strict and diagnostic APIs or result
  types.
- Add boundary or property tests for every row partition, non-overlap, recovery, and rectangular
  Grid behavior.

Do not perform this as an isolated parser refactor before the Language Map work establishes the
required interface.

## Recommended order of work

1. Keep F-01 closed and use its corrected contract as the source for language-map work.
2. Remove the unused Portal and dead conversions (F-09, F-11).
3. Centralize Function definitions and add exhaustive compiler-checked tests (corrected F-02).
4. Enforce typed Raw Play operands and centralize typed extraction (F-04).
5. Replace MIDI tables with checked conversion logic and exhaustive tests (F-03).
6. Simplify parser borrowing and Atom handoff (F-07 and corrected F-06).
7. Move test tracing out of normal dependencies (F-10).
8. Remove unjustified inline directives and direct string allocation as maintenance work (F-05).
9. Decide the Expression representation from the confirmed Language Unit/Atom relationship (F-08).
10. Implement the Language Unit lexical pass with boundary/property tests (F-12).

F-02 and F-04 should precede addition of the planned Function set. F-08 should be decided together
with F-12 rather than as an isolated container refactor.

## Verification evidence

The following commands were run against the audited current worktree:

```text
cargo fmt --all -- --check — passed
cargo check --package lang --all-targets --locked — passed
cargo clippy --package lang --all-targets --locked -- -D warnings — passed
cargo nextest run --package lang --locked — passed, 30/30 tests
cargo test --package lang --doc --locked — passed, 0 doctests
mise run check_wasm — passed
mise run audit_deps — passed; advisories, bans, licenses, and sources OK
```

`mise run check` was not run because this was a read-only review scoped to `lang`; the complete
scoped gate plus the WASM and dependency risk gates passed.

## Risk assessment

- **Public API:** Removing `Portal` and changing parser construction affect exported interfaces;
  perform human caller review. The language and application have no public compatibility contract.
- **Unsafe:** No unsafe code or invariants are involved in these recommendations.
- **Dependencies:** Moving `tracing-subscriber` changes the manifest and requires the dependency
  audit gate.
- **Features and platforms:** `lang` declares no features. Native and WASM targets must remain
  supported.
- **Performance:** F-05 and F-06 have no benchmark evidence. Avoid performance claims until a
  representative benchmark or profile exists.
- **Parser/protocol boundary:** F-03, F-04, F-08, and F-12 require exhaustive boundary or property
  tests because they define Source interpretation and generated-Source round trips.
