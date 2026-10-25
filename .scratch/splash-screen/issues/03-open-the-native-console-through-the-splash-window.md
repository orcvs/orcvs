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
- [ ] The key press or click that ends the splash is consumed: it does not edit the Source, move
      the Cursor or change Playback.
- [ ] The hold and the fade advance on egui's input time, not the wall clock, so a test steps them
      deterministically.
- [ ] Every launch opens at the splash size, including one with a stored window geometry. The
      console then ends at the stored size and position, or the default when none is stored.
- [ ] Quitting during the hold stores the console's geometry, never the splash window's.
- [ ] A close request during the hold ends the splash first, so the console's discard confirmation
      for an unsaved Source shows in a decorated window.
- [ ] A failure to start the console is reported as it is today. No borderless window is left
      without a way to close it.
- [ ] The Source, the Playback Engine and MIDI start as they do today; nothing waits on the splash.
- [ ] The logo is decoded by eframe's own icon decoder; no dependency is added.
- [ ] Tests through the console's egui test harness cover what the splash draws while held, the
      dismissing input being consumed, relaunch with a stored geometry, and quit and close
      requests during the hold.
- [ ] An ADR records that the native console opens through a splash window, and why.
