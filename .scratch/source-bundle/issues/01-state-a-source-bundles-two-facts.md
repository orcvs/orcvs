# 01 — State a Source Bundle's two facts and derive every bundle rule from them

Status: resolved
Blocked by: None (can start immediately)

**What to build:**

A Source Bundle states two facts about itself: whether it **clears its own Span** (Advance, Fire) and whether it **writes displaced** (Advance, Emit). Every module that asks about a bundle reads those facts instead of matching on Advance, Emit or Fire:

- **Portal sites:** a bundle writes its own Span if it clears it, then the displaced destination if it writes one, in that order.
- **Tick scheduling:**
  - the self-overlap exemption is "clears its own Span";
  - contacts are ordered after the producer only when the bundle clears its own Span;
  - an intrinsically active contact is exempt only when the bundle also writes displaced.
- **Execution:** a Fire still takes the Bang Function's Turn. A refused displaced write leaves `**` when the bundle clears its own Span and diagnoses otherwise.

No Grid plays differently, and no scheduling edge or ADR changes. See `../spec.md`.

## Acceptance criteria

- [x] `SourceBundle` answers both facts, each through an exhaustive match, so a new variant must state both.
- [x] The Portal module builds a Source-writing Function's writes from the two facts and no longer matches on the variants.
- [x] Tick scheduling's three bundle predicates are deleted, and the producer-edge rule reads the two facts directly, with comments stating each rule against its fact.
- [x] Execution chooses the refusal arm and the entered-Cells filter from `clears_own_span()` rather than comparing against `Advance`.
- [x] The per-Function Source-effect table test asserts both facts for every Source-writing Function.
- [x] Every existing Tick, placement, Bang Function and Function reference test passes unchanged.
- [x] `CONTEXT.md` has a Source Bundle entry that names Advance, Emit and Fire by the two facts and states that refusal cost follows from the first.
- [x] Gates pass: `cargo fmt --all -- --check`, clippy and nextest on `lang` and `orcvs` and their dependents, then the workspace runs and doctests, with `PROPTEST_CASES=32`.

## Comments

2026-10-11 — Implemented. `SourceBundle::clears_own_span` and `writes_displaced` state the facts. The Portal module builds a bundle's writes from them, and `producer_edges` reads three rules off them once per producer: the self-overlap exemption, whether contacts are ordered, and the mover's intrinsic exemption. Execution routes a bundle that writes nothing displaced to the Bang Function's Turn and reads the refusal arm off `clears_own_span`. No Tick test changed. `SourceEffect.spelling` still restates `writes_displaced`, held by the table test; folding the spelling into the variants is outside the spec, which keeps `SourceEffect`'s fields.
