# A finger Pans the Source and asks for the keyboard

Status: accepted. Adds touch to the Pan gestures of [ADR 0045](0045-the-source-view-is-a-bounded-space.md),
and amends [ADR 0091](0091-the-web-console-reads-its-settings-from-its-page.md)'s phone Cell from 24 by 36
points to 16 by 24.

On a phone the web console showed a selected Cell but could not be edited or moved around. No
keyboard appeared, every Pan gesture needed a mouse or a key (middle-drag, Alt with a drag, the
wheel), and a finger's drag selected a Region. egui was not the limit: eframe 0.37 opens a phone's
keyboard through a hidden text input whenever a frame reports where text is edited
(`PlatformOutput::ime`), and it reports touches beside the primary pointer. The console used
neither, because the Source Grid is painted rather than a `TextEdit`.

## Decision

**A finger's drag Pans.** A drag a finger starts Pans the Source by what the finger moved, under
the same clamp as every Pan, and leaves the Cursor and the Region. A tap selects the Cell under it,
as a click does. A finger held still for egui's long press (`InputOptions::max_click_duration`,
0.8 seconds) selects a Region from the Cell it pressed to the Cell nearest the finger, for as long
as it stays down. A mouse and a pen keep the gestures ADR 0045 names.

**A finger's press asks for the keyboard.** While the latest primary press was a finger's and the
Source holds the keys, each frame reports the Cursor's Cell as where text is edited, so eframe
focuses its text input and the phone opens its keyboard. A mouse press withdraws the request, so a
desktop keeps the canvas holding the keys and its key events unchanged. A control that holds the
keys reports its own position, or none.

**A phone keyboard's text and IME events are Source input.** Each character of a committed text
is one Cell's `Text`, and a `DeleteSurrounding`, which is how eframe reports Backspace under
Android's Gboard, is that many Backspaces. A composition still in progress writes nothing. The
console turns off autocorrect, autocomplete and spellcheck on eframe's text input, so a keyboard
does not rewrite Cells already written, and the page's viewport asks the browser to shrink the page
for the keyboard (`interactive-widget=resizes-content`), so the Cursor follow keeps the Cursor above
it where the browser honours that.

**A phone starts at 16 by 24 point Cells.** 24 by 36 showed about 16 columns on a phone; 16 by 24
shows about 24 and keeps a Cell a finger can tap.

## Considered options

- **Two fingers Pan, one finger keeps drag-to-select.** Closer to desktop, but a phone's viewer
  tries one finger first and would select a Region by accident.
- **Ask for the keyboard on every platform.** A desktop's keys would then pass through eframe's text
  input rather than the canvas, changing how every key reaches the Source for no gain.

## Consequences

- Safari on iOS ignores `interactive-widget`, so there the keyboard can still cover the lower
  Cells. Sizing the canvas to `visualViewport` would answer it and is not done here.
- A phone keyboard has no arrows, Tab or Escape. Tapping a Cell moves the Cursor; an on-screen key
  row is not part of this decision.
- Android's Gboard reports Space as text, which writes nothing, so Space does not toggle Playback
  there; the Panel's `P` does.
