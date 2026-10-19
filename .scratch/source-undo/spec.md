# Source undo: Command Z takes back an edit, Command Shift Z puts it back

Status: needs-triage

## Problem Statement

The console has no Undo. A key that overwrites the wrong Cell, a Fill over the
wrong Selection, or a Cut and Paste in the wrong place can only be repaired by
retyping. Orca binds Undo and Redo to Command Z and Command Shift Z and keeps a
history of whole Source strings. The tracker example is edited
live while it plays, which is exactly when a slip costs most.

Toby asked for it on 2026-10-19 while reviewing the tracker examples against
Orca.

## What an edit is

A user edit is any input that writes Source Cells: a typed character, Delete
and Backspace, Fill, Cut and Paste. Each one is a single undoable step, however
many Cells it writes. Cursor and Selection moves are not edits.

Tick writes are not user edits. Functions write Source every Tick through their
Output Portals, so an Undo history that recorded them would undo the music
rather than the edit.

## Open questions

1. **What does an Undo restore while Playback runs?** The Cells an edit wrote
   may have been written again by Tick writes since. Restoring the whole Source
   from before the edit, as Orca's whole-Source history does, discards those
   Tick writes; restoring only the edited Cells keeps them.
   The default proposed here is per-Cell: an Undo writes back the prior content
   of the Cells the edit wrote, and nothing else.
2. **Are consecutive typed characters one step or several?** The default
   proposed here is one step per edit, so each typed character is its own
   step.
3. **Does the history survive an Open, a New or a Save?** `.scratch/file-new`
   notes that nothing survives an Open today. The default proposed here clears
   the history on Open and New and keeps it across Save.
4. **How deep is the history?** Each step holds only the Cells it wrote, so a
   fixed bound such as a few hundred steps costs little.
5. **Does an Undo restore the Cursor and Selection?** The default proposed
   here puts the Cursor back where the edit began, so the Undo is visible.
6. **Web chords.** Command Z is not reserved by the browser, so the web binds
   it too, unlike the Rewind chord.

## Relation to Tick navigation

`../tick-navigation/spec.md` replays Ticks from a base Source. Whichever lands
second has to decide whether an Undo during a stopped, navigated Playback
changes the base it replays from. This effort does not depend on that one.
