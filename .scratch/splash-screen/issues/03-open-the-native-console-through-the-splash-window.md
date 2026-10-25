# 03 — Open the native console through the splash window

Status: ready-for-agent
Blocked by: 01, 02

**What to build:** Launching the native console shows the splash window chosen in
[`../spec.md`](../spec.md), which then becomes the console.

- [ ] The application opens as a borderless, centred splash window with the logo, `O R C V S` and
      the version, on the presented Theme's window backdrop.
- [ ] The logo is white on a dark appearance and black on a light one.
- [ ] After the hold, or at the first key press or click, the window gains its decorations and
      grows to the console, and the logo fades over the running Grid.
- [ ] With persistence on, the console ends at the stored window size and position, and the splash
      does not overwrite them.
- [ ] The Source, the Playback Engine and MIDI start as they do today; nothing waits on the splash.
- [ ] The logo is decoded by eframe's own icon decoder; no dependency is added.
- [ ] The splash is covered by tests through the console's egui test harness: what it draws while
      held, and that input ends it.
- [ ] An ADR records that the native console opens through a splash window, and why.
