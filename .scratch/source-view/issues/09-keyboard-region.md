# 09: The Region from the keyboard

**What to build:** Shift with an arrow moves the Cursor and keeps the anchor; a bare arrow collapses the Region and moves; command `A` spans the whole Grid; Escape collapses the Region onto the Cursor. Command is ⌘ on macOS and Ctrl elsewhere. See ADR 0046.

**Blocked by:** 07

**Status:** ready-for-agent

- [ ] Shift with an arrow extends the Region, and passing the anchor flips it.
- [ ] A bare arrow collapses the Region and moves the Cursor.
- [ ] Command `A` spans the whole Grid.
- [ ] Escape collapses the Region onto the Cursor, and with the BPM field focused Escape reverts the BPM and leaves the Region as it was.
- [ ] None of these chords writes to the Source.
