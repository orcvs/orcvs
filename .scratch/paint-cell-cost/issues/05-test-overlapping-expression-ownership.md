# 05 — Test overlapping Expression ownership in the Claim index

**What to build:** A test for the branch where a later Span clears an earlier claim's ownership of a Cell. `03` required focused tests for overlapping Expression ownership, and none exercises it.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] A Source whose Expressions overlap reaches `by_index[index] = None` (`orcvs/src/language_map.rs:421-423`). The test asserts which Cells keep which Claim, and what their written state reads through `RenderCell`.
- [ ] The test fails when that clearing line is removed. Confirm this by breaking it once, then revert.
- [ ] `cargo nextest run --package orcvs --locked` and `--package console` pass.

## Comments

**2026-09-23 — opened by the audit of the merged pull requests against their issues.** #115 added two `render_frame` tests, neither with overlapping Spans.
