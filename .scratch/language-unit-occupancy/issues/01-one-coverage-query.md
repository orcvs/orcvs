# 01 — One coverage query for Copy, contact and Snapshot occupancy

Status: resolved

**What to build:** `portal::coverage`, `Coverage`, `Occupant` and
`WorkingSource::coverage`; `occupancy_of`, `WorkingSource::contact` and the
Copy read go through them. See `../spec.md`.

## Acceptance criteria

- [x] A Copy reads a mover that stepped one Cell into its Input Portal.
- [x] Coverage meets a unit only through the Cells it still holds.
- [x] Every existing occupancy, contact and Copy test passes unchanged.
- [x] Gates pass with `PROPTEST_CASES=32`.

## Comments

2026-10-16 — `a_copy_reads_a_mover_that_stepped_one_cell_into_its_input_portal` failed first (`"  >>=>  "` and a partial-input diagnostic) and passes now with `"  >>=>>>"`. `coverage_meets_a_unit_only_through_the_cells_it_still_holds` pins the visibility rule.
