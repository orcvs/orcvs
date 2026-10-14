# 04 — Decide whether evaluator dispatch is derived

**What to decide:** ADR 0028 requires evaluation dispatch to be derived from the single declaration rather than restated beside it. `lang-foundations/02` deliberately accepted a hand-written exhaustive match instead. Decide which stands, then record it.

**Blocked by:** 01 — Correct the evaluation machine decision.

**Status:** resolved

**Sources of truth:** ADR 0028's third paragraph states the requirement and explicitly leaves the declaration's form open; `lang-foundations/02` records the accepted alternative.

- [x] The decision is recorded, either as an amendment to ADR 0028 or as this ticket's answer. *(Dispatch stays a hand-written match, recorded in ADR 0028's third paragraph.)*
- [x] If dispatch stays a hand-written match, ADR 0028 no longer lists it among the things derived from the declaration. *(Met: ADR 0028's paragraph at line 9 says the dispatch is not derived and states what the compiler does and does not check about it.)*
- [ ] If dispatch becomes derived, a ticket exists for the change and states the form chosen. *(Not applicable: dispatch is not derived.)*
- [x] Either way, adding a Function still cannot leave dispatch silently incomplete. *(Holds today: the match in `execute_function` has no wildcard arm, `lang/src/interpreter.rs:113-176`.)*

## Comments

`execute_function` in `lang/src/interpreter.rs` matches every `Function` variant explicitly and routes each to its implementation. `lang-foundations/02` closed with "Evaluator dispatch names every variant explicitly", accepted as satisfying "signature lookup and evaluator dispatch are exhaustive and contain no wildcard fallback for a real Function". That is genuinely safe today: the match is exhaustive, so a new variant fails to compile until it is dispatched. What it is not is derived, and ADR 0028 asks for derived.

The gap between the two is small and may be worth nothing. An exhaustive match already gives the property the ADR wants — a change to the declaration cannot be invisible to dispatch — by a different mechanism than derivation. The cost of deriving it is a macro that must express each implementation's call shape, and the shapes are not uniform: `math::add` takes `&mut Context` and returns `Result<Value, Error>`, while the five Terminal Output Functions (Control Change, Monophonic Play, Pitch Bend, Raw Play and Timed Play) return early with `Interpretation::Play` and never push. A derivation that flattened those would have to encode the value-or-effect distinction, which is issue 05's subject.

This is a decision, not a task, which is why it is `needs-triage` rather than `ready-for-agent`. The ADR says the declaration's form "may be types, or data, or a mixture, and this decision constrains only that there is one of it and that everything else is derived". Whoever triages this should be willing to conclude that dispatch is an acceptable exception and amend the ADR to say so, rather than treating the ADR's list as settled.

### Audit at cad296df — 2026-09-29

- `Interpreter::execute` was deleted in 26167b95. Dispatch now lives in `execute_function`
  (`lang/src/interpreter.rs:93-186`), and the Comment above names it.
- No `Function::Play` exists. Five Terminal Output Functions return `Interpretation::Play` early,
  and the lock and Source-effect Functions return before dispatch and reach `unreachable!` arms.
  The Comment now says so.
- ADR 0028 already carries the "not derived" wording (`01680928`), and the match is exhaustive with
  no wildcard, so criteria 2 and 4 hold. Only the decision itself (criterion 1, and criterion 3 if
  dispatch becomes derived) remains. The status stays `needs-triage`.

### Decision — 2026-10-14

Dispatch stays a hand-written exhaustive match, and ADR 0028's third paragraph records it as not
derived, by choice.

The reason is cost against benefit, not that a body cannot be declared. Deriving dispatch would put a
body column in every row of `define_functions!`. The directional Copies fit that shape with a
one-line wrapper each, because `copy` reads its Function only to label a `PartialInput` error. The
benefit derivation would buy is that the declaration cannot change invisibly to dispatch, and the
exhaustive match already gives that: there is no wildcard arm, so a new Function fails to compile
until it is dispatched.

What neither the match nor the compiler checks is that each arm agrees with its Function's declared
kind. `Function::RawPlay => math::add(&mut ctx)?` compiles, and so does routing `Add` to a Play body.
A value Function added to the `unreachable!` arm compiles and panics when evaluated:
`locking_and_source_writing_functions_are_interpreted_from_their_kind` visits only Functions whose
kind already returns before dispatch, so it cannot see one. Issue 07 records that gap and the two
shapes that would close it: a sweep over `Function::ALL` comparing each answer's interpretation with
the declared kind, or the generated per-Function evaluation trait issue 06 asked this ticket to
weigh, which would make a mis-route a type error. That trait is a form of derivation. It is left to
07 because it is worth its cost only if a test sweep cannot close the gap.

Hand-written per-Function matches remain elsewhere, outside this decision. `Function::output_portal`,
`dynamic_output_portal` and `input_portal` in `lang/src/atom.rs` default through wildcard arms, so a
new Function silently receives default Portals.
