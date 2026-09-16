---
name: egui
description: Implement or review the console's egui and eframe presentation in this repository. Use for widgets, custom painting, layout, input, focus, the Source view's pan, zoom and presentation behaviour, egui_kittest UI regressions, live inspection through egui_mcp, and egui or eframe dependency and feature changes. Combine with rust-change for implementation, rust-dependency-change for the manifests and lockfile, and rust-review for independent review.
---

# egui

The console is the only crate that sees `egui`. ADR 0022 and ADR 0042 hold that
boundary: a fact about a Source belongs to `orcvs`, a decision about how this
console shows one belongs to `console`, and freedom from the toolkit is
necessary for a module to live in `orcvs` but not sufficient.

`references/guide.md` holds the resolved versions, the reference hierarchy, the
inspection and `egui_mcp` setup, the testing patterns, and the troubleshooting.
Read it before deriving an instruction from an upstream document.

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

- Do not move domain behaviour into a widget or mint a second authoritative
  copy of Source state for the UI.
- No blocking I/O, blocking receive, `block_on`, or repeated expensive work in
  the UI path; do not hold a mutex guard across a widget closure. Follow the
  snapshot and message-passing the console already uses.
- Widget identity is stable and derived from domain identity, never from a list
  position or a mutable display string.
- Reuse the existing style, `PALETTE`, font, `GlyphTable`, and paint
  infrastructure; do not rebuild a reusable rendering resource every frame.
- Respect clipping, points against physical pixels, the owned transform,
  hit-testing, focus ownership, and input consumption. Rendering and
  hit-testing stay consistent under resize and pan or zoom.
- Preserve the established repaint, animation, and reduced-motion behaviour.
- Do not paint a thousand Cells as a thousand AccessKit widgets to satisfy a
  tool, and do not add a domain-control API because the canvas is painted.
- General performance advice is not a licence for an unrequested optimisation;
  ADR 0040 and the atlas budget in `console.rs` are the standing decisions.
