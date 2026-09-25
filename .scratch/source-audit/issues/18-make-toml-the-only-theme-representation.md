# 18 — Make TOML the only Theme representation and derive its decoding

**What to build:** A Theme document is a `.toml` file and nothing else. JSON and YAML Theme documents are dropped, and with them the `serde-saphyr` decoder and every crate only it pulls in. Decoding becomes a derived Serde structure: one optional field per style property (generated from the same declaration as the property keys), and validating newtypes for the format marker, version, labels, colours, widths and the optional Cursor fills. The hand-written visitors, the format dispatch (`enum Format`), the non-mapping-root guard, and the cross-format duplicate-stem handling (`SUFFIXES` in `theme_registry.rs`) all go, because TOML is typed, always has a table root, and its parser already rejects duplicate keys. The theming ADR (0053) and the Theme schema (`.scratch/theming/schema.md`) are corrected to state TOML as the only representation and record why the others were dropped. Theme files are native-only: since 8a5f9ef6 the web build loads only the built-in Themes. JSON Theme decoding is `console`'s only use of `serde_json`, so it leaves `console`'s dependencies too. `toml` stays regardless; `config.rs` reads `~/.orcvs/config.toml` with it.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] Only `.toml` files are Theme files; any other extension, `.json`, `.yaml` and `.yml` included, is reported as not a Theme file, and every message naming accepted extensions names only `.toml`.
- [x] `serde-saphyr` and `serde_json` are gone from the console manifest, no crate reachable only through them remains in the lockfile, and `mise run audit_deps` passes.
- [x] Theme decoding is derived: no hand-written `Visitor` or `DeserializeSeed` remains in it, and validation lives in the newtypes it deserializes into.
- [x] Unknown properties, repeated keys, wrong types, a wrong format marker or version, malformed colours, out-of-range or non-finite widths, and invalid labels are all still refused, each with a test; TOML errors still carry line and column.
- [x] The size limit, UTF-8 check and BOM handling are unchanged.
- [x] Duplicate-stem detection covers only what one extension can still produce (the case-variant spelling decision is made and tested); logic that existed only for cross-format stems is deleted.
- [x] Format-equivalence and JSON- and YAML-specific tests and fixtures are removed.
- [x] The theming ADR and the Theme schema state TOML as the only Theme representation, the decoding contract matches the derived structure, and the loss of the did-you-mean hint for mistyped property names is recorded (kept only if a small custom key parser is judged worth it).

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Core still valid on `main`. 8a5f9ef6 ("Disable web Theme import for v1") removed web import, web drops and stored web Themes, so the criteria about them are gone. The claim that web storage keeps `serde_json` was false; its removal is now a criterion. PR #136 implements this ticket and must rebase over 8a5f9ef6 and 5f35edc9.

**2026-09-25 — resolved.** Merged in orcvs/orcvs#136 (`0dd4fe9d`) after the `199c3331` audit, rebased over 8a5f9ef6 and 5f35edc9; every criterion verified on `main`.
