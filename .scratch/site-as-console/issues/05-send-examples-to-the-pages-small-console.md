# 05 — Send examples to the pages small console

**What to build:** A page with examples holds one small console (`01`). Pressing an example's Grid
starts the small console on first use and sends it the example by message. No link or control on
the site opens a new tab or window.

See `../spec.md`.

**Blocked by:** 01, 04

**Status:** resolved

- [x] Pressing two examples in turn opens each in the same small console, stopped.
- [x] No `target="_blank"` and no `window.open` remains in `site/`.
