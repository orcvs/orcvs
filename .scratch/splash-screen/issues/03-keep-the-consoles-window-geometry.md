# 03 — Keep the console's window geometry

Status: resolved
Blocked by: None — can start immediately

**What to build:** The native console saves and restores its own window size and position, and
eframe's restore of the stored geometry into the first window is turned off. Nothing changes for
the person using it. This prefactor lets the splash window open at its own size on every launch
(see [`../spec.md`](../spec.md)).

- [x] Relaunching the console opens it at the size and position it was closed at, as today.
- [x] With nothing stored, it opens at the default size, centred.
- [x] A restored size is kept within the monitor the window opens on; the position is left to the
      platform, as ADR 0083 records.
- [x] Builds without the `persistence` feature open at the default size and store nothing.
- [x] Tests cover storing and reading back a geometry and the commands that restore it; relaunch
      through the window is covered with the splash in 04.
- [x] An ADR records that the console owns its window geometry and that the splash is the console's
      own window, borderless then resized; why a separate splash window was not chosen; and that
      turning eframe's restore back on breaks the splash on the second launch.
