# 01: Decide how Undo treats Tick writes, typing and focus

**What to build:** An ADR that settles the open questions in `../spec.md`:
what an Undo restores when a Tick write has rewritten an edit's Cells, how
typed characters group into steps, what clears the history, its depth, where
the Cursor lands, which control a focused chord reaches, and whether any menu
lists the commands.

**Blocked by:** None (can start immediately)

**Status:** needs-triage

- [ ] Each open question is answered, adopting or replacing its proposed
      default.
- [ ] The ADR is written and the spec's open questions point to it.

## Comments
