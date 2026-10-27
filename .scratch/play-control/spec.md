# An explicit Play control

Status: needs-triage

## The question

Space plays and stops Playback, and nothing else does. A phone has no Space key, so the web console
cannot play on a phone at all. The web build is where phones arrive (the site is the console), and a
browser starts audio only after a gesture, so the control must be something the reader taps.

Where does the control sit, what does it show, and does it carry anything beside play and stop?

## What is not in question

- What a press does is what Space does: it plays a stopped Playback and stops a playing one, and a
  stop holds its Tick (ADR 0076). Orcvs has no separate paused state.
- Playback starts only from the reader's action. A Linked or small console opens stopped (ADR 0086).
- The Built-in Synth resumes its audio context on the first message after the reader's gesture
  (ADR 0087). A tap is a gesture, as a key press is.
- Space keeps playing and stopping.

## What is open

1. **Placement.** The bottom Panel beside the transport Readouts, the top bar, or a menu.
2. **Glyphs.** Play `▶` with stop `■` or pause `⏸`, in Monaspace Neon (all present in the font).
3. **Rewind.** The web binds no Rewind (ADR 0076: the browser reserves Command Shift R) and the
   console has no Playback menu, so a phone and a browser cannot rewind. A Rewind control beside
   Play would close that gap.
4. **The small console.** It shows the bottom Panel but no menu bar (ADR 0086), so placement decides
   whether embedded examples on Learn and Reference get the control.

## Prototype

`issues/01-prototype-the-play-control.md`.
