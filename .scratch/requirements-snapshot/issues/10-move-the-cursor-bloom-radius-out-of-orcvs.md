# 10 — Move the Cursor bloom radius out of `orcvs`

**Status:** needs-triage

**What to fix:** CONS-011 (ADR 0042) puts every decision about how the console shows a Source in
the console. `orcvs` still carries `CursorBloomRadius` (`orcvs/src/opts.rs`) and hands it to the
Render Frame as `cursor_bloom_radius`, and the console draws no bloom that reads it.
`docs/requirements/console.md` lists this under "Accepted but not built".

**Done when:** `orcvs` carries no Cursor bloom radius: the console owns it, or nothing does because
nothing draws it.
