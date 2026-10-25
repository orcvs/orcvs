# 04 — Run the contrast and colour-vision gates over every built-in

**What to build:** The shipped-Theme contrast gate and colour-vision gate cover every built-in the Theme registry holds, and no gate test names a built-in. Tests that state a per-Theme fact, such as a named Theme having nothing to except, become a property every built-in must satisfy, or go where the general gate already says it. The generic contrast tests (measurement, compositing, the Comment exemption, transparent glyphs) take the default Theme and override what they need.

**Blocked by:** 01 — Record the rule and build Paint's test Theme from the default.

**Status:** resolved

- [x] `shipped_theme_gate` and `shipped_theme_colour_vision_gate` iterate the registry's built-ins.
- [x] No contrast or colour-vision test names a built-in Theme function or identity.
- [x] The ticket records in its comments whether the colour-vision thresholds still say something for grayscale built-ins whose only hues are the Bang and Diagnostic. The gate is not deleted.
- [x] `cargo fmt --all -- --check`, `cargo clippy --package console --all-targets --locked -- -D warnings` and `cargo nextest run --package console --locked` pass.

## Comments

`shipped_theme_gate` already ran over the registry. `shipped_theme_colour_vision_gate` now does too, and it absorbs `the_grayscale_built_ins_lose_no_pair_to_a_dichromacy`. A pair below the floor fails only when at least one of its colours is not a neutral grey. `okabe_ito_has_nothing_to_except` and `orcvs_light_has_nothing_to_except` became `no_built_in_has_an_accepted_exception`. The per-Theme tritan and red–green figure pins are removed.

On the thresholds for grayscale built-ins: CONFUSION_FLOOR still means something, but only for pairs that include a hued channel. With two grayscale built-ins, those are the Bang and the Diagnostic, measured against every other channel. A pair of two neutral greys is exempt because no dichromacy moves a neutral (`simulation_leaves_neutrals_alone`): such a pair looks the same to a dichromat as to anyone else, and the contrast gate covers telling greys apart. Tritanopia is still measured and not gated. The gate is kept.
