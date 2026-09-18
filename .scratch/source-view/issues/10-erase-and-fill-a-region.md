# 10: Erase and fill a Region

**What to build:** Backspace and Delete empty every Cell of a Region larger than one Cell and keep the Region; on one Cell they empty it and step left as today. Typing writes at the Cursor, steps right, and collapses the Region. Command Enter followed by a character fills every Cell of the Region with it. See ADR 0046.

**Blocked by:** 09

**Status:** ready-for-agent

- [ ] Backspace and Delete empty every Cell of a Region larger than one Cell and keep the Region.
- [ ] On a Region of one Cell, Backspace and Delete behave as they do today.
- [ ] Typing writes at the Cursor, steps right, and collapses the Region.
- [ ] Command Enter fills every Cell of the Region with the character, and a plain keystroke never fills.
