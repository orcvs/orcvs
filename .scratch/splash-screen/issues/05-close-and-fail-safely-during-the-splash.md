# 05 — Close and fail safely during the splash

Status: resolved
Blocked by: 04

**What to build:** Quitting, closing or failing while the splash window is held leaves the console's
state and the screen as they would be without the splash.

- [x] Quitting during the hold stores the console's geometry, never the splash window's.
- [x] A close request during the hold ends the splash first, so the console's discard confirmation
      for an unsaved Source shows in a decorated window.
- [x] A failure to start the console is reported as it is today. No borderless window is left
      without a way to close it.
- [x] Tests through the console's egui test harness cover a save and a close request during the
      hold.

## Comments

A start failure has no splash to test: `Console::start` fails before the splash exists, eframe's
app creator returns the error, and eframe closes the window as it did before the splash. The
borderless window cannot outlive the process.
