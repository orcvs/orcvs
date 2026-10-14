# 04 — Decide whether evaluator dispatch is derived

**What to decide:** ADR 0028 requires evaluation dispatch to be derived from the single declaration rather than restated beside it. `lang-foundations/02` deliberately accepted a hand-written exhaustive match instead. Decide which stands, then record it.

**Blocked by:** 01 — Correct the evaluation machine decision.

**Status:** resolved

**Sources of truth:** ADR 0028's third paragraph states the requirement and explicitly leaves the declaration's form open; `lang-foundations/02` records the accepted alternative.

- [x] The decision is recorded, either as an amendment to ADR 0028 or as this ticket's answer. *(Dispatch stays a hand-written match, recorded as ADR 0028's one accepted exception.)*
- [x] If dispatch stays a hand-written match, ADR 0028 no longer lists it among the things derived from the declaration. *(Met: ADR 0028's paragraph at line 9 names dispatch as "the one accepted exception" and states the guarantees that stand in for derivation.)*
- [x] If dispatch becomes derived, a ticket exists for the change and states the form chosen. *(Not applicable: dispatch is not derived.)*
- [x] Either way, adding a Function still cannot leave dispatch silently incomplete. *(Holds today: the match in `execute_function` has no wildcard arm, `lang/src/interpreter.rs:113-177`.)*

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

Dispatch stays a hand-written exhaustive match, and ADR 0028 now records it as the one accepted
exception to derivation, stating the guarantees that stand in for it.

Issue 05 resolved the obstacle this ticket named: the table's kind column now declares value or
effect. That does not make derivation worth its cost. The match already cannot be silently
incomplete, because it has no wildcard arm and the arms' return types (`Atom` for a value body,
`PlayCommand` for a Terminal Output body) tie each Function to a body of its own kind. Deriving it
would put a body column in `define_functions!`, and the directional Copies would need a new body
signature, because they take the Function as an argument.

The one residual gap is the `unreachable!` arm for the lock and Source-write Functions. It is
covered today by `locking_and_source_writing_functions_are_interpreted_from_their_kind`. Splitting
the Function enum by kind would make that arm unrepresentable. It was considered and not taken; open
a ticket for it if a new effect kind is added.
