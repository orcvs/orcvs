# 01 — Record the rule and build Paint's test Theme from the default

**What to build:** The rule that nothing depends on a particular Theme is written down as an ADR, and the first fixture follows it. The ADR states that shipped code reaches a Theme through the Theme registry. A test fixture is the appearance's default Theme, reached through the registry's fallback, with only the fields the test needs overridden. Expected colours are read from the Theme, not copied as hex. Tests about shipped Themes cover every built-in the registry holds, and a parent or saved selection in a test is the appearance's default identity. Paint's `test_theme` becomes the dark default with a visible `cursor.background`, and the paint tests that use it read their expectations from it.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] A new ADR in `docs/adr/` records the rule and links ADR 0053. `docs/adr/README.md` lists it.
- [ ] Paint's `test_theme` is built from `ThemeRegistry::built_in().fallback(Appearance::Dark)` and names no built-in.
- [ ] Every test that uses `test_theme` asserts through the Theme's fields, not hex literals, and passes.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass.
