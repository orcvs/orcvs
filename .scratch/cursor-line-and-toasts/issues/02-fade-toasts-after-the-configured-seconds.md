# 02 — Fade Toasts after the configured seconds

Status: ready-for-agent
Type: task
Blocked by: 01

**What to build:** a Toast fades out after `[toasts] seconds` ([spec](../spec.md), ADR 0100).

- [ ] `[toasts] seconds`, a whole number from 1 to 60, default 5, read by `Config` like every other
      key, and from the page on the web (ADR 0091).
- [ ] `console/src/config.rs`'s header documents the key; an out-of-range or mistyped value is the
      usual notice and the default applies.
- [ ] A Toast fades out once its seconds have passed since it was raised, and a faded Toast does not
      return for the same message.
- [ ] The console asks egui for a repaint while a Toast is showing or fading, and for none once the
      last one has gone.
- [ ] Tests: the settings reader's cases for the key, and a kittest that advances past the
      configured seconds and finds the Toast gone.
