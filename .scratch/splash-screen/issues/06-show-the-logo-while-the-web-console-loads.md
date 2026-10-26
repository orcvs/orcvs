# 06 — Show the logo while the web console loads

Status: resolved
Blocked by: 02

**What to build:** The web page shows the splash logo and the name while the WASM
downloads and starts, in the native splash's look, and the console's start removes it.

- [x] The splash is the `loading_text` element, which the entry point already removes on start. On
      a failed start or a missing canvas, the crash message replaces the whole splash, logo
      included, as it replaces the element's contents today.
- [x] It shows the logo and `O R C V S` on the OS appearance's backdrop: the logo white on dark,
      inverted on light. The page cannot read the stored View mode or Theme before the WASM loads,
      so a viewer whose mode differs from the OS sees the OS appearance until the first frame. This
      is the same accepted limit as the page's backdrop colour.
- [x] Trunk publishes the logo through a `copy-file` link, `sw.js` pre-caches it, and the service
      worker's cache name is bumped so returning viewers fetch the new page.
- [x] The unused spinner CSS is removed, or used by the splash.
- [x] The browser regression suite still covers a missing canvas and a failed start.
