# 02 — Test the shipped highlight, not a copy

Status: resolved
Blocked by: 01

**What to build:**

Delete `LanguageMap::output_portal_cells`. Its callers in `language_map.rs`,
`tick.rs` and `tick/nested_copy/vertical_and_map.rs` read the shipped
`output_portal_highlight` instead. Replace the hand-copied scheduler projection
loops in `tick.rs`'s `output_portal_exclusion` with focused tests on the shared
function from ticket 01, including that a Source Bundle reserves no Output
Portal Reservation. See `../spec.md`.

## Acceptance criteria

- [x] No `#[cfg(test)]` copy of the highlight remains.
- [x] The Source Bundle exclusion is tested against the shared function.
- [x] Every Function kind the old focused tests covered is still covered.
- [x] Gates pass with `PROPTEST_CASES=32`: fmt, clippy and nextest on `orcvs`
  and `console`, then the workspace gates before the PR.

## Comments

2026-10-16 — `LanguageMap::output_portal_highlight` is the shipped highlight, and `SourceRevision::output_portal_highlight` returns it, so the language-map and nested-Copy picture tests now read what paint reads. `tick.rs`'s `output_portal_exclusion` module and the nested-Copy agreement test, each a hand copy of `Lookup::new`'s write-Claim loop, are replaced by four focused tests on `PortalAccess::output_portal_reservation` in `portal.rs`: root and nested value answers for Add and every Copy direction, Portals off the Grid or short of a pair, a Source Bundle the schedule reserves but the highlight does not, and Halt, Terminal Output and refused nested Functions. Workspace clippy, nextest (1713), doctests and rustdoc pass with `PROPTEST_CASES=32`.
