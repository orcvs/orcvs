# The web console takes settings from its page

Status: ready-for-agent

## Problem Statement

Toby (2026-10-28): "Can we pass config into the web console? Because for mobile-friendly web we
need much smaller grid and bigger cells. Need to turn the overview off." The web runs on the
default settings, because it has no `~/.orcvs/config.toml`, and the Overview has no setting.

## Solution

Toby's direction, 2026-10-28:

- "These should all be existing parameters." The web reads the `config.toml` keys native reads.
- "How is the console executed - isn't that the place to inject parameters? We control the html."
  `console/index.html` holds the settings as `config.toml` text, and a media query chooses when they
  apply (ADR 0090).
- "We should expose overview AND diagnostics as keys." `[overview] open` and `[diagnostics] open`.

The Grid stays 256 by 256 (ADR 0054). A phone gets 24 by 36 point Cells, so it shows fewer of them.

## Out of scope

Nothing ADR 0090 decides.
