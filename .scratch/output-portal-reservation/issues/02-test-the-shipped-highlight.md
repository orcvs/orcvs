# 02 — Test the shipped highlight, not a copy

Status: ready-for-agent
Blocked by: 01

**What to build:**

Delete `LanguageMap::output_portal_cells`. Its callers in `language_map.rs`,
`tick.rs` and `tick/nested_copy/vertical_and_map.rs` read the shipped
`output_portal_highlight` instead. Replace the hand-copied scheduler projection
loops in `tick.rs`'s `output_portal_exclusion` with focused tests on the shared
function from ticket 01, including that a Source Bundle reserves no Output
Portal Reservation. See `../spec.md`.

## Acceptance criteria

- [ ] No `#[cfg(test)]` copy of the highlight remains.
- [ ] The Source Bundle exclusion is tested against the shared function.
- [ ] Every Function kind the old focused tests covered is still covered.
- [ ] Gates pass with `PROPTEST_CASES=32`: fmt, clippy and nextest on `orcvs`
  and `console`, then the workspace gates before the PR.
