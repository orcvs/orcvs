# 01 — Move each test cluster to its topic file

Status: resolved

**What to build:** see `../spec.md`.

## Acceptance criteria

- [x] `tick.rs` holds no test module.
- [x] The workspace runs the same test names before and after.
- [x] Gates pass with `PROPTEST_CASES=32`.

## Comments

2026-10-17 — Fourteen topic files and `fixture.rs`; the test name sets before and after are identical (1720).
