# 05 — Verify live editing in the console

Status: ready-for-agent
Blocked by: 04

**What to build:**

Use ordinary console input and file workflows to load a tracker, edit an
Item, clear it to a rest and restore it while Playback runs.

## Acceptance criteria

- [ ] Typing or pasting a Note changes the next eligible Tick; a malformed intermediate edit recovers without spurious notes and with bounded useful diagnostics.
- [ ] Clearing an Item preserves later Item positions; copy and paste retain blank Items.
- [ ] Source Paint identifies the List as Track data and presents a malformed selected Item’s diagnostic where it is consumed.
- [ ] Open, Save, reopen and a new Playback run reproduce the written pattern and reset the Clock according to the Playback contract.
- [ ] Tests drive real console input and existing file workflows under the egui skill; no dedicated tracker editor or extra adapter is introduced.
