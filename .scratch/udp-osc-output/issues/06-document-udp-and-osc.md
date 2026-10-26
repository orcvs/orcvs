# 06 — Document `!u` and `!o`

Status: ready-for-agent
Type: task
Blocked by: 03, 04

**What to build:** The Function reference gains `!u` and `!o` with the examples from the
[spec](../spec.md), and the Orca capability map shows them as implemented.

- [ ] `docs/functions.md` entries for both, drawn as Grids as the site's reference is.
- [ ] The in-console Function reference lists both.
- [ ] The scratch docs that still say UDP and OSC wait for a text or message value say they are
      settled by ADR 0085 instead: `.scratch/v1-release/definition-of-done.md` (they stay an
      accepted v1 deferral), `.scratch/sequence-values/spec.md`, `.scratch/source-read-write/spec.md`,
      `.scratch/midi-output-family/spec.md` and `.scratch/v1-roadmap-wayfinding/map.md`.
- [ ] `mise run site_build` and `mise run site_test` pass.
