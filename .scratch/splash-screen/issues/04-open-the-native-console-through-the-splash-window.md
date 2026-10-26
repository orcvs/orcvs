# 04 — Open the native console through the splash window

Status: ready-for-agent
Blocked by: 02, 03

**What to build:** Launching the native console shows the splash window chosen in
[`../spec.md`](../spec.md), which then becomes the console.

- [ ] Every launch opens as a borderless, centred splash window at the splash size, with the logo,
      `O R C V S` and the version on the presented Theme's window backdrop, including a launch with
      a stored window geometry.
- [ ] The logo is white on a dark appearance and black on a light one.
- [ ] After the hold, or at the first key press or click, the window gains its decorations and
      grows to the console's restored geometry, or the default when none is stored, and the logo
      fades over the running Grid.
- [ ] The key press or click that ends the splash is consumed: it does not edit the Source, move
      the Cursor or change Playback.
- [ ] The hold and the fade advance on egui's input time, not the wall clock, so a test steps them
      deterministically.
- [ ] The Source, the Playback Engine and MIDI start as they do today; nothing waits on the splash.
- [ ] The logo is decoded by eframe's own icon decoder; no dependency is added.
- [ ] Tests through the console's egui test harness cover what the splash draws while held and
      fading, the dismissing input being consumed, and the window commands that end it.
