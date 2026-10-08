# 01 — Name Input Portals by when their position is known

**What to build:** A Function's Input Portal is declared as static or dynamic, naming the one distinction scheduling and the Turn act on: a static Input Portal's position is known when the Source is parsed, so its writers are ordered before the Tick; a dynamic Input Portal's position is known only at the Function's Turn, from its operands, so its writers are found then. Jump, Increment and Interpolation declare static Input Portals; Track declares the one dynamic Input Portal. A performer sees no change: the same Source plays, paints and diagnoses as before.

**Blocked by:** None — can start immediately.

**Status:** resolved

## Acceptance criteria

- [x] The Input Portal declaration's two kinds are named static and dynamic, replacing the anchored and after-operands names. The static kind carries its offset from the anchor; the dynamic kind carries nothing.
- [x] The declaration's documentation states that it is the Portal's position that is static or dynamic. The Cells any Input Portal reads are read from working Source at the Turn, so a static Input Portal is not one whose contents are fixed.
- [x] Scheduling, the Turn and every other reader of the declaration branch on static versus dynamic with unchanged behaviour. No other vocabulary in those paths still names the after-operands framing.
- [x] CONTEXT.md's Input Portal entry names Jump, Increment and Interpolation's Input Portals static and Track's dynamic; the Track Function entry uses the same term where it says its Input Portal is known only at its Turn.
- [x] ADR 0067 records the static/dynamic vocabulary as an amendment: a Jump's Input Portal is static and its dependency known before the Tick; Track's is dynamic and its dependency found at its Turn.
- [x] Existing Track, Jump, Increment, Interpolation, schedule-reuse and cycle tests pass unchanged; no test is rewritten to accommodate the rename beyond the renamed identifiers.
- [x] Review the complete diff and run the repository's required verification for `lang` and its dependents, with the local proptest case count set to 32. Leave the broader checks designated for CI there.

## Scope and constraints

This is a vocabulary change under ADR 0067, classified as active language design rather than public-API breakage. It does not move the selection rule, change any public function's behaviour, or alter Track's semantics. Resolving a dynamic Input Portal's position in `lang` is ticket 02.

## Comments

**2026-10-08 — implemented in `9380786d`.** `lang::InputPortal` is `Static(PortalCoords)` / `Dynamic`; CONTEXT.md's Input Portal and Track entries and an amendment section in ADR 0067 carry the vocabulary.
