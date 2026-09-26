---
name: egui
description: egui in this repository. Use for the console's egui and eframe presentation, live inspection, and egui or eframe dependency or feature changes.
---

# egui

The console is the only crate that sees `egui`. ADR 0022 and ADR 0042 hold that
boundary: a fact about a Source belongs to `orcvs`, a decision about how this
console shows one belongs to `console`, and freedom from the toolkit is
necessary for a module to live in `orcvs` but not sufficient.

`references/guide.md` holds the resolved versions, the reference hierarchy, the
inspection and `egui_mcp` setup, the testing patterns, and the troubleshooting.
Read it before citing an upstream API, before `mise run inspect`, before writing
a kittest, and before attaching `egui-mcp`.

1. Read `AGENTS.md`, `console/Cargo.toml`, and the console modules the change
   reaches. Confirm the resolved versions with
   `cargo tree --package console --locked -i egui` before citing an API.
2. Find the matching upstream API for the pinned release and the closest
   upstream example. Prefer an existing egui primitive over a new one.
3. Make the smallest coherent change. Keep domain state in `orcvs` and view
   transforms, style, and toolkit concerns in `console`. Treat a UI function as
   repeatable: it may run many times per frame and must start no domain action
   or external effect merely by executing.
4. Add focused regression coverage. Prefer a behavioural or geometry assertion
   over a screenshot. Drive the shipped `Console` rather than a test-only UI —
   `console::kittest_tests` is the harness, `console::tests` the `Shape`-level
   assertions beside it.
5. Inspect the running console where the question is about live behaviour:
   `mise run inspect`, then attach through `egui-mcp`. Convert anything
   repeatable into a test; an MCP session proves nothing on its own.
6. Run the console gate from `AGENTS.md`, plus `mise run check_inspection` when
   the `inspection` feature or `eframe` is touched, `mise run check_wasm` for
   platform or rendering work, and `mise run audit_deps` for a manifest or
   lockfile change.
7. Review the complete diff. Report the commands run, the checks not run and
   why, and any interaction that was inspected rather than tested.

## Hold these

- Domain behaviour and Source state stay in `orcvs`; the UI reads a snapshot.
  Do not move domain behaviour into a widget or mint a second authoritative
  copy of Source state for the UI.
- Follow the snapshot and message-passing the console already uses. No
  blocking I/O, blocking receive, `block_on`, or repeated expensive work in
  the UI path; do not hold a mutex guard across a widget closure.
- Widget identity is stable and derived from domain identity, never from a list
  position or a mutable display string.
- Reuse the existing style, resolved `Theme`, font, `GlyphTable`, and paint
  infrastructure; do not rebuild a reusable rendering resource every frame.
- Respect clipping, points against physical pixels, the owned transform,
  hit-testing, focus ownership, and input consumption. Rendering and
  hit-testing stay consistent under resize and pan or zoom.
- Preserve the established repaint, animation, and reduced-motion behaviour.
- Cells stay painted; AccessKit nodes belong to real controls. Do not mint
  per-Cell widgets solely for automation, and do not add a domain-control API
  because the canvas is painted.
- ADR 0040 and the atlas budget in `console::glyphs` are the standing decisions.
  General performance advice is not a licence for an unrequested optimisation.
