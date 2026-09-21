# 10 — Make a custom Theme

**What to build:** A viewer makes a Theme of their own, one that inherits from exactly one built-in Theme and states only the values it changes. It is stored, it survives restart, and it appears in the pickers beside the built-in Themes.

**Blocked by:** 06 — Paint the Source from a scheme; 07 — Load a scheme at runtime.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] A custom Theme names one built-in Theme as its parent, and holds a name, an appearance, and only the slots and named keys it sets. Everything else resolves from its parent.
- [ ] A custom Theme can never be a parent. The editor offers only built-in Themes, and a stored custom Theme naming another is refused whole and reported.
- [ ] The editor starts a custom Theme from the Theme currently shown, and edits each slot and key with an alpha-capable colour picker, labelled in words ("Cursor frame"), not key spellings.
- [ ] Changes preview live. A value the viewer clears goes back to resolving from the parent.
- [ ] Custom Themes are stored as documents in the same storage `07` uses for loaded schemes, separate from settings. On the WASM target a document is a storage entry.
- [ ] With `persistence`, a custom Theme survives restart, and settings that name it resolve it.
- [ ] Deleting a custom Theme that settings name falls back to the default Theme of that appearance.
- [ ] `08`'s report is shown for the Theme being edited.
- [ ] `cargo nextest run --package console --locked`, the `--no-default-features` arm, and `mise run check_wasm` pass.

## Comments

**2026-09-21 — opened from `01`.** ADR 0053 removes overrides: a different look is a different Theme. This is the issue that makes that true for a viewer. It follows Helix's `inherits`, restricted to one level so there are no chains, and so a parent always exists because built-in Themes are compiled in.
