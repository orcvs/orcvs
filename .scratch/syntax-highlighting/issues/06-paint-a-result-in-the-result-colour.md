# 06 — Paint a Result in the Result colour

**What to build:** Cells the decision in `05` names as a Result draw their glyph in the Result colour on a Result-colour tint, so a written value stands apart from the Token-tinted Expression that produced it.

**Blocked by:** 01 — Expose Source colours as Theme settings; 05 — Decide how a Result is known before a Tick. (Uses the Fill tint from 02.)

**Status:** needs-triage

- [ ] A scalar Result (`07` south of `.+0304`, `C4` south of `.^3C`) draws in the Result colour on a Result tint at the Fill tint strength.
- [ ] A Sequence Result (`04030201` south of `:<:-0104`) is painted the same way across all its Cells.
- [ ] A Bang Result (`**` south of `~*0401`) keeps the Bang glyph colour on the Result tint.
- [ ] A Terminal Output Function and an Absence Marker paint nothing south of the root.
- [ ] Focused paint tests cover each case.

## Comments

Status stays `needs-triage` until `05` is decided, since the acceptance above assumes position-based Result Cells and `05` may change that.
