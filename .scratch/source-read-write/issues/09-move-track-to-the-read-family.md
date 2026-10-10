# 09 — Move Track to the Read family

Status: resolved
Blocked by: 07 — Accept the Read, Write and Copy families

**What to build:**

Track moves from `@t index count` to `&t index count` (ADR 0070), because it reads. Behaviour is unchanged, including `index` counting pairs. This frees `@t` for Push (ticket 06).

## Acceptance criteria

- [x] `&t` is in the Function table with Track's declaration unchanged except for its spelling; `@t` no longer parses as Track.
- [x] Every existing Track test passes with only the spelling changed, including Turn-time ordering, discovered cycles and the reused-versus-fresh schedule comparison.
- [x] Source Paint and any console test that spells Track use `&t`.
- [x] The Function reference and every shipped Source File spell `&t`.
