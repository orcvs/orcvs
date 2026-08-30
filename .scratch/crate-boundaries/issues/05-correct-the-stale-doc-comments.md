# 05 — Correct the stale doc comments

**What to build:** Fix two comments that describe code that no longer exists. `console.rs` documents
a type called "ConsoleApp"; the type is `Console`. `app.rs` documents "The console's editing state";
the type also owns playback, playback state, and MIDI device selection.

**Status:** ready-for-agent

- [ ] The `console.rs` comment names the type that is actually there.
- [ ] The `app.rs` comment lists what the type owns, or is replaced by issue 04's comment.
- [ ] The doctest embedded in the `app.rs` comment still passes.
- [ ] No other comment in either crate names a type that does not exist.

## Comments

This is independent of the crate move and can land first. It is small, but both comments actively
mislead a reader about the seam this effort depends on.

Issue 04 rewrites the `app.rs` comment as part of its own work. If 04 lands first, this issue covers
`console.rs` alone.
