# 04 — Run the contrast and colour-vision gates over every built-in

**What to build:** The shipped-Theme contrast gate and colour-vision gate cover every built-in the Theme registry holds, and no gate test names a built-in. Tests that state a per-Theme fact, such as a named Theme having nothing to except, become a property every built-in must satisfy, or go where the general gate already says it. The generic contrast tests (measurement, compositing, the Comment exemption, transparent glyphs) take the default Theme and override what they need.

**Blocked by:** 01 — Record the rule and build Paint's test Theme from the default.

**Status:** ready-for-agent

- [ ] `shipped_theme_gate` and `shipped_theme_colour_vision_gate` iterate the registry's built-ins.
- [ ] No contrast or colour-vision test names a built-in Theme function or identity.
- [ ] The ticket records in its comments whether the colour-vision thresholds still say something for grayscale built-ins whose only hues are the Bang and Diagnostic. The gate is not deleted.
- [ ] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass.
