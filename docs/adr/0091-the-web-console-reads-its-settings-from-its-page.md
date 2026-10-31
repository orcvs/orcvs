# The web console reads its settings from its page

Status: accepted. Amends [ADR 0079](0079-source-cells-are-configurable-rectangles.md)'s "The web reads no settings file and draws the default". Settled in `.scratch/web-console-config/`.

A phone shows the web console at the native default: 12 by 18 point Cells too small to tap, and the Minimap covering a corner of a window that has no room for it. Native reads `~/.orcvs/config.toml`, and the web has no file, so the settings that would fix this cannot reach it. The Minimap and the Diagnostics have no setting at all: each starts as the console decides, and only the View menu changes it.

## Decision

**The web console reads `config.toml` text from the page that runs it.** `console/index.html` may hold one `<script type="application/toml" id="orcvs_config">` element. Its text is read by the same reader as native's file, with the same keys, bounds and notices. A `data-media` attribute holds a CSS media query, and the element applies only while the page matches it when the console starts; without the attribute it always applies. No element, or one whose media does not match, is the defaults. The page is read once, at startup, as native's file is.

**`[minimap] open` and `[diagnostics] open` are settings.** Each is a boolean saying whether that view is open when the console starts, on native and the web alike. The defaults are today's: the Minimap open, the Diagnostics closed. The View menu and its shortcuts still open and close either.

**The shipped page holds a phone setting.** Under `(max-width: 600px)` the console starts with 24 by 36 point Cells and the Minimap closed. The Grid stays 256 by 256 ([ADR 0054](0054-a-grid-is-always-256-by-256.md)); a phone shows fewer, larger Cells of it.

## Considered options

**URL parameters for each setting.** A second spelling of every key beside `config.toml`, and a page could not choose by viewport without rewriting its own URL.

**A phone layout chosen inside the console by viewport width.** It would hard-code one layout in Rust where the page that runs the console is already the thing a deployment controls.

**Posting settings to the console by message, as ADR 0086 posts a Source.** The settings are needed before the first frame, and a message arrives after the console starts.

## Consequences

- The page's settings report problems through the settings notice, naming the page rather than a file.
- A page that frames the console, such as the site, gets the phone setting from `/play/` itself and passes nothing.
