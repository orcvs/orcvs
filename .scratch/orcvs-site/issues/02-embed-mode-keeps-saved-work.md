# 02 — Embed mode keeps saved work and skips the service worker

**What to build:** A console opened with `?example=` or `#source=` is in embed mode. In embed mode it
neither restores the Source from eframe storage nor stores one, so the Source a reader keeps in
`/play/` is never read over or written over. In embed mode `console/index.html` does not register
`sw.js`. See `../spec.md` (Saved work, Service worker).

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] A test shows that with a stored Source, an embed-mode start opens the URL's Source, and that
      the stored Source is unchanged after the embed runs and edits.
- [ ] The same holds with `persistence` off:
      `cargo nextest run --workspace --tests --no-default-features --locked`.
- [ ] The browser suite (`mise run test_wasm`, deferred to CI) or a focused browser test shows
      embed mode registers no service worker and `/play/` still does.
