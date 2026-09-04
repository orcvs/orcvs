# 04 — Drive the console input path through the real App

**What to build:** Run the real `eframe::App` under a test harness and assert that keyboard and
pointer input travel from egui through the Console into the running Orcvs and back out to what is
drawn.

**Blocked by:** 01 — Rename the shell crate to console.

**Status:** ready-for-agent

- [ ] `egui_kittest` is added as a dev-dependency at the version matching the pinned `egui` and
      `eframe`, with the `eframe` feature and no others. Not `wgpu`, not `snapshot`.
- [ ] The rationale is recorded per the repository contract, and `mise run audit_deps` is run with
      its result recorded. The new tree is `kittest` and `accesskit_consumer`; the `wgpu`, `dify` and
      `image` trees are specifically not taken on.
- [ ] The harness is built with `Harness::builder().build_eframe(Console::new)`, at a fixed
      `with_size` and `with_pixels_per_point`, so a layout-dependent assertion is reproducible.
- [ ] Typing a Glyph writes it into the selected Cell, and the Source reads it back.
- [ ] Arrow keys move the Cursor and clamp at all four Grid edges.
- [ ] Backspace, Delete and Space each do what `translate_event` claims they do, end to end.
- [ ] A key egui delivers that `translate_event` drops — Enter, or a key release — changes nothing.
- [ ] Clicking a Cell button selects that Cell's Position.
- [ ] The menus are exercised by label: File and its Quit item, View, the Tempo drag-commit, and the
      MIDI menu opening.
- [ ] Assertions are made against `Console` and `Orcvs` state and the Render Frame, not by querying
      grid Cell labels. The Source Grid is a field of Buttons whose labels are single characters and
      mostly blank, so a label query cannot address them unambiguously.
- [ ] The tests run inside the existing `cargo nextest run --workspace --profile ci --locked` line.
      No feature gate, no new mise task, and `scripts/check-tooling-contract.sh` is untouched.

## Comments

This is the gap the effort exists for. `translate_event` has a test covering every key it accepts and
several it drops. `Orcvs::event_handler` is covered on the other side. Nothing asserts the two are
connected, so a change that dropped every translated event would pass the full gate.

`Console::new` takes `&eframe::CreationContext<'_>` and `build_eframe` takes
`FnOnce(&mut CreationContext) -> State`, so the real application constructs under the harness with no
refactor. `Console` is already `pub` and already implements `eframe::App`.

The dependency enables `egui/accesskit` for test builds. Under resolver 3 a dev-dependency's features
do not reach an ordinary build, so the shipped binary is unchanged; confirm this with `cargo tree`
rather than assuming it.

Leave the MIDI destination branches alone. They need a fake backend, and `MidiDeviceSelection`
already has five tests at that seam.
