# 05 — Close and fail safely during the splash

Status: ready-for-agent
Blocked by: 04

**What to build:** Quitting, closing or failing while the splash window is held leaves the console's
state and the screen as they would be without the splash.

- [ ] Quitting during the hold stores the console's geometry, never the splash window's.
- [ ] A close request during the hold ends the splash first, so the console's discard confirmation
      for an unsaved Source shows in a decorated window.
- [ ] A failure to start the console is reported as it is today. No borderless window is left
      without a way to close it.
- [ ] Tests through the console's egui test harness cover a quit and a close request during the
      hold, and a start failure.
