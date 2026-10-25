# 02 — Move the style tests onto the default Theme

**What to build:** The style module's tests check Cell visuals, border composition and chrome without naming a built-in. Each test takes the default Theme for its appearance and overrides only the fields its behaviour depends on. Expected colours come from that Theme's fields. Tests that check one built-in's chrome against the Theme record become one check over every built-in the registry holds. The byte baseline frozen from Okabe–Ito's composed visuals is deleted, because field-reading tests cover the behaviour it guarded.

**Blocked by:** 01 — Record the rule and build Paint's test Theme from the default.

**Status:** ready-for-agent

- [ ] The style module's tests name no built-in Theme function or identity.
- [ ] The chrome-record check runs over every built-in the registry holds and reads each one's table in the Theme record.
- [ ] `okabe_ito_reproduces_the_pre_refactor_cell_visuals_exactly` is gone, and the behaviours it covered (unclaimed, claimed, Portal, Locked, Diagnostic, Cursor and Region states) each have a test that reads the Theme's fields.
- [ ] Where the default sets a needed channel transparent, or sets two channels alike, the test overrides those fields and says why.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass.
