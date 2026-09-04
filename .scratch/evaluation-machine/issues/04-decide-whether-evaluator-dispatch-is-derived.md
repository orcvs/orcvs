# 04 — Decide whether evaluator dispatch is derived

**What to decide:** ADR 0028 requires evaluation dispatch to be derived from the single declaration rather than restated beside it. `lang-foundations/02` deliberately accepted a hand-written exhaustive match instead. Decide which stands, then record it.

**Blocked by:** 01 — Correct the evaluation machine decision.

**Status:** needs-triage

**Sources of truth:** ADR 0028's third paragraph states the requirement and explicitly leaves the declaration's form open; `lang-foundations/02` records the accepted alternative.

- [ ] The decision is recorded, either as an amendment to ADR 0028 or as this ticket's answer.
- [ ] If dispatch stays a hand-written match, ADR 0028 no longer lists it among the things derived from the declaration.
- [ ] If dispatch becomes derived, a ticket exists for the change and states the form chosen.
- [ ] Either way, adding a Function still cannot leave dispatch silently incomplete.

## Comments

`Interpreter::execute` in `lang/src/interpreter.rs` matches every `Function` variant explicitly and routes each to its implementation. `lang-foundations/02` closed with "Evaluator dispatch names every variant explicitly", accepted as satisfying "signature lookup and evaluator dispatch are exhaustive and contain no wildcard fallback for a real Function". That is genuinely safe today: the match is exhaustive, so a new variant fails to compile until it is dispatched. What it is not is derived, and ADR 0028 asks for derived.

The gap between the two is small and may be worth nothing. An exhaustive match already gives the property the ADR wants — a change to the declaration cannot be invisible to dispatch — by a different mechanism than derivation. The cost of deriving it is a macro that must express each implementation's call shape, and the shapes are not uniform: `math::add` takes `&mut Context` and returns `Result<Value, Error>`, while `Function::Play` returns early with `Interpretation::Play` and never pushes. A derivation that flattened those would have to encode the value-or-effect distinction, which is issue 05's subject.

This is a decision, not a task, which is why it is `needs-triage` rather than `ready-for-agent`. The ADR says the declaration's form "may be types, or data, or a mixture, and this decision constrains only that there is one of it and that everything else is derived". Whoever triages this should be willing to conclude that dispatch is an acceptable exception and amend the ADR to say so, rather than treating the ADR's list as settled.
