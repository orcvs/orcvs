# 07 — Retire `cell.selection.border.width`

**What to build:** Since 04, the Cursor's outline is its Cell's edges at `cursor.border.width`, and no Source Cell reads `cell.selection.border.width`, so the Theme key is accepted and does nothing. Decide whether to remove it from the schema, the built-in Themes and the Theme document parser (an unknown key is an error, so a custom Theme that sets it would stop loading), or keep it accepted for that reason. If it stays accepted, a Theme that sets it should at least get a diagnostic saying it is retired, since a Theme that relied on it to keep a border on the Cursor while `cursor.border.width` is zero now marks the Cursor by its fill alone. `RenderFrame::cursor_visible` likewise has no reader in the console any more.

**Blocked by:** 04 — The selection outline owns its edges.

**Status:** needs-triage
