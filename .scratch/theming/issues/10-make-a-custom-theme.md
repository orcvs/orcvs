# 10 — Load a custom Theme file

**What to build:** Load a custom Theme authored as a file outside Orcvs. It inherits from exactly one built-in Theme and states only the values it changes. On native its file is read at startup; on WASM its imported document is retained with persistence. It is selected by identity in `~/.orcvs/config.toml` (`theme.dark` / `theme.light`), exactly as a built-in Theme is; there are no in-app pickers and no in-app Theme editor.

**Blocked by:** 06 — Paint the Source from a named Theme; 07 — Load versioned Orcvs Theme files; 08 — Validate a Theme’s composited contrast; 17 — Prove the transparent fills and Grids a custom Theme can load.

**Status:** ready-for-agent

**Tags:** release/v1

**Scope for v1:** the web Theme import is disabled (`.scratch/menu-structure/issues/04`), so the WASM clauses below — imported documents, web reimport, the web parser path and stored web documents — are deferred past v1. v1 is the native file path; the web has the built-ins alone.

- [x] A custom Theme references one built-in Theme identity as its parent, and holds a display name and only the named properties it sets; its appearance is inherited from its parent. Everything else resolves from its parent. On both targets, its filename stem is its identity, following `07`; the display name is not a settings reference.
- [ ] A custom Theme inherits its built-in parent's dark/light appearance. An omitted appearance resolves from the parent; an explicit conflicting declaration rejects the document with an error identifying the mismatch. A light custom Theme must start from a light built-in, and a dark custom Theme from a dark built-in. Tests cover omitted appearance, matching explicit appearance and a rejected conflicting declaration on both loading paths.
- [x] A custom Theme can never be a parent. A loaded or stored custom Theme naming another, or naming an unknown parent, is refused whole and reported.
- [x] The versioned Orcvs format (`format = "orcvs-theme"`, `version = 1`, `name`, `inherits`, `style`) is documented with a complete example a viewer can edit in a file outside Orcvs, including colour alpha, role backgrounds, Grid and Cell colours, borders and border widths. Width values are finite display-point numbers: 0–1 inclusive for Grid/Cell borders and seams, 0–2 inclusive for chrome borders; zero hides the stroke. The parser validates the documented types, units and ranges; omitted values inherit from the built-in parent. Unknown appearance keys and out-of-range widths reject the whole document with an error identifying the setting, as in `07`; there is no silent ignoring or clamping. Font choice and custom font assets are not part of the planned format; do not add speculative font-loading support.
- [x] Custom documents use `07`'s loading entry points: files discovered in `~/.orcvs/themes/` at native startup, file imports on WASM, with a successful reimport of the same filename identity updating the stored document. Both use `07`'s single document parser and `06`'s inheritance resolver; this issue does not introduce another parser.
- [x] A named property omitted from the document resolves from the parent. An explicit transparent colour remains distinct from omission.
- [ ] Optional Cursor fills distinguish omission, none and explicit transparent colour. Omission inherits the parent value. None removes the optional fill and uses the existing fallback: for the Cursor inside a Region, it falls back to the ordinary Cursor fill; for the ordinary Cursor, it supplies no Cursor fill. An explicit transparent colour remains a supplied value and does not trigger that fallback. These optional states apply only to `cursor.background` and `region.cursor.background`, not to every colour key. Use the `"none"` explicit-clear spelling in `../schema.md` and test omission, clear and transparent values through both native and web parsers. Reject a nonopaque `window.background`; this key is the opaque backdrop, not an optional fill.
- [x] A custom Theme inherits its parent's resolved named properties and replaces only properties explicitly supplied by the document. Omitted properties remain unchanged; no palette-slot recalculation exists. Tests show that changing one Source property leaves inherited chrome properties unchanged, and that transparent explicit values replace inherited colours.
- [x] Custom Themes use the same document-loading mechanism as `07`: authoritative files on native, imported documents on WASM, separate from settings.
- [x] On native, custom Theme files are read at each startup, and an external edit is reflected only on the next launch without re-importing. There is no file watcher or reload action in this release. Application storage never restores a stale copy of a native Theme file. Built-in identities are reserved as in `07`. Duplicate native filename stems follow `07`'s conflict rule, across all Orcvs Theme files.
- [x] `~/.orcvs/config.toml`'s `theme.dark` and `theme.light` hold Theme references (`console/src/config.rs:158-159`), not application storage. A reference naming a custom Theme resolves it from the freshly read native file.
- [x] A missing or malformed selected custom Theme follows `07`'s startup fallback: use the default built-in Theme of that appearance, show the file error, and leave the selection in `config.toml` untouched. Restoring or fixing the file restores the selected custom Theme on the next launch.
- [x] `08`'s report is shown when the custom Theme is loaded.
- [ ] `cargo nextest run --package console --locked`, the `--no-default-features` arm, and `mise run check_wasm` pass.

## Comments

**2026-09-21 — opened from `01`.** ADR 0053 removes overrides: a different look is a different Theme. This is the issue that makes that true for a viewer. It follows Helix's `inherits`, restricted to one level so there are no chains, and so a parent always exists because built-in Themes are compiled in.

**2026-09-21 — corrected by the user: file authoring, no editor.** “Making a theme is a file change. No editor.” The former acceptance for colour pickers, live previews and starting an editor from the currently displayed Theme is removed. This issue implements loading externally authored custom Theme documents. It does not require an in-app creation or editing workflow. Reload behaviour after an external edit remains to be decided; this correction does not imply file watching.

**2026-09-21 — startup reading confirmed.** The user confirmed: “On startup Orcvs reads the files.” Native files are authoritative and are reread on launch. Only behaviour for edits during a running session remains open; this does not add a file watcher or reload control.

**2026-09-21 — startup-only reload and file identity confirmed.** The user confirmed that edits during a session take effect on the next launch only. The filename stem identifies a native custom Theme; its declared name is a display label, and built-in identities are reserved. This settles the open reload question above.

**2026-09-21 — parent appearance confirmed.** A custom Theme inherits its built-in parent's dark/light appearance. An omitted appearance resolves from the parent; an explicit conflicting declaration rejects the document with an error identifying the mismatch. A light custom Theme must start from a light built-in, and a dark custom Theme from a dark built-in.

**2026-09-21 — exact inheritance confirmed.** A custom Theme inherits its parent's resolved slot and named-key values, then replaces only values explicitly supplied by the custom document. Changing a slot does not recalculate inherited named-key values. For example, changing `base05` changes Ordinary Source glyphs but leaves the parent's explicit `text` colour unchanged; changing both requires both entries. Slot-derived defaults complete a bare published scheme, not a second pass that rewrites inherited custom-Theme values.

**2026-09-22 — one Orcvs format confirmed.** The user chose one versioned Orcvs Theme format with named style properties. Base16 is inspiration only; importing/conversion is deferred. This supersedes the earlier sixteen-slot, palette/template and published-scheme requirements. Native startup loading, web file imports, exact inheritance, strict validation and the confirmed appearance controls remain in scope.

**2026-09-22 — explicit role backgrounds confirmed.** Role backgrounds are explicit colour properties, such as `source.function.background`, `source.number.background`, `diagnostic.background` and `output_portal.background`. They composite over the uniform `cell.background` under the confirmed fixed precedence. Changing a role foreground does not recalculate its background. There is no `fill_tint` property or shared tint-strength setting in the Orcvs Theme format. Extract and verify the built-in role background colours from current rendering so its appearance is preserved; illustrative colours in the interview are not accepted defaults.

### Audit at cad296df — 2026-09-29

Most of this ticket is implemented on the native path; the ticked lines were checked against the
code. The WASM clauses stay deferred under the v1 scope note above, and each ticked line counts
its native half only. What remains:

- `17` is still open, so this ticket stays blocked, whatever its status line says.
- The line on inheriting resolved properties is not ticked. `omitted_properties_inherit_from_the_parent`
  (`console/src/theme.rs:1574`) and `foreground_change_leaves_inherited_background_unchanged`
  (`:1613`) exist, but no test was confirmed to show that one Source property leaves inherited
  chrome unchanged.
- The gates line is not ticked; they were not run in this audit.

Selection moved to a config file in 5f35edc9. `no_menu_offers_a_setting`
(`console/src/console/kittest_tests.rs:271`) asserts that no menu offers a Theme, so the pickers
and `persistence`-settings clauses are rewritten against `~/.orcvs/config.toml`.

Evidence for the ticked lines:

| Line | Evidence |
|---|---|
| Parent and stem identity | Built-in-only parent lookup (`theme.rs:1129-1134`); `a_file_name_stem_is_its_identity_matched_case_insensitively_by_suffix` (`theme_registry.rs:517`) |
| Appearance | `appearance_mismatch_is_rejected` (`theme.rs:1712`), `matching_declared_appearance_is_accepted` (`:1731`) |
| Never a parent | `unknown_parent_is_rejected` (`:1654`), `a_custom_theme_cannot_chain_from_another_custom_theme` (`:1677`) |
| Format | `.scratch/theming/examples/okabe-ito-copy.toml`, parsed by `theme_document.rs:478`; width range tests (`theme.rs:1805`, `:1832`, `:1868`) |
| Omission and transparency | `theme.rs:1574`, `:1927` |
| Optional fills | `theme.rs:1888`, `:1903`, `:1927`; nonopaque `window.background` rejected (`:1757`) |
| Startup read, reserved identities, duplicate stems | `theme_registry.rs:120-126`, `:755`, `:850` |
| Selection fallback and repair | `theme_registry.rs:1181`, `:1209` |
| `08` report | `a_theme_below_the_contrast_floor_is_loaded_and_its_report_shown` (`theme_registry.rs:580`) |

### Independent implementation audit at c34bccb6 — 2026-09-29

The previous audit missed `explicit_color_overrides_the_parent`
(`console/src/theme.rs:1590-1605`): it changes the Source Grid background and
asserts the inherited chrome `panel_background` is unchanged. Together with
`explicit_transparent_colour_is_a_supplied_value_not_a_clear` (`:1927`), the
existing tests cover the inheritance criterion, now ticked. `resolve` clones
the parent and changes only supplied keys (`:1150-1155`). Issue `17` and the
verification criterion still keep this ticket open. This correction claims
source inspection; test execution is recorded in the audit report.

Both boxes that ask for tests on both loading paths are unticked again. Native covers the matching
(`console/src/theme.rs:1731`) and conflicting (`:1712`) appearance declarations, but no test
resolves a document that omits `appearance`. The web half of both boxes is deferred past v1 by the
scope note above, so neither box can be ticked on the native half alone.

**2026-09-29 — `17` resolved; this ticket is unblocked.** `17` landed its Paint-level Region
Cursor fill test and its composited Grid test, so every issue on the `Blocked by:` line (`06`,
`07`, `08`, `17`) is resolved. The blocking notes in both audits above no longer hold. The status
stays `ready-for-agent`: the appearance box (no test resolves a document that omits
`appearance`), the optional Cursor fills box (its web half is deferred past v1, so the native half
alone cannot tick it) and the gates box remain open.
