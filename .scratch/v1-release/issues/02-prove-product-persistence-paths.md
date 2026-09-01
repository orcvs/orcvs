# 02 — Prove product persistence paths

**What to build:** Prove that persistence stores Grid and character Source as authority, rejects
malformed state, rebuilds every derived language view, and restores through the actual shipped
native and WASM storage integrations.

**Blocked by:** language-map/03 — Move Source consumers behind the Language Map.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] Automated model tests round-trip non-square edited Source, preserve Grid and Cells, reject
      malformed dimensions, and compare rebuilt Language Map, Glyph, and executable behavior.
- [ ] Native save, restart, and reload follows the shipped storage path and records a repeatable
      procedure and result.
- [ ] WASM save, browser restart/reload, and restore follows the shipped storage path and records a
      repeatable procedure and result.
- [ ] Automated end-to-end coverage replaces a manual smoke wherever the host integration permits
      reliable automation.
- [ ] The exact candidate gate reruns the automated proof; its final record links the native and
      WASM product-path evidence for the nominated SHA.
