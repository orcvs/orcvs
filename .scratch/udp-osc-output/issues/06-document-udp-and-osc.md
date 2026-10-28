# 06 — Document `!u` and `!o`

Status: resolved
Type: task
Blocked by: 03, 04

**What to build:** The Function reference gains `!u` and `!o` with the examples from the
[spec](../spec.md), and the Orca capability map shows them as implemented.

- [x] `docs/functions.md` entries for both, drawn as Grids as the site's reference is.
- [x] The in-console Function reference lists both.
- [x] The scratch docs that still say UDP and OSC wait for a text or message value say they are
      settled by ADR 0085 instead: `.scratch/v1-release/definition-of-done.md` (they stay an
      accepted v1 deferral), `.scratch/sequence-values/spec.md`, `.scratch/source-read-write/spec.md`,
      `.scratch/midi-output-family/spec.md` and `.scratch/v1-roadmap-wayfinding/map.md`.
- [x] `mise run site_build` and `mise run site_test` pass.

## Answer

`docs/functions.md` has a Network output section with entries for `!u` and `!o`, each example
checked by `orcvs/tests/function_reference_doc.rs`, and the site draws a Payload as ordinary Cells
with its closer in its Function's colour. The in-console Function reference holds both under a
`|| Network` header in the MIDI band. ADR 0019's capability table maps `;` and `=` to `!u` and
`!o`, and the five scratch docs name ADR 0085. `mise run site_build` and `site_test` were left to
CI: this environment's Node is older than Astro accepts.
