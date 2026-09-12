# 12 — Finish the deletion ticket 01 started

**What to build:** `MidiOutputAdapter::destinations()` (`orcvs/src/midi.rs:140`, body at `:141`) is
`pub` with no production caller. Its only callers are `orcvs/src/midi.rs:520` and
`orcvs/src/native_midi.rs:282`, both inside `mod tests` (at `:315` and `:186` respectively). It is
exactly the dead public surface ticket `01` existed to remove.

**The attribution matters, and an earlier report got it backwards.** This method was never deleted
and never recreated. It exists unchanged at every commit on this branch and before it —
`25c8de8:orcvs/src/midi.rs:95` is the same four lines as `HEAD:orcvs/src/midi.rs:140`. What ticket
`01` (`f65209a`) deleted was `PlaybackEngine::midi_destinations`, whose body was
`lock_recover(&self.inner).adapter.destinations()`. That deletion removed the only production caller
and orphaned this method while leaving it `pub`. So: `01` orphaned it, `04` did not recreate it.

`01` checked `select_midi_destination` for the same shape and deliberately left it, because twelve
uses in `orcvs/src/midi.rs`'s own tests make it a live test seam rather than dead surface. This one
has two, and neither crosses a crate boundary, so narrowing its visibility costs those tests nothing.

**Status:** resolved

- [ ] `MidiOutputAdapter::destinations` is no longer `pub`, or is gone.
- [ ] The two test call sites still compile and still assert the same thing.
- [ ] `MidiOutputAdapter::published_destinations` and `refresh_destinations` — the path production
      actually uses — are untouched.
- [ ] Whatever `01`'s reasoning was for leaving `select_midi_destination` still holds; it is not in
      scope here.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D
warnings` and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `orcvs` and
`console`. The public API shrinks, so `cargo test --workspace --doc --locked` too.

## Comments

Filed from the `playback-actor` review ledger as **CR-13**, minor.

The original report said ticket `04` recreated a method ticket `01` had deleted. That is wrong in
both halves and the issue above says what actually happened, because the two framings lead to
different work: a recreated method is a regression to revert, an orphaned one is a deletion to finish.

### Resolved

`MidiOutputAdapter::destinations` is deleted rather than narrowed. Both callers
were in test modules and both now go through the path production uses —
`refresh_destinations()` followed by a read of `published_destinations()` —
which asserts more than the deleted method did, since it covers the publish as
well as the discovery.

`published_destinations` and `refresh_destinations` are untouched, and
`select_midi_destination` is out of scope here: it was deleted separately, as
part of the review pass, for the reason `01` left it standing no longer holding
— its twelve test call sites now route through `MidiSelectionHandle`.
