# 31 — Register the service worker by the console's own reading of a link

**What to build:** `console/index.html` registers `sw.js` exactly when `opening` in
`console/src/linked_source.rs` does not read the URL as a link and the URL does not name `small`.
[ADR 0113](../../../docs/adr/0113-a-link-is-refused-whole-and-only-the-full-console-caches-itself.md) decides it (SITE-082). Today the two decide apart and differ at the edges:
`?example` with no `=` registers no worker, yet opens the stored Source and autosaves, and
`#a&source=..` opens Linked, yet registers the worker.

**Blocked by:** None (can start immediately).

**Status:** needs-triage

- [ ] One reading of a link decides both what opens and whether the worker registers.
- [ ] `site/tests/smoke.spec.ts` covers both edge URLs.
- [ ] `docs/requirements/site.md` no longer lists SITE-082 as built differently.
