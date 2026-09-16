# 04 — The menu stops guessing whether its own request was answered

**What to build:** The MIDI menu stops tracking whether the answer it is looking at belongs to the
refresh it asked for, because the answer now arrives as a return value and there is nothing left to
wait for. The console stops repainting on a timer to catch a reply, a refused port stops reading as
success, and the message saying the running Orcvs is gone reaches the status line even when an
older message is still showing.

**Blocked by:** 03.

**Status:** resolved

- [x] The console holds no state whose purpose is deciding whether a refresh has been answered, and
      no repaint driven by an outstanding request.
- [x] Selecting a destination reports success only when the port opened. A refusal leaves a visible
      message rather than clearing the status line.
- [x] Unavailability of the running Orcvs reaches the status line regardless of what is already
      showing, and cannot be permanently suppressed by an unrelated earlier message.
- [x] A message the engine reported and a message a console action produced have a stated
      precedence; neither silently overwrites the other on every frame.
- [x] A persistent discovery failure is still reported without restating itself on every frame the
      menu is open.
- [x] Menu tests cover: a list rendered from a successful discovery, a failure rendered in place of
      the rows, a refused port opening, and an unavailable running Orcvs reported over a stale
      message. The console's existing backend fake drives all four.
