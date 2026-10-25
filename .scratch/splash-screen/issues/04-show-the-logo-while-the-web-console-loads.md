# 04 — Show the logo while the web console loads

Status: ready-for-agent
Blocked by: 02

**What to build:** The web page shows the splash logo and the name while the WASM
downloads and starts, in the native splash's look, and the console's start removes it.

- [ ] The page carries the `loading_text` element the entry point already removes on start and
      fills with the crash message on failure.
- [ ] It shows the logo and `O R C V S` on the OS appearance's backdrop: the logo
      white on dark, inverted on light.
- [ ] The browser regression suite still covers a missing canvas and a failed start.
