# 05 — Name the default as the parent in document and registry tests

**What to build:** Theme document, Theme registry, Theme selection, config and storage tests that need a parent Theme or a saved selection use the appearance's default identity, not `okabe-ito` or `orcvs-light`. A test about a wrong-appearance selection uses the other appearance's default. A test about a custom file claiming a reserved identity claims a default's. Document tests that check inherited values read them from the default Theme. The test that resolves the Okabe–Ito copy example is left for the retirement ticket, which deletes the example.

**Blocked by:** 01 — Record the rule and build Paint's test Theme from the default.

**Status:** ready-for-agent

- [ ] Those modules' tests name no built-in other than through `ThemeIdentity::default_for` or the registry's fallback, apart from the Okabe–Ito copy example test.
- [ ] The config module's doc example names a default identity.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass, and so does `cargo nextest run --workspace --tests --no-default-features --locked` for the storage tests.
