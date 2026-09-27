# 22 — Keep an earlier refused Source when a later start also refuses

**What to build:** A stored value that fails to decode is set aside so the viewer can recover it, and a second refusing start does not overwrite what the first set aside. Today saving the refused payload writes its key unconditionally, so two refusing starts in a row lose the first payload.

**Blocked by:** None — can start immediately.

**Status:** resolved

- [x] A second refusal does not replace an existing set-aside payload (it is kept, or both are kept under distinct keys).
- [x] A test performs two refusing starts in a row and recovers the first payload.
- [x] The feature-off build still compiles without persistence.

## Comments

**2026-09-25 — audited against `origin/main` `199c3331`.** Still valid; no change needed.

**2026-09-27 — resolved in [#167](https://github.com/orcvs/orcvs/pull/167).** Both payloads are kept under distinct keys. A refused start picks the first free key — `REFUSED_KEY`, then `orcvs_source_refused_2`, `_3`, and on — and the save writes the payload there, so a second refusal no longer overwrites the first. The start report and the menu-bar notice name the key the payload is actually kept under (`Persistence::notice_key`, which replaces `notice_visible`). Keeping the existing payload and dropping the new one was rejected because the second payload would be lost and the notice would name a key that does not hold it. `a_second_refusing_start_keeps_the_payload_the_first_set_aside` walks the loss sequence — first refusal and save, another invalid primary payload, second refusal and save — and recovers the first payload from `REFUSED_KEY` and the second from the key its notice names. `cargo nextest run --workspace --tests --no-default-features` passes.
