# 01 — Delete the unreferenced destination queries

**What to build:** `PlaybackEngine::midi_destinations` and `PlaybackEngine::selected_midi_destination_id` have no callers anywhere in the workspace — not in `console/`, not in `orcvs/`, not in tests or benches. The console reaches MIDI only through `MidiSelectionHandle`, which has its own `destinations` and `selected_destination_id`. Delete both.

This lands first and on its own so that a deletion is not buried inside a rewrite. Every method left on `PlaybackEngine` after this is one the rest of the effort has to carry across the seam, and two of them do not need carrying.

Check `PlaybackEngine::select_midi_destination` while you are there: it has no `console/` callers either, but twelve uses in `orcvs/src/midi.rs`'s own tests. That is a live test seam, not dead surface — leave it, and say in the ticket that you checked.

**Status:** resolved

- [x] `midi_destinations` and `selected_midi_destination_id` are gone from `PlaybackEngine`.
- [x] `MidiSelectionHandle`'s equivalents are untouched and the console behaves identically.
- [x] `select_midi_destination` is left in place, with the reason recorded.
- [x] No test changes. Nothing referenced them.

## Verification

`cargo fmt --all -- --check`, then `cargo clippy --package <crate> --all-targets --locked -- -D warnings` and `PROPTEST_CASES=32 cargo nextest run --package <crate> --locked` for `orcvs` and `console`. Public API shrinks, so `cargo test --workspace --doc --locked` too.

## Comments

A workspace-wide search (every tracked file type, `console/` and `.scratch/` included) found
`midi_destinations` and `selected_midi_destination_id` only at their own definitions in
`orcvs/src/playback.rs`. Both are deleted. `MidiSelectionHandle::destinations` and
`::selected_destination_id` — the path the console actually uses — are untouched, and they keep
`MidiOutputAdapter::destinations`/`selected_destination_id` live, so nothing underneath became
dead code.

`select_midi_destination` was checked and stays: it has no `console/` callers, but twelve uses in
`orcvs/src/midi.rs`'s own tests, which is a live test seam rather than dead surface.

No test changed, as predicted.
