# 05 — Decide how a Result is known before a Tick

**What to build:** A recorded decision on how a Source revision names the Cells that hold a Function's Result, so the paint can colour them, and an amendment to the spec, which says the role already exists.

**Blocked by:** None — can start immediately.

**Status:** ready-for-human

- [ ] The decision states which Cells are Result Cells for a root: at least the Output Portal site one row south, and how far a Function that can answer a Sequence reaches (for example, to the end of that row, as a Reservation does).
- [ ] It states where the fact lives — the Language Map, the Render Frame's derivation, or neither — and why that is not a third classifier.
- [ ] It states what the written Cells parse as. Today a `07` left south of `.+0304` re-parses as two unknown one-Cell Functions with diagnostics, so it would paint as Function and Diagnostic, not Result. Whether that parse changes is part of the decision, because it is a language change, not a paint change.
- [ ] It states whether a Result is recognised by position alone (a stale write under an edited Function) or only while its root still answers that shape.
- [ ] The spec's claim that every role already exists is corrected.
- [ ] If the decision changes parsing or the Language Map's vocabulary, it is recorded in `CONTEXT.md` and an ADR.

## Comments

Research found no Result notion before Tick planning: Reservations and Portal destinations are computed while a Tick is planned and are not reachable from the Source revision or Render Frame. A Function's declared Output Portal exists on the Function itself but nothing at parse or render time reads it. A `**` a Function wrote is indistinguishable from a typed one.
