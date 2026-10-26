# The console opens through its own window as a splash

Status: accepted. Settled in `.scratch/splash-screen/`.

The native console shows a splash with the Orcvs logo as it opens. egui and eframe have no splash API: nothing of the console runs before eframe has created the first window, and eframe sizes that window from its own stored geometry when persistence is on.

## Decision

**The splash is the console's own root window.** `main` opens it borderless, splash-sized and centred. When the splash ends, viewport commands give the same window its decorations and resize it to the console, and the logo fades over the running Grid.

**The console stores and restores its window's geometry itself, and eframe's `persist_window` is off.** The console's save stores the window's content size, frame position, and maximised and fullscreen state under its own key. The splash's end returns the window to that geometry, or to the default size centred when none is stored. While the splash holds, the save stores the geometry the console is going to return to, never the splash window's.

## Rejected alternatives

**A separate splash window over a hidden root window.** eframe would go on restoring the root window, but a root window that starts hidden may get no frames on some platforms, so the console could not end the splash it is hidden behind.

**Keeping eframe's restore and shrinking the window to the splash on the first frame.** eframe applies viewport commands after it makes the first frame visible, so every launch would show the console-sized window for a frame before the splash. A quit during the splash would still store the splash window as the console's geometry.

## Consequences

- Turning `persist_window` back on breaks the splash from the second launch: eframe opens the first window at the stored console size, and the splash has nothing to grow from.
- A geometry is restored in egui points through the monitor the window opens on. eframe restored physical pixels and, on Windows only, pulled a window back onto a connected monitor. The console clamps the size to the monitor it opens on and leaves the position to the platform.
- Eframe's own `window` key goes unwritten. A geometry stored under it before this decision is not read.
- On macOS egui reports the window's maximised state only when the window opens, so a window zoomed after launch is stored at its zoomed size and position rather than as maximised.
- The splash's end resizes the window even when it ends for a close, so a clean close during the hold shows the console-sized window for a frame before it goes.
