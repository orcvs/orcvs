# 22 — Keep an earlier refused Source when a later start also refuses

**What to build:** A stored value that fails to decode is set aside so the viewer can recover it, and a second refusing start does not overwrite what the first set aside. Today saving the refused payload writes its key unconditionally, so two refusing starts in a row lose the first payload.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] A second refusal does not replace an existing set-aside payload (it is kept, or both are kept under distinct keys).
- [ ] A test performs two refusing starts in a row and recovers the first payload.
- [ ] The feature-off build still compiles without persistence.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid; no change needed.
