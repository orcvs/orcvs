# 28 — Copy a link to the running Source

**What to build:** A reader can get the site link that opens what the console is running, so they
can send it. The top bar's share icon, beside the mode toggle, puts it on the clipboard (ADR 0097).

Toby (2026-10-31): "There is no way of copying or accessing the actual link. We should add a share
button or something." Then: "Make it a share icon near the theme toggles."

**Acceptance:**

- An unchanged shipped example copies `https://orcvs.com/?example=<link-name>`.
- Any other Source copies `https://orcvs.com/#source=<text>`, which opens the same Source.
- A Source whose link would be over the linked console's limit raises a notice and copies nothing.

**Status:** resolved
