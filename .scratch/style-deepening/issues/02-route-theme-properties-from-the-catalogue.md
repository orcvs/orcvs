# 02 — Route Theme properties from the catalogue

**What to build:** `style_catalogue!` in `console/src/theme.rs` generates what is still written by hand from it: the `ColorKey`, `GridWidthKey` and `ChromeWidthKey` enums, and their six routing matches (`color_mut`, `color_ref`, `set_grid_width`, `grid_width`, `set_chrome_width`, `chrome_width`). Each is one `macro_rules!` arm beside `property_names`, fed by the same table, so adding a property is one catalogue line. The Theme's named fields and their typed widths stay as they are, and so does the derived document decoder in `theme_document.rs`.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] The three key enums and the six matches are generated from the catalogue; none is declared by hand.
- [x] The enums keep their variant order (the catalogue's, which a decoded document lists properties in), their derives and their `wasm32` `dead_code` expectations.
- [x] The generated readers (`color_ref`, `grid_width`, `chrome_width`) stay test-only, as the hand-written ones are today (`#[cfg(test)] impl Theme`, source-audit/11), and the writers stay shipped.
- [x] No new trait, code-generation crate or runtime registry.
- [x] The hand-written literal-value acceptance tests stay, so a generated reader and writer cannot both agree on a wrong field.
- [x] A Theme document still decodes, inherits and resolves exactly as before: the decoder, resolve and built-in record tests pass unchanged.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and `cargo nextest run --workspace --tests --no-default-features --locked` pass.
