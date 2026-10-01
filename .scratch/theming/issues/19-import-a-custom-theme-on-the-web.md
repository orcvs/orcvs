# 19 — Import a custom Theme on the web

**What to build:** A viewer of the web console imports a custom Theme document, has it retained across reloads, and sees it selected. The document is the same versioned Orcvs Theme format `10` loads on native, parsed by `07`'s single document parser and resolved by `06`'s inheritance resolver; this issue introduces no second parser. Split from `10`, whose v1 scope is the native file path alone.

**Blocked by:** None — can start immediately.

**Status:** needs-triage

- [ ] Decide how the web selects a custom Theme. The web reads no config file (`Config::start` supplies the defaults there) and the in-app pickers are gone, so `theme.dark` / `theme.light` have no web equivalent. This is why web import was disabled for v1 (`.scratch/menu-structure/issues/04`): a dropped Theme was imported and stored but never presented. Record the decision here before implementation.
- [ ] A Theme file imported on the web is decoded and resolved through the same entry points as a native file. Its filename stem is its identity, following `07`; built-in identities are reserved, and a document naming a custom or unknown parent is refused whole and reported.
- [ ] A successful reimport of the same filename identity replaces the stored document.
- [ ] The imported document is retained with persistence and restored on the next load. A stored document that names a custom or unknown parent, or no longer parses, is refused whole and reported, as a native file is. Values earlier builds stored under `imported_themes` are not read.
- [ ] A custom Theme's appearance is its built-in parent's on the web loading path: tests cover an omitted appearance, a matching explicit appearance and a rejected conflicting declaration.
- [ ] Optional Cursor fills (`cursor.background`, `region.cursor.background`) are tested omitted, `"none"` and explicitly transparent through the web parser path, with the fallback `10` describes.
- [ ] `08`'s report is shown when an imported custom Theme is loaded.
- [ ] `cargo nextest run --package console --locked`, the `--no-default-features` arm, `mise run check_wasm` and the headless browser suite pass.

## Comments

**2026-10-01 — split from `10`.** `10`'s v1 scope deferred every WASM clause because the web Theme import is disabled. Its native halves are complete, so the web halves move here and `10` resolves on its native scope. The two clauses `10` left unticked (appearance and optional Cursor fills on the web path) and the web halves of its ticked lines (imported documents, reimport, stored web documents) are the criteria above. Not tagged `release/v1`: the v1 web console has the built-in Themes alone.
