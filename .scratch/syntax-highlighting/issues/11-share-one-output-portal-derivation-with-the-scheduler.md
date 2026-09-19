# 11 — Share one Output Portal derivation with the scheduler

**What to build:** One derivation of the Cells each root's Output Portal covers, read by both the Output Portal highlight and Tick scheduling's reservations, replacing `10`'s two derivations and the agreement test between them.

**Blocked by:** 10 — Derive Output Portal destinations and prove agreement with the scheduler.

**Status:** needs-triage

- [ ] Tick scheduling reads its scalar and Sequence-capable reservations from the same derivation the Language Map carries, or the reason it cannot is recorded.
- [ ] `10`'s agreement test is removed or reduced to the exclusions `05` names.
- [ ] No schedule changes: every existing Tick test passes unchanged.

## Comments

Raised 2026-09-19 while resolving `05`. `derive_reservations` (`orcvs/src/source/tick.rs`) works on tick planning's `Computation` tree and widens bottom-up; sharing means the Language Map's derivation owns that propagation. Larger than `10`, so kept apart.
