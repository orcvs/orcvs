# A splash screen with the Orcvs logo

Status: ready-for-agent

## The decision

The native console opens as a splash window that becomes the console. Toby chose this on
2026-10-25 from three native prototypes (variant B; see ticket 01).

Native is the point. Orcvs is used as a native application; the web build is a marketing channel
and is rarely the real usage. The web loading screen comes second and borrows the native look.

## What the splash is

- The application opens as a small, borderless, centred window: the logo, `O R C V S` set in the
  console's font, and the version beneath it, on the presented Theme's window backdrop.
- After a short hold, or at the first key press or click, the same window gains its decorations,
  grows to the console's window size and position, and the logo fades out over the Grid, which is
  already running beneath it.
- The logo is tinted white on a dark appearance, which reproduces the artwork, and black on a light
  one.

## Constraints the prototype found

- egui and eframe have no splash API. Nothing runs before the first window opens, so the splash is
  the root viewport itself, resized by viewport commands.
- eframe's own icon decoder, which already decodes the window icon, decodes the logo. No dependency
  is added.
- The Playback Engine and the Source start as they do today. The splash hides nothing that is
  loading: native startup is quick, so the hold is for presentation, and input ends it early.
- eframe restores the stored window geometry when persistence is on. The splash must not overwrite
  it, and the console must open at the restored size and position afterwards.
- The web build cannot run egui until the WASM has loaded, so its loading screen is HTML in the
  page. The entry point already removes a `loading_text` element on start and writes the crash
  message into it on failure; the page lacks the element.

## The logo asset

`logo-splash.png` is derived from Toby's `orcvs.png` (1254 px, RGB, about 2 MB, untracked). The
artwork is greyscale, so its brightness becomes alpha over a white fill: drawn untinted on black it
is the original, and tinted or inverted it serves a light appearance. At 512 px, posterized to 32
levels, it is 76 KB. The source artwork is not committed.
