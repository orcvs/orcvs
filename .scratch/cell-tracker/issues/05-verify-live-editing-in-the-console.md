# 05 — Verify direct pattern editing in the console

Status: needs-triage
Blocked by: 04

## Work

Use the real console input and file paths to load the tracker Source, edit an
item, clear it to a rest, and restore it. Reuse the existing Grid, Cursor,
Region, Source Paint and diagnostic presentation. A dedicated tracker editor or
moving playhead is not required.

## Acceptance

- [ ] Typing or pasting a Note changes that item. A two-character edit may pass
      through an invalid intermediate Source and recover without a spurious note.
- [ ] Clearing an item makes a rest and leaves later items where they were.
      Copy and paste preserve blank items.
- [ ] Source Paint shows the List as Track's data and shows a malformed item's
      diagnostic where it is played.
- [ ] Open, Save, reopen and a new Playback run reproduce the pattern and reset
      the clock as the Playback contract specifies.
- [ ] Console tests drive real input; use the egui skill.
