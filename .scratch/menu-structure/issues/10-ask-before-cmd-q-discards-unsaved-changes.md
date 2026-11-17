# 10 — Ask before ⌘Q discards unsaved changes

**What to build:** On macOS, ⌘Q from the app menu asks before discarding unsaved changes, as File →
Quit and the window's close button do. [ADR 0111](../../../docs/adr/0111-the-native-console-edits-one-source-file-and-autosave-only-recovers.md) decides it (CONS-135). Today the app
menu's Quit ends the process without a close request, so the console never sees it, as the File
menu's comment in `console/src/console/menu_bar.rs` records.

**Blocked by:** None (can start immediately).

**Status:** needs-triage

- [ ] ⌘Q with unsaved changes shows the same Discard and Cancel question, and Cancel keeps the
      console open.
- [ ] ⌘Q with nothing unsaved quits at once.
- [ ] File → Quit shows the ⌘Q chord once it asks.
- [ ] `docs/requirements/console.md` no longer lists CONS-135 as built differently.
