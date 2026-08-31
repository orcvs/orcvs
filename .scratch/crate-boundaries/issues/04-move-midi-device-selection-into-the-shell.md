# 04 — Move MIDI device selection into the shell

**What to build:** Move `refresh_midi_destinations`, `midi_destinations`,
`select_midi_destination`, `selected_midi_destination_id`, `midi_status`, and the fields they own
out of `Orcvs` and into a type in `shell`. `Orcvs` keeps editing and playback.

**Blocked by:** 03 — Name the running instance `Orcvs`.

**Status:** resolved

- [x] `Orcvs` owns `opts`, `cursor`, `grid`, `source`, `playback`, and `playback_state`, and nothing else.
- [x] Device discovery, the destination list, and the status string live in `shell`.
- [x] `Orcvs` still receives an output adapter and does not know how it was chosen.
- [x] The platform `cfg` attributes move with the code they guard.
- [x] The doc comment on `Orcvs` describes what it now owns.
- [x] Device selection behaviour is unchanged for the user.

## Comments

The type currently holds three groups of state: editing, playback, and device selection. Its own doc
comment calls it "The console's editing state", which describes only the first group. The comment and
the fields disagree, and that disagreement is the reason to split.

Editing and playback stay together because CONTEXT.md defines Live Editing as exactly that pair:
changing the Source while Playback continues.

Device selection leaves because the choice of a MIDI port is user configuration. It is neither
composition nor performance, and it has no place in a running Orcvs.

Resolved on `feat/extract-orcvs-crate`. TDD covers discovery, selection identity, backend error
status, and playback-start errors reaching the shell-owned status. Two-axis review confirmed the
boundary and found a lost start-error status plus stale `app` identifiers; both were regression-
tested where behavioural and fixed before commit. Native package gates and the WASM check pass.
