# 03 — Confirm before New discards written content

**What to build:** Choosing `File > New` while the Source holds written content asks before
replacing it. Cancelling leaves the environment exactly as it was. New on an already-empty Source
asks nothing and opens straight away.

**Blocked by:** 02 — File > New opens an empty default Grid.

**Status:** ready-for-agent

**Tags:** release/v1

New is irreversible and it destroys work: there is no undo across an Open, and with `persistence`
on the next ordinary save overwrites the stored revision the discarded Source came from. It is one
menu click away from `Load Function reference`, which is equally destructive and equally unasked —
so whatever this builds should be able to cover both, even if only New uses it here.

The console has no confirmation or modal anywhere today. This introduces the first one, which is
why it is its own ticket rather than a line inside `02`: the pattern it sets will be reached for
again.

- [ ] New on a Source holding written content asks before opening.
- [ ] Cancelling leaves the Source, Grid, Cursor, playback state and Source View untouched, and
      stores nothing.
- [ ] Confirming opens the empty default Grid exactly as `02` specifies.
- [ ] New on an empty Source does not ask.
- [ ] The confirmation is keyboard-reachable and dismissable, and Escape cancels rather than
      confirms.
- [ ] While it is open, Source keys do not reach the Grid behind it — the console already keeps
      keys from the Source while a popup is open, and this follows that rule rather than inventing
      a second one.
- [ ] The pattern is general enough that `Load Function reference` could adopt it without being
      rewritten. Whether it does is out of scope here.
- [ ] Tests cover: asked and confirmed, asked and cancelled, and not asked on an empty Source.
- [ ] The scoped gates for `console` pass, and the egui skill's guidance is followed for any new
      presentation.

## Comments

Deliberately separable. If New is wanted before this lands, `02` ships an unconfirmed, destructive
New and this follows; the acceptance criteria above do not depend on that ordering.
