# 32 — Answer every /play/ file from the network first

**What to build:** `console/assets/sw.js` answers every `/play/` request from the network first,
caching each good response and falling back to the cache only offline, so a deploy reaches a
returning reader on their next load for every file. [ADR 0113](../../../docs/adr/0113-a-link-is-refused-whole-and-only-the-full-console-caches-itself.md) decides it (SITE-083). Today
only navigations, `console.js` and `console_bg.wasm` go to the network first; every other cached
file, `logo-splash.png` among them, is served from the cache first and changes for a returning
reader only when `sw.js` itself changes.

**Blocked by:** None (can start immediately).

**Status:** needs-triage

- [ ] Every `/play/` file is network-first, and offline still serves the last cached console.
- [ ] `site/tests/deploy.spec.ts` covers a file other than the page and `console.js`.
- [ ] `docs/requirements/site.md` no longer lists SITE-083 as built differently.
