# 17 — Prove the transparent fills and Grids a custom Theme can load

**What to build:** Tests that show a translucent or transparent value a custom Theme supplies paints as the schema says. `10` lets users load exactly these values, and today only the resolver covers them. Moved from `13`.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] A Paint-level test shows that an explicit transparent `region.cursor.background` suppresses the fallback to the Cursor fill for the Cursor inside a Region, while an omitted value falls back.
- [ ] A test measures the composited pixel colour of a transparent Grid and of a partial-alpha Grid over the console surface, rather than only checking that the fill is passed through.
- [ ] `cargo nextest run --package console --locked` and the `--no-default-features` arm pass.

## Comments

**2026-09-24 — split from `13` by the release-membership audit.** These two lines are release evidence for `10`'s acceptance ("an explicit transparent colour remains a supplied value and does not trigger that fallback") and for `07`'s translucent loaded Theme backdrop. Blocks `10`.
