# 03 — Move the paint and Overview tests onto the default Theme

**What to build:** Paint and Overview tests check edges, fills, role borders, Portals, Locked Cells and the Overview's picture without naming a built-in. Each takes the default Theme for its appearance and overrides only what it asserts on. Tests that probe Portal and Locked fills set distinct fills themselves, because Grayscale Dark fills both Portals alike. A test that resolves a Theme document names the appearance's default identity as its parent.

**Blocked by:** 01 — Record the rule and build Paint's test Theme from the default.

**Status:** resolved

- [x] The paint and Overview tests name no built-in Theme function or identity outside the registry's fallback.
- [x] Expected colours are read from the Theme the test uses.
- [x] The Theme-document paint tests use `ThemeIdentity::default_for` for `inherits`.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass.

## Comments

Paint's tests moved with 01. Its three Theme-document tests now build their documents with `tests_support::dark`, which inherits `ThemeIdentity::default_for(Appearance::Dark)`. The Overview tests use `default_theme`.
