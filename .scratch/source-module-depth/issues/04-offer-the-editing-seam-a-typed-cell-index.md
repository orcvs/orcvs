# 04 — Offer the editing seam a typed Cell index

**What to build:** A caller editing the Source can address a Cell with an index the Grid minted,
rather than a bare number the Source re-checks. Both forms work; nothing that compiles today stops
compiling. This is the expand half of a two-step change.

**Blocked by:** 03 — Tidy the Language Map's interfaces.

**Status:** resolved

**Tags:** release/v1

- [x] Setting, clearing and reading a Cell each accept a Grid-minted index.
- [x] The existing number-taking forms still work and are still tested.
- [x] The typed form performs no bounds check of its own, because the index already carries one.
- [x] No caller is migrated in this ticket.

## Comments

Expand–contract because the blast radius is wide: the console, persistence, and roughly forty test
call sites address Cells by number. Adding the typed form first keeps every batch of the migration
green on its own.

Sequencing note worth weighing before starting: `spatial-tick-planning` 03–05 add producers that
write to Source, and may change what a planned write needs to carry. Doing this after those land
costs the same and lands on a settled shape. It is not blocked by them — it is a judgement about
when the shape stops moving.
