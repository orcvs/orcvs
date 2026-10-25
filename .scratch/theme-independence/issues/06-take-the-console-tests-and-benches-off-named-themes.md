# 06 — Take the console tests and benches off named Themes

**What to build:** The console's unit and kittest tests, the release capture, and the paint and Overview benches name no built-in. `ticked_console` presents the dark default with its Portal and Locked fills overridden to differ from each other and from the Grid background, so the Portal and lock tests still tell the roles apart by fill. The benches paint with the dark default. The release-capture test captures each viewport in each appearance's default.

**Blocked by:** 01 — Record the rule and build Paint's test Theme from the default.

**Status:** ready-for-agent

- [ ] The console's tests and the benches name no built-in Theme function or identity outside the registry's fallback.
- [ ] `ticked_console`'s overrides say why each field is set.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass, and `mise run check_release_capture` passes.
