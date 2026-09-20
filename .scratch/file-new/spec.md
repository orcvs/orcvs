# File > New

Opening a fresh, empty environment from the running console, without restarting it.

## Why

The console can reach a blank default Grid only by restoring nothing. With the `persistence`
feature on, a stored revision is restored whole — its Grid included — which is `source-view/13`'s
recorded decision:

> - [x] A stored Source keeps the Grid it was stored with.

That decision stands, and this effort does not reopen it. Its consequence is that a console which
has ever stored a Source is pinned to the Grid it stored, so a later change to the default Grid
never reaches it. The default moved from 64 by 40 to 128 by 80 in `source-view/13`, and a console
carrying an older stored Source still opens the older Grid. `File > New` is the way out: an
explicit, user-asked reset rather than a silent migration.

A silent migration is the alternative this effort rejects. The Grid is row-major, so growing the
column count reflows every Position; a stored Source cannot be widened without re-laying-out its
content row by row, and doing that unasked would move a viewer's work.

## What resets, and what does not

**The environment resets:** the Source, its Grid, the Cursor, playback, the Source View's Pan and
Zoom, and the running Orcvs's opts — which includes Bpm.

**Settings do not:** Theme, Cursor effects, Source colours, Diagnostics visibility. They are
preferences rather than environment, and `Theme → Reset to theme defaults` already resets the ones
that want it.

Bpm resetting is a consequence of replacing the whole Orcvs, which is how `Load Function reference`
already behaves. It is recorded as a criterion rather than left implicit so that preserving it
across a New is a visible change to make later, not a surprise to discover.

## Vocabulary

- **Open** — replace the running Orcvs's Source, Grid included, and rest every view derived from it.
  Both `File > New` and `File > Load Function reference` are Opens.
- **Environment** — what an Open replaces. Distinguished from the settings an Open leaves alone.

## Out of scope

Migrating a stored Source to a new default Grid; changing `source-view/13`'s decision; any Save,
Save As, or named-document model; and whether the File menu should offer more than these two Opens.
