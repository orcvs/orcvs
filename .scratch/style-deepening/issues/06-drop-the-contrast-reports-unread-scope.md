# 06 — Drop the contrast report's unread scope

**What to build:** `ContrastReport::scope` in `console/src/contrast.rs` is written from the `SCOPE` const and read only by the test `validate_reports_the_floor_and_scope_in_the_returned_data`; its one shipped caller, the Theme notice, shows the floor and the failing states. The field and its `dead_code` expectation go, and the test asserts the same things of `SCOPE`. The scope stays stated once, in the const and the module's `# Scope` section.

This reverses theming/08's choice to carry the scope in the report's data (kept in source-audit/11): shipped data nothing shipped reads is removed, and a caller that later wants to show the scope reads the const.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] `ContrastReport` has no `scope` field, and `contrast.rs` carries no `dead_code` expectation.
- [x] The test still fails if `SCOPE` stops naming text contrast or any excluded chrome case.
- [x] The module doc's reference to `ContrastReport::scope` points at `SCOPE` instead.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and `mise run check_wasm` pass.

## Comments

- `SCOPE` is `#[cfg(test)]`, because a shipped `SCOPE` with no shipped reader would need the `dead_code` expectation this ticket removes. A caller that later shows the scope makes it shipped again.
