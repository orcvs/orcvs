# 03 — Choose the grayscale Themes' role borders

**What to build:** Grayscale Dark and Grayscale Light use role borders. The grayscale Themes already tell roles that share a glyph grey apart by fill strength and overlay borders rather than by hue. Role borders give them another channel, for example a Function border, and this ticket decides which roles get one and at what colour and width. The values are chosen by eye in the theme workbench and confirmed in the console with the tracker example open.

**Blocked by:** 02 — Give the other five roles a border

**Status:** ready-for-human

- [ ] For each of the six roles, a recorded decision for both grayscale Themes: a visible border with its colour and width, or none (transparent).
- [ ] The grayscale values table in the console's Theme record and the two built-in literals carry the chosen values, and the record tests pass.
- [ ] The Theme record's list of grayscale roles that share a grey, and what tells each apart, is updated where a role border now does that work, and the test that checks that list passes.
- [ ] The shipped-Theme contrast gate still passes for both grayscale Themes.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings`, `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.
