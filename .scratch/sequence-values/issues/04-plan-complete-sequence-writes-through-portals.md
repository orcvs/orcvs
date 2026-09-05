# 04 — Plan complete Sequence writes through Portals

**What to build:** Route ordinary Atom and Sequence results through one Portal destination into
atomic Tick Plan Cell writes, following ADR 0009.

**Blocked by:** 01 — Add the Sequence language value; language-map/03.

**Status:** resolved

**Tags:** release/v1

- [x] Cell writes validate their whole destination before any Cell of them is emitted.
- [x] Out-of-Grid or non-fitting output diagnoses and plans no partial write.
- [x] Empty Sequence plans no writes.
- [x] Current encoding never clears a stale tail from an earlier result.
- [x] Planned Cells participate independently in ADR 0020 conflict ordering.
- [x] Generated content is ordinary Source on the next Tick.
- [x] Portal remains internal destination state, not a language value or persisted object.

## Comments

Concrete Source Read/Write addressing remains deferred; ordinary result Portals are implementable.

### Resolution, 2026-09-05

`Portal`, `PortalError`, and `SpanWrite` now live in `orcvs/src/source/portal.rs`.
`Portal::ordinary_result` resolves the Cell below a producer's anchor or answers `BelowSource`;
`Portal::admit` answers with a validated whole-destination `SpanWrite` or `CrossesRowEdge`, and is
that type's only constructor, so a write exists only because a Portal accepted its whole
destination. Resolution moved out of `tick` because ADR 0009 expects destination resolution to
change — an infinite canvas among the reasons — without touching evaluation, effect ordering, or
commit.

Much of the behaviour this issue names was already standing from issue 01: the below-root
complete-fit path, the empty-Sequence early return, per-Cell conflict resolution in `resolve`, and
`is_computation` deciding whether generated Cells compute again. What this issue adds is the name,
the seam, and the evidence. `tick::emit_result` is the new seam — "an Interpretation plus an anchor
becomes effects" — and it is the only way the Sequence half of the result path can be reached
before issues 02 and 03 add the Functions that spell one. No test-only Function was added and no
Function spelling was prejudged.

The multi-Portal effect bundle ADR 0009 also describes is deliberately not built: ADR 0005 defers
the Source-addressing Functions that would resolve one, and a bundle API with no caller would be
shaped by guesswork. Because every Portal answers with a whole write or a refusal, a bundle is
those answers collected with `?` — it needs no validation pass of its own.

The last acceptance line is mostly an argument the compiler makes — `Portal` is `pub(super)` inside
`orcvs`, absent from `lang`, and derives no `Serialize`. `lang/src/portal.rs` held a dead
`Portal { atom, x, y }` that contradicted it outright, so this branch also closes
`lang-foundations/01`.

A finding rather than a defect: three adjacent Numbers written as a Sequence result parse on the
next Tick as one Expression whose head is an unknown Function, so a bare Sequence result normally
sits under a syntax diagnostic until something heads it. That is ordinary Source parsing — exactly
what typing the same characters gives — and ADR 0020 permits it explicitly, but issues 02 and 03
should keep it in view.

A second one, on the same acceptance line and worth more attention. `is_computation` is not a
guarantee that a generated Sequence sits still. It answers true for any Expression containing an
`Atom::Function`, and `Sequence::check_member` admits `Atom::Function(_)` and `Atom::Bang` — it
refuses only `Atom::Activation` and `Atom::Empty`, since ADR 0029's narrower "only Functions that
answer a value" rule is not implemented yet. So a Sequence result whose members include a Function
spelling encodes a live computation into the Source: `[Function(Add), Number(0x01), Number(0x02)]`
writes `+0102` into the row below, which the next Tick evaluates and writes a row further down
again, marching down the Grid. A member spelling `**` likewise becomes a Source-resident Bang.

That is the acceptance line satisfied rather than violated — it is precisely ADR 0007's "no
privileged literal-Sequence interpretation", and it is what typing the same characters gives — but
the inertness is a property of the Atoms a result happens to carry, not of the result path. Issues
02 and 03 own the question of which Atoms their Functions may answer with; the tests here use
all-Number encodings and pin nothing either way.


### Deepen ordinary-result delivery, 2026-09-05

Confirmed in the architecture review: delivery returns one optional Effect, while its caller
retains emission order, conflict resolution, and commit. `tick::result_effect` replaces
`emit_result` and is internal to Source. It owns absence/empty handling, Atom and Sequence
encoding, Portal admission, Play Commands, and diagnostic conversion without mutating an Effect
list. No evaluator adapter or external interface was introduced.

Source tests now supply actual Interpretations through that production path. The old
`plan_through_portal` helper and its string-based assembly are gone. Stale-tail, generated-versus-typed,
and printable-content round-trip tests now cover result encoding as well as commit. The empty
Sequence case reaches commit from a bottom-row root, demonstrating that no destination is needed.
Its narrower duplicate in the Tick tests was removed.

A generated Sequence containing Add and Numbers 01 and 02 writes the canonical `.+0102`; only the
next Tick computes and writes `03`, with the same Source outcome as typing those Cells. This pins
the Function-member behavior the earlier comment discussed (whose `+0102` example omitted the
numeric family prefix). The inert Number-only test and `is_computation` comment now state that
narrower claim explicitly.


Verification for the delivery follow-up:

- `cargo test --package orcvs --lib --locked source::model::test::a_generated_function_sequence_computes_on_the_next_tick` — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo check --package orcvs --all-targets --locked` — passed.
- `mise run check_pull_request` — passed: workspace Clippy with warnings denied, 364 tests,
  and doctests (covers the affected crate's scoped Clippy/test/doc gate).
- `mise run test_persistence` — passed: 384 tests, Clippy, doctests, and rustdoc with warnings denied.
- `cargo check --package orcvs --lib --target wasm32-unknown-unknown --features persistence --locked` — passed.
- `git diff --check` — passed; complete Source model and Tick diff reviewed.

Not repeated for this internal refactor: full `mise run check` and browser execution, which stopped
in the preceding Cell-content verification on the existing workflow-version contract and
OutputCommand/PlayCommand comparison respectively. No additional unsafe change was made, so the
preceding Miri run was not repeated. No dependencies, features, public interface, concurrency
behavior, or performance claims changed in this follow-up.
