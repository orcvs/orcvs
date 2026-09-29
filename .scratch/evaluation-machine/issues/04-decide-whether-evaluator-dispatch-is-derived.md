# 04 — Decide whether evaluator dispatch is derived

**What to decide:** ADR 0028 requires evaluation dispatch to be derived from the single declaration rather than restated beside it. `lang-foundations/02` deliberately accepted a hand-written exhaustive match instead. Decide which stands, then record it.

**Blocked by:** 01 — Correct the evaluation machine decision.

**Status:** needs-triage

**Sources of truth:** ADR 0028's third paragraph states the requirement and explicitly leaves the declaration's form open; `lang-foundations/02` records the accepted alternative.

- [ ] The decision is recorded, either as an amendment to ADR 0028 or as this ticket's answer.
- [x] If dispatch stays a hand-written match, ADR 0028 no longer lists it among the things derived from the declaration. *(Met: ADR 0028's paragraph at line 7 says "The evaluation dispatch is not: it is still a hand-written match" and leaves the question open.)*
- [ ] If dispatch becomes derived, a ticket exists for the change and states the form chosen.
- [x] Either way, adding a Function still cannot leave dispatch silently incomplete. *(Holds today: the match in `execute_function` has no wildcard arm, `lang/src/interpreter.rs:125-181`. Re-check it if dispatch becomes derived.)*

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
