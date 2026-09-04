# Test the console at its egui seam

**Goal:** Verify the console where it is currently unverified — the boundary between egui and the running Orcvs — without freezing visual decisions that are still open.

## Why

`shell` has twenty real tests and not one of them constructs an `egui::Context`. Everything from `Console::update` down is unexercised: the Scene, the Cell button grid, the Cursor overlay, the menu bar, the MIDI and Tempo menus.

The input path is the sharpest case. `translate_event` is tested in isolation and `Orcvs::event_handler` is tested in isolation, and nothing asserts that the two halves meet. A change that dropped every translated event on the floor would pass the full gate.

The palette is the second case. `restyle-egui-console/02` pins twenty-two tokens to exact hex values and names `shell/src/theme.md` as the decided record. The six tests in `shell/src/style.rs` assert only relationships — that Bang is soft red, that Numbers are distinct from Functions, that sector strength attenuates alpha. No test asserts a value, so the record and the code can drift apart silently.

## Rules

Image comparison is out of scope, and this is a decision rather than an omission. `restyle-egui-console/02` records the Cursor focus matrix as proposed rather than settled, and `03` exists to report the captured reticle against the prototype and say whether its four bands are kept, retuned, or dropped. A committed baseline would assert that an explicitly open question must not change. `03` does the judgement an image diff cannot do; this effort does the regression checking `03` is not for.

Test the seam, not the derivation. `orcvs/src/render_frame.rs` already covers which Glyph each Cell receives, the sector seam strengths, the Cursor bloom bands, and the determinism of the hashed edge breakup. `cell_visuals` and `sector_line` already cover the mapping from Glyph and state to colour. None of that is repeated here.

Assert on state, not on labels. The Source Grid is a field of `egui::Button`s whose labels are single characters and mostly blank, so AccessKit queries cannot address them unambiguously. The harness earns its place by driving the real `eframe::App` end to end; the assertions are made against `Console` and the Render Frame afterwards. Label queries are used only for the menus, which carry distinct labels.

Values, not adjectives. A palette test names hex.

## Tooling

`egui_kittest`, version-matched to the pinned `egui` and `eframe` at 0.36.1, as a dev-dependency with the `eframe` feature only. Not `wgpu`, not `snapshot`: those pull wgpu, `dify`, and `image` into a `deny.toml` graph that runs `all-features` across five targets including `wasm32-unknown-unknown`, and they buy only the image comparison this effort has ruled out.

The dependency enables `egui/accesskit` for test builds. Under resolver 3 a dev-dependency's features do not reach an ordinary build, so the shipped binary is unchanged.

No CI change. These are ordinary tests and run inside the existing `cargo nextest run --workspace --profile ci --locked` line, so `scripts/check-tooling-contract.sh` stays byte-identical and the workflow does not move.

## Vocabulary

**Console** is domain vocabulary and belongs in `CONTEXT.md`. The glossary already leans on the word in four definitions — Orcvs, Cursor, Marker, and Render Frame — without ever defining it, while the crate is named `shell`, a word the glossary explicitly avoids under Application Command Function.

**Reference Render** is tooling vocabulary and is recorded here rather than in `CONTEXT.md`, which stays free of implementation detail. It names one committed image of the console captured for comparison against a later capture. It is reserved now because `Snapshot` already means Source Snapshot across `CONTEXT.md` and a dozen ADRs, and the overload has already begun: `restyle-egui-console/02` calls a plain `render_frame.rs` unit test "the snapshot test".

**Semantic palette** is not used as a term. It has no definition in the repository, and it flattens
three unlike groups of tokens in `ConsolePalette`: seven Glyph colours keyed by the `Glyph` term
`CONTEXT.md` defines, four console surfaces, and eleven Cursor and selection treatments. `CONTEXT.md`
also lists "semantic Grid" and "semantic Source" under `_Avoid_`, so the name pattern cuts against the
glossary. Where the distinction matters, the term is **Glyph colour**.

## Issues

- `issues/01-rename-the-shell-crate-to-console.md`
- `issues/02-define-console-in-the-glossary.md`
- `issues/03-pin-the-console-palette-to-its-recorded-values.md`
- `issues/04-drive-the-console-input-path-through-the-real-app.md`
- `issues/05-cover-the-source-grid-viewport-geometry.md`

## Later

Reference Renders, on a stated trigger rather than a schedule. Once `restyle-egui-console/01` and `02` have landed and `03` has blessed a candidate, the question changes from "is this right" to "has this drifted", and a baseline against a settled palette and a settled reticle is worth committing. Reconsider then, and reconsider sooner if `05` finds the viewport geometry cannot be expressed as pure functions.

The WASM console's rendering. `test_wasm` exercises `orcvs` logic in headless Firefox, not the egui layer. `restyle-egui-console/03`'s WASM captures cover it by review.

## Order

`03` covers the dark palette only. `restyle-egui-console/04` ports the per-theme style mechanism and
renames the constant it pins; `restyle-egui-console/05` decides a light palette and extends the test
to it. Neither blocks `03`, which should run against whichever constant exists when it is written.

`05` follows `restyle-egui-console/01`, which is building `grid_viewport.rs` now. Nothing else here waits on it.

These issues carry no `Tags:` line. `docs/agents/issue-tracker.md` requires every open tagged issue to sit inside the release Gate's dependency closure, and `v1-release/03` does not depend on this effort. Tagging them would misreport the release's critical path.
