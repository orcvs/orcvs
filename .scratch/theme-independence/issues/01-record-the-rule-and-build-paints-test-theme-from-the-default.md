# 01 — Record the rule and build Paint's test Theme from the default

**What to build:** The rule that nothing depends on a particular Theme is written down as an ADR, and the first fixture follows it. The ADR states that shipped code reaches a Theme through the Theme registry. A test fixture is the appearance's default Theme, reached through the registry's fallback, with only the fields the test needs overridden. Expected colours are read from the Theme, not copied as hex. Tests about shipped Themes cover every built-in the registry holds, and a parent or saved selection in a test is the appearance's default identity. Paint's `test_theme` becomes the dark default with a visible `cursor.background`, and the paint tests that use it read their expectations from it.

**Blocked by:** None (can start immediately).

**Status:** resolved

- [x] A new ADR in `docs/adr/` records the rule and links ADR 0053. `docs/adr/README.md` lists it. (Moot: the README lists no ADRs, see Comments.)
- [x] Paint's `test_theme` is built from `ThemeRegistry::built_in().fallback(Appearance::Dark)` and names no built-in.
- [x] Every test that uses `test_theme` asserts through the Theme's fields, not hex literals, and passes.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass.

## Comments

The ADR is `docs/adr/0082-nothing-depends-on-a-particular-theme.md`. `docs/adr/README.md` holds the numbering rules and lists no ADRs, so there was nothing to add there.

`theme_registry::tests_support::default_theme(appearance)` clones `ThemeRegistry::built_in().fallback(appearance)`, and every test fixture starts from it. Paint's `test_theme` is that call for dark with `cursor.background` set. The other paint tests moved at the same time, because they compare against `test_theme`'s output.
