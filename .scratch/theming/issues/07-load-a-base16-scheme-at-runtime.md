# 07 — Load versioned Orcvs Theme files

**What to build:** Discover native Orcvs Theme files at startup and import them on WASM through one versioned document parser. Base16 loading is deferred.

**Blocked by:** 11 — Validate chrome contrast through the painted composition; 06 — Paint the Source from a named Theme; 03 — Derive the console's chrome from the Theme; 04 — Decide the light built-in Theme; 08 — Validate a Theme’s composited contrast.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Parse the Orcvs format marker and version, metadata and typed `style` properties into the document type resolved by `06`. Reject absent/unknown format markers and unsupported versions without partial application. Built-ins and resolved custom documents use one rendering type.
- [ ] Document and implement the bounded native YAML subset, including quoted colours, decimal widths and explicit optional-fill clearing. Reject duplicate keys and wrong types. No Base16 layouts, slot inference or generic YAML evaluation are required. Keep the existing no-new-dependency constraint unless a rationale is recorded.
- [ ] Malformed input is refused whole and reported, never partly applied.
- [ ] Unknown appearance keys and out-of-range widths reject the entire Theme document. The error identifies the offending setting and explains the valid key or range; values are neither silently ignored nor clamped. A failed web reimport preserves the previous valid document. Contrast warnings alone do not reject a document. Tests cover unknown appearance keys, width range boundaries and failed replacement preserving the previous valid web document.
- [ ] If the selected native Theme file is missing or malformed, startup succeeds with the default built-in Theme of that appearance and shows an error identifying the Theme and file problem. The saved selection is retained, including across autosave; restoring or fixing the file restores the intended Theme on the next launch. A regression test covers fallback, save, file repair and restart.
- [ ] The native target scans `~/.orcvs/themes/` at startup and loads Theme files from that canonical directory. Settings select native Themes by filename stem; a document's `name` is only a display label. No additional XDG or other Theme directory is searched. The WASM target imports a file and uses its filename stem as the Theme identity, with the document name as a display label. Both targets reach the same parser.
- [ ] Built-in Theme identities are reserved. A file whose stem collides with one is refused and reported; it never replaces the built-in. Tests cover reserved identities, display-label changes preserving selection, and file renames changing identity.
- [ ] Reimporting a file with the same identity on WASM updates the stored document only after successful parsing and validation; malformed input leaves the previous document intact. Test replacement, persistence across restart, filename identity and reserved built-in identities. Contrast warnings alone do not reject an import.
- [ ] If multiple native files have the same filename stem, all files with that identity are refused and the conflict is reported with the conflicting filenames. Directory enumeration order never chooses a winner. If the selected identity is conflicted, use the default built-in Theme of the same appearance while retaining the saved selection; resolving the conflict restores the intended Theme on the next launch. Test duplicate identities with both directory enumeration orders, a selected conflicting identity, saved-selection preservation and recovery after conflict resolution.
- [ ] On native, Theme files remain authoritative and are read at every startup. Editing a file between runs changes the Theme on the next launch without re-importing it. Application storage never supplies a cached copy of the native file's Theme values.
- [ ] During a native session, file edits leave the loaded Theme unchanged until the next launch. This release has no file watching or reload action.
- [ ] With `persistence`, settings retain the selected Theme references. On WASM, imported Theme documents are also stored so their values survive restart; without `persistence`, imported documents and selections remain session-only. Native startup file reading is independent of saving selections.
- [ ] Resolve the custom document using `06`'s pure inheritance resolver; list it only in the picker matching that parent appearance. No background-lightness inference is needed.
- [ ] `08`'s validator runs on load and its report is shown. A Theme that fails the contrast floor is loaded anyway and reported, not refused — the viewer chose it.
- [ ] `mise run check_wasm` passes.


## Comments

**Why a loader at all, given compiled-in schemes exist.** The point of adopting base16 rather than a shape of our own is the several hundred published schemes. Compiled-in schemes prove the template; the loader is what makes the ecosystem reachable without a release.

**Scheme identity.** Settings reference a Theme rather than storing its resolved values. A built-in Theme resolves from the compiled-in definition. On native, a loaded scheme resolves from its authoritative file, read at startup. On WASM, it resolves from the imported document, retained in storage when persistence is enabled.

**2026-09-21 — revised for ADR 0053.** No overrides exist to key by scheme, so that acceptance line is replaced. A loaded scheme is stored as a Theme document, which is the same storage `10` uses for custom Themes.

**2026-09-21 — native files are authoritative.** The user confirmed: “On startup Orcvs reads the files.” This supersedes the native stored-copy requirement: startup rereads scheme files, rather than restoring their values from application storage. Web documents retain their storage-backed import path. File discovery and in-session reload behaviour remain to be decided.

**2026-09-21 — discovery location confirmed.** The user chose `~/.orcvs/themes/` as the canonical native directory, scanned at startup, with settings selecting Themes by name. There is no second search location or precedence rule. This settles discovery; in-session reload behaviour remains open.

**2026-09-21 — startup fallback confirmed.** For a missing or malformed selected Theme, use the default built-in Theme of the same appearance and show the error. Preserve the saved selection so repairing the file restores that Theme on the next launch. Fallback must not rewrite settings during autosave.

**2026-09-21 — reload and identity confirmed.** Native file edits take effect at startup only for this release. Settings reference the filename stem; a document's declared name is a display label. Built-in identities are reserved so a file cannot replace a built-in Theme. This settles the in-session reload question left open above.

**2026-09-21 — web file imports confirmed.** Replace pasted text with file import. The filename stem is the identity on both targets; reimporting the same filename updates the web document. Browser storage retains imported documents when persistence is enabled.

**2026-09-21 — strict appearance validation confirmed.** Unknown appearance keys and out-of-range widths reject the entire Theme document. The error identifies the offending setting and explains the valid key or range; values are neither silently ignored nor clamped. A failed web reimport preserves the previous valid document. Contrast warnings alone do not reject a document.

**2026-09-21 — duplicate native identities confirmed.** If multiple native files have the same filename stem, all files with that identity are refused and the conflict is reported with the conflicting filenames. Directory enumeration order never chooses a winner. If the selected identity is conflicted, use the default built-in Theme of the same appearance while retaining the saved selection; resolving the conflict restores the intended Theme on the next launch.

**2026-09-21 — published-format correction.** [Upstream examples](https://github.com/tinted-theming/home/blob/main/styling.md#yaml-scheme-examples) include legacy flat slots and modern nested `palette` slots. The earlier flat-only description would not satisfy loading published base16 schemes. Schema preparation is recorded in `../schema-draft.md`; proposed values there are not accepted defaults.

**2026-09-22 — one Orcvs format confirmed.** The user chose one versioned Orcvs Theme format with named style properties. Base16 is inspiration only; importing/conversion is deferred. This supersedes the earlier sixteen-slot, palette/template and published-scheme requirements. Native startup loading, web file imports, exact inheritance, strict validation and the confirmed appearance controls remain in scope.
