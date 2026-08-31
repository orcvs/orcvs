# 05 — Correct the stale doc comments

**What to build:** Fix two comments that describe code that no longer exists. `console.rs` documents
a type called "ConsoleApp"; the type is `Console`. `app.rs` documents "The console's editing state";
the type also owns playback, playback state, and MIDI device selection.

**Status:** resolved

- [x] The `console.rs` comment names the type that is actually there.
- [x] The `app.rs` comment lists what the type owns, or is replaced by issue 04's comment.
- [x] The doctest embedded in the `app.rs` comment still passes.
- [x] No other comment in either crate names a type that does not exist.

## Comments

This is independent of the crate move and can land first. It is small, but both comments actively
mislead a reader about the seam this effort depends on.

Issue 04 rewrites the `app.rs` comment as part of its own work. If 04 lands first, this issue covers
`console.rs` alone.

Resolved by the comments landed with issues 03–04. `Console` now documents that it wraps the
running Orcvs with egui presentation concerns, and `Orcvs` documents its options, Source and Grid,
Cursor, and Playback lifecycle while assigning output-device discovery and selection to the shell.
The embedded doctest passes, and stale-name searches find neither `ConsoleApp` nor the former
"console's editing state" wording.
