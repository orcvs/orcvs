# 23 — Refuse invalid Grid dimensions through the error path

**What to build:** Constructing an Orcvs with a zero or oversized Grid returns an error rather than panicking. Today the Grid constructor asserts on its dimensions, so constructors that already return a `Result` still panic on bad dimensions.

**Blocked by:** None — can start immediately.

**Status:** obsoleted — e7232654 fixed the Grid at 256 by 256 (ADR 0054)

- [ ] Grid construction from untrusted dimensions is fallible, and the Orcvs constructors surface that failure as an error.
- [ ] Tests cover zero width, zero height, and dimensions beyond the Cell limit.
- [ ] Callers with dimensions known valid at compile time keep an infallible path, or state the invariant at the call.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Obsolete. `Grid::new()` takes no dimensions and cannot fail, and no shipped constructor takes dimensions. The asserting `Grid::with_shape` and `Orcvs::with_shape` are test-only, behind `cfg(any(test, feature = "test-grid-shapes"))`, which only dev-dependencies enable. Untrusted shapes are refused on the error path: a persisted Grid of another shape through `TryFrom<PersistedGrid>`, and an oversized Source File through `SourceFileError::TooManyLines` / `LineTooLong`.
