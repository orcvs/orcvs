# 07 — Verify direct pattern editing in the console

Status: needs-triage
Blocked by: 06

## Work

Use the real console input and file paths to load the tracker Source, edit a
step, clear it to a rest, and restore it. Reuse the existing Grid, Cursor,
Region, Source Paint and diagnostic presentation. A dedicated tracker editor
or moving playhead is not required for acceptance.

## Acceptance

- [ ] Typing/pasting a Note changes that step without duplicating endpoints or
      reconstructing a nested expression. A two-character edit may pass through
      an invalid intermediate Source and recover without a spurious note.
- [ ] Clearing a complete step makes the agreed rest and leaves later positions
      unchanged. Copy/paste preserves the region's blank steps.
- [ ] The Language Map supplies Note/data classification and diagnostics to the
      existing paint path. UI code never independently decodes pattern characters.
- [ ] While Playback runs, an accepted edit is heard on the next eligible
      Snapshot; a Tick planned against an old revision cannot overwrite the edit.
- [ ] Save/reopen preserves every note and rest position. Playback restart uses
      the current saved or edited Source and begins its clock at Tick 0.
- [ ] A focused egui test drives the actual input path and asserts the resulting
      Source and observable Playback behavior without sleep-based timing.

Apply the egui and Rust-change skills. Cross-platform/browser coverage belongs
to CI unless the actual changes trigger a local platform gate in `AGENTS.md`.
