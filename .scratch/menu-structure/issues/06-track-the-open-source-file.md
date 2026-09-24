# 06 — Track the open Source File

**What to build:** On native the console knows the path of the Source File it has open, if any, and whether the Source changed since that file was opened or saved. The window title says both. Every action that discards the Source asks first when there are unsaved changes.

**Blocked by:** file-new/03.

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] The title reads the file name, or `Untitled`, with an unsaved marker while the Source differs from what was last opened or saved.
- [ ] New, Help > Function Reference, Quit and closing the window ask before discarding unsaved changes, through the confirmation `file-new/03` introduced. Cancelling changes nothing.
- [ ] An unchanged Source asks nothing.
- [ ] Autosave still stores the Source, and a restart restores it; whether the restored Source counts as unsaved against its file is decided and recorded here.
- [ ] Web builds track nothing and show no marker.
- [ ] Tests drive each discarding action through the console with and without unsaved changes, confirmed and cancelled.
- [ ] Scoped gates for `console` pass.
