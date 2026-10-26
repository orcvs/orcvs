# 03 — Keep the console's window geometry

Status: ready-for-agent
Blocked by: None — can start immediately

**What to build:** The native console saves and restores its own window size and position, and
eframe's restore of the stored geometry into the first window is turned off. Nothing changes for
the person using it. This prefactor lets the splash window open at its own size on every launch
(see [`../spec.md`](../spec.md)).

- [ ] Relaunching the console opens it at the size and position it was closed at, as today.
- [ ] With nothing stored, it opens at the default size, centred.
- [ ] A stored position off every connected monitor is brought back on screen, as eframe does
      today.
- [ ] Builds without the `persistence` feature open at the default size and store nothing.
- [ ] Tests through the console's egui test harness cover relaunch with and without a stored
      geometry.
- [ ] An ADR records that the console owns its window geometry and that the splash is the console's
      own window, borderless then resized; why a separate splash window was not chosen; and that
      turning eframe's restore back on breaks the splash on the second launch.
