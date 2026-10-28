# The Panel plays and rewinds

Status: accepted. Amends [ADR 0076](0076-stopping-playback-holds-its-tick.md), under which the web
had no Rewind, and [ADR 0087](0087-the-site-plays-the-console-through-a-built-in-synth.md)'s
clause that the audio context starts on the message that follows the reader's gesture. Settled in
`.scratch/play-control/`.

Space was the only way to play. A phone has no Space key, so the web console could not play on a
phone at all, and the web had no Rewind because the browser reserves Command Shift R.

## Decision

**The Panel opens with two Cells, `P` then `R`, before **B**.** Each is one Source Cell, with one
Cell between them, on the window's Cell columns, so the Panel continues the Grid and each is as easy
to tap as a Cell is to type into. Both are drawn in the Function role, and `P` in the Bang role
while Playback plays. `P` does what Space does, and `R` does what Command Shift R does on native;
Space and the chord are unchanged. Every console shows the Panel, the small console included
(ADR 0086), so an embedded example plays without a key.

**A gesture resumes the Built-in Synth's audio context itself.** The page resumes its context on
each pointer, touch or key gesture, on the page or in a console's frame, while the gesture is
still running. Safari resumes a context only inside a gesture, and the console's first
`orcvs.midi` arrives after the gesture that caused it. The page also asks for media playback, so
iOS's ring/silent switch does not mute it. Resuming on the first message stays as the fallback.

## Considered options

The prototypes are on the branches `prototype/play-control` and `prototype/play-control-cells`.

- **`⏮` and `▶`/`⏸` as egui buttons.** Too small to tap on a phone, and unlike anything else on the
  Grid.
- **Cells twice the Source Cell.** A Cell that is too small to tap would be too small to type into.
- **Glyphs at the right of the top bar.** At phone width they crowd the menus, and the small
  console has no top bar.
- **A Play/Stop word before File.** The small console has no top bar.
- **A Playback menu.** It takes two taps, and the small console has no top bar.
