# 02 — Play and rewind from the Panel

Status: ready-for-human
Type: task
Blocked by: 01

**What to build:**

The decision in ADR 0089: `⏮` and `▶`/`⏸` at the start of the Panel, doing what Command Shift R
and Space do, and a Built-in Synth page that resumes its audio context inside the reader's gesture.

**Acceptance criteria:**

- [x] The Play control plays and stops Playback as Space does, and a Space after it toggles once.
- [x] The Rewind control makes Tick `0` the next Tick of a paused Playback.
- [x] The small console shows both controls.
- [x] Their glyphs are in the console's font.
- [x] A tap in the site's console frame resumes the page's audio before any note.
- [ ] Sound on a real iPhone and a real Android phone, checked by the user.
