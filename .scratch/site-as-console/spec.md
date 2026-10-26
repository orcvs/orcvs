# The site is the console

Status: resolved

## Problem Statement

The site PR #313 shipped is a Starlight docs site with a hero, prose written for it, and examples
that launch a console or open it in a new tab. Toby (2026-10-26): "Truly awful site", "There is no
look. Orcvs already has an aesthetic", "The console embed does not work". The console already has
a look: its Grid, its Monaspace Neon font, its Grayscale Dark and Light Themes, its splash and
logo.

## Solution

Toby's direction, 2026-10-26:

- "The site should be the console." The home page is the console. Layout: [menubar] [console]
  [sound module]. The menubar is the console's own.
- "The console should already open on the splash. We should not need to replicate." The site
  shows the console's own splash (`console/index.html`) and draws no splash of its own.
- "We have light and dark color themes the site should follow." Every page uses the Grayscale Dark
  and Light colours (`console/src/theme.md`) and Monaspace Neon, and follows the reader's
  appearance.
- "I do not want your copy." Pages carry the repository's own docs (`docs/first-result.md`,
  `docs/functions.md`), not prose written for the site.
- "Function source code references should use a grid." Examples are drawn as Cells.
- "Launching examples in a new window is bad." Nothing opens a new tab or window.
- "Can we embed a smaller console and pass messages to it for code examples?" A page with examples
  holds one small console and sends it each example by message (ADR 0086).
- Learn and Reference are reached from the console's Help menu (`orcvs-site/13`).

The prototype is on the `prototype/site-as-console` branch (`site/prototype-console-site/`).

## Implementation Decisions

- **Home.** `/` is a page whose console fills the window: `/play/` in a same-origin frame, with the
  sound module below it once the built-in synth exists (`orcvs-site/07`, `08`).
- **Small console.** `/play/?small` (ADR 0086): no menu bar, Linked, listens for `orcvs.open`.
- **Learn and Reference.** Starlight renders the repository's docs. Its colours come from the
  Grayscale Themes and its fonts from Monaspace Neon. The site's own pages hold no prose of their
  own beyond titles. The About page and the home page's hero go.
- **Examples.** Each ` ```orcvs ` and ` ```text ` block whose content is Source is drawn as a Grid
  of Cells. Pressing an example sends it to the page's small console, which the page starts on the
  first press.
- **Supersedes.** `orcvs-site`'s launch-on-click embed and its new-tab links. The URL opening it
  added stays for links.

## Out of Scope

- The sound module itself, which waits on the built-in synth.
