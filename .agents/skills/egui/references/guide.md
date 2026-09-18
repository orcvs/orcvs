# egui in Orcvs

Detail behind `../SKILL.md`. Read the section you need; do not read this file to
find out what to do next, which is what the steps in `SKILL.md` are for.

## Resolved versions

Everything here is stated against one release of the egui stack. Confirm it
rather than trusting this table — the table is a record of what was true when it
was written, and `Cargo.lock` is the answer:

```sh
cargo tree --package console --locked -i egui
cargo tree --package console --locked --all-features --prefix none -e normal,dev \
  | grep -E '^(egui|eframe|epaint|emath|ecolor|kittest|accesskit)' | sort -u
```

| Crate | Version | Where it is declared | What it is for |
| --- | --- | --- | --- |
| `egui` | `=0.36.1` | `console/Cargo.toml` `[dependencies]` | The toolkit. `default-features = false`. |
| `eframe` | `=0.36.1` | `console/Cargo.toml` `[dependencies]` | Windowing and the `App` loop. `glow`, `wayland`, `x11`; no `wgpu`. |
| `epaint`, `emath`, `ecolor` | 0.36.1 | `Cargo.lock` | egui's own crates. Held at 0.36.1 by the lockfile. |
| `egui_inspection` | 0.36.1 | `Cargo.lock`, through `eframe/inspection` | The inspection plugin and wire protocol. Reached only by `console/inspection`. |
| `egui_kittest` | `=0.36.1` | `console/Cargo.toml`, non-WASM `dev-dependencies` | The UI test harness. Feature `eframe` only. |
| `egui_mcp` | 0.2.0 | `mise run install_egui_mcp`, `.mcp.json`, `.codex/config.toml` | The MCP server an agent attaches through. Not a workspace dependency. |

The second command is the duplicate check: one version of each egui crate.
Multiple `accesskit_consumer` entries inside `accesskit_winit` are expected
(dev and inspection only); a *second egui version* is a fault. The three
direct requirements are exact (`=0.36.1`). Pins are exact; moving the stack
is its own ticket — it re-reads the citations `console.rs` takes from this
release (`epaint-0.36.1/src/text/font.rs:567`,
`epaint-0.36.1/src/text/mod.rs:62`, `epaint-0.36.1/src/text/fonts.rs:734-748`,
`egui-0.36.1/src/containers/scene.rs`, `egui-0.36.1/src/context.rs:436-446`,
`emath-0.36.1/src/ts_transform.rs:55-57`,
`egui-0.36.1/src/response.rs:452-465`), re-runs the geometry and painting
tests in `console::tests`, and moves `egui_kittest` with it. `epaint`,
`emath` and `ecolor` are held by the lockfile; if they drift,
`cargo update epaint --precise 0.36.1` (and the same for the other two) is
the repair. `egui_mcp`'s version is its own: 0.2.0 requires
`egui_inspection ^0.36.0` and `egui ^0.36.0`.

## Provenance

Every reference below was fetched and read. Hosted documentation is the first
stop; when it is unavailable, read the matching tagged source or the resolved
registry source under `~/.cargo/registry/src/*/`, and never a different version.

Orcvs first — the contract, the domain model, the decisions:

- `AGENTS.md`, `CONTEXT.md`, `docs/tooling.md`.
- ADR 0022 — `orcvs` does not depend on `egui`, `eframe` or `winit`.
- ADR 0038 — the console owns the Source Grid transform, not an `egui::Scene`.
- ADR 0040 — the console paints from a value (`Paint`), not from widgets.
- ADR 0041 — the Playback Engine owns its state in one task.
- ADR 0042 — freedom from the toolkit is necessary to live in `orcvs`, not
  sufficient.

Then the framework, at the pinned release:

- <https://docs.rs/egui/0.36.1/egui/>
- <https://docs.rs/eframe/0.36.1/eframe/>
- <https://docs.rs/egui_kittest/0.36.1/egui_kittest/>
- <https://docs.rs/egui_inspection/0.36.1/egui_inspection/>
- <https://github.com/emilk/egui/tree/0.36.1>

Then upstream examples and demos, as patterns to read rather than code to copy:

- <https://github.com/emilk/egui/tree/0.36.1/examples>
- <https://github.com/emilk/egui/tree/0.36.1/crates/egui_demo_lib>
- <https://github.com/emilk/egui/blob/0.36.1/crates/egui_demo_lib/src/demo/toggle_switch.rs>
  — the custom-widget lifecycle in one file: `allocate_exact_size`, `interact`,
  `mark_changed` on a real change, `widget_info` for the semantic report, then
  paint, and only when `is_rect_visible`.
- <https://github.com/emilk/egui/tree/0.36.1/crates/egui_kittest>
- <https://github.com/emilk/egui/tree/0.36.1/crates/egui_inspection>

Then the inspection bridge:

- <https://docs.rs/egui_mcp/0.2.0/egui_mcp/>
- <https://github.com/rerun-io/kittest_inspector/tree/main/crates/egui_mcp>
  — 0.2.0 is commit `3833079a6a07798c3de9ddd2a48d0fc6712daac3`, path
  `crates/egui_mcp`, which is what the published crate records in
  `.cargo_vcs_info.json`. Read the tree at that commit rather than at `main`:
  <https://github.com/rerun-io/kittest_inspector/tree/3833079a6a07798c3de9ddd2a48d0fc6712daac3/crates/egui_mcp>.

Third-party guidance, adapted rather than followed:

- <https://github.com/livekit-examples/rust-dev-client/blob/main/AGENTS.md> —
  its UI section is the closest published statement of the same practices, and
  five of them are adopted in `SKILL.md` almost unchanged: no blocking work in
  the UI path, no mutex guard held across a widget closure, application state
  outside the UI, stable IDs keyed by domain identity rather than list position,
  and reusing textures and style rather than rebuilding them per frame. What is
  *not* adopted is its architecture: its `src/ui` module-per-widget rule, its
  actor conventions, its `From`/`TryFrom` guidance, and its release and spelling
  gates all belong to that application. Orcvs already has `AGENTS.md`, ADR 0040,
  ADR 0042 and `docs/tooling.md` saying where code goes and what verifies it,
  and a second, weaker copy of those rules would only disagree with them.

## What the console already decided

These are Orcvs conventions, not egui requirements. A change that wants to move
one is arguing with an ADR, which is a ticket rather than an edit.

- **The Source Grid is painted, not built from widgets.** `show_source`
  allocates one `Ui::interact` rectangle over the whole Grid with `Sense::CLICK`
  — deliberately not `Sense::click()`, which is `CLICK | FOCUSABLE` and would
  put that rectangle in the keyboard tab order — and issues one `Painter::extend`
  for every Shape. Cells are painted into the rectangle; they are not widgets.
  A `Painter::add` per Cell takes a `Context` write lock per Cell. ADR 0040 is
  the decision; `console/benches/paint.rs` measures it.
- **The console owns the transform.** `SourceView { to_global, adjusted }`, not
  an `egui::Scene` region. `egui::Scene::register_pan_and_zoom` is still used
  for zoom-at-pointer, smooth scroll and the zoom clamp, because all three are
  layer-independent; its drag-pan branch is switched off with
  `DragPanButtons::empty()` and replaced, because that branch corrects for a
  division that happens only inside a transformed layer. ADR 0038.
- **No layer transform.** A transformed layer reaches every `TextShape` in it
  through `Arc::make_mut` at end of pass, and a cached galley's refcount is
  never one. This is why glyphs are laid out at the size they are drawn at
  instead.
- **Glyph scale is quantised** to `GLYPH_SCALE_STEP` (an eighth), taken
  downwards. The long comment above that constant is an atlas budget with
  numbers in it; read it before changing how a size reaches a `FontId`.
- **Colour comes from `PALETTE` and `style()`**, `console/src/style.rs`. Not
  from a literal at the call site.
- **Reduced motion is honoured** through `prefers_reduced_motion` and
  `CursorEffectSettings::respecting_reduced_motion`, per platform.
- **The Source Grid answers, the owner acts.** `show_source` returns
  `Option<Position>`; `Console::ui` is what calls `orcvs.select`. Keep that
  shape: it is what lets the painting be tested without a running Orcvs.

## Inspection

Development-only. `console/inspection` forwards to `eframe/inspection`, which is
upstream's own integration — the feature pulls `egui_inspection` in and
`eframe::maybe_attach_inspection_plugin` attaches the plugin at start
(`eframe-0.36.1/src/lib.rs:214-222`). The console writes no server.

Two things have to be true before anything listens: the feature has to be
compiled in, and `EGUI_INSPECTION` has to be set to something truthy at run
time. Neither is true of a shipped build. `bind_addr_from_env` treats unset,
empty, `0` and `false` as off, `1` and `true` as `127.0.0.1:5719`, and anything
else as a `host:port` to bind (`egui_inspection-0.36.1/src/lib.rs:26-50`).

**Inspection is full, unauthenticated control of the running console.** Whatever
connects can inject input, read the widget tree, resize the window and take
screenshots. Bind loopback. Do not bind `0.0.0.0`; if you need it from another
machine, tunnel over SSH. `mise run inspect` writes
`EGUI_INSPECTION=127.0.0.1:…`. Why that spelling, and why the launcher
redirects `HOME` and `XDG_DATA_HOME`, is in `docs/tooling.md`.

### Setting it up

```sh
mise run install_egui_mcp
```

Installs `egui-mcp` 0.2.0 into `~/.cargo/bin`. Confirm it is on the PATH the
agent will use:

```sh
command -v egui-mcp
```

`.mcp.json` at the repository root registers it for Claude Code as a stdio
server named `egui-mcp`. Codex reads the same command and args from
`.codex/config.toml`. Trust this checkout so Codex loads that file. Why those
files are tracked and write nothing user-global is in `docs/tooling.md`.

The same table in `~/.codex/config.toml` remains an optional alternative when
you want the server outside this repository:

```toml
[mcp_servers.egui-mcp]
command = "egui-mcp"
args = []
```

If `~/.cargo/bin` is not on the agent's PATH, give the absolute path instead of
the bare name in either configuration.

### Launching, attaching, verifying, shutting down

```sh
mise run inspect              # binds 127.0.0.1:5719
mise run inspect 5720         # same, on another port
```

The task builds `console` with `--features inspection --locked`, then runs the
binary that build reported — not a written-down `./target/debug/console`, which
would ignore `CARGO_TARGET_DIR` and `build.target-dir` — with `HOME` and
`XDG_DATA_HOME` pointed at `target/inspection`. `target/` is ignored; delete
`target/inspection` to start from an empty Source.

That redirect isolates storage on macOS and Linux only. eframe resolves its
storage directory from `HOME` on macOS and `XDG_DATA_HOME` on Linux, but on
Windows from `SHGetKnownFolderPath(FOLDERID_RoamingAppData)`
(`eframe-0.36.1/src/native/file_storage.rs:37,46-90`) — a shell known-folder
lookup, not an environment variable — so no assignment moves it there. On
Windows an inspection session writes the real `%APPDATA%\Orcvs\data`. See
`docs/tooling.md`.

No MIDI destination is connected. The console connects an output only when the
MIDI menu is used, so an inspection session sends nothing to a real device
unless you tell it to. Do not tell it to.

Then, from the agent:

1. `attach` — defaults to `127.0.0.1:5719`, with `host`, `port` and
   `timeout_secs` available.
2. `query_tree` — read what the console actually exposes before writing a
   selector. See the next section for what it does and does not contain.
3. Drive it. Pointer tools take a target: `click` and `hover` accept a locator
   (`id`, `role`, `label_contains`, `content_contains`, `value_contains`) or a
   raw `pos` in logical points; `scroll` takes that target plus a `delta`;
   `drag` takes separate `start` and `end` targets. Keyboard and text tools do
   not: `type_text` takes the text and an optional semantic focus target, not a
   raw position; `press_key` takes a key and modifiers. Window and session
   tools are their own shape: `resize` takes width and height; `wait_for` takes
   query constraints and waiting parameters; `batch` takes an array of named
   actions with arguments. Inspect the connected server's tool schemas before
   calling an unfamiliar tool.
4. `screenshot` — returns a PNG inline and optionally writes it to `save_path`.
5. `status` to confirm the connection, `disconnect` to drop it. Close the
   console window to stop the server; nothing persists after the process exits.

### Troubleshooting

- **`attach` times out.** The console is not running, was built without the
  feature, or was launched without the variable. A console built without the
  feature and launched with the variable logs `Inspection env var set but app
  was compiled without eframe/inspection feature` — that message means the
  build, not the environment. Check the console's own stderr first.
- **Port already in use.** Another console is still running, or another egui app
  is on 5719. `mise run inspect 5721` and pass a matching `port` to `attach`.
- **`no accesskit tree yet`.** The first frame has not produced one. Run
  `wait_for` or take any action and retry. If it persists, the app is not
  rendering — an occluded or minimised window still produces a tree, so this is
  a start-up failure rather than a visibility one.
- **`node has no bounds — can't target`.** The node is in the tree but is not
  laid out, typically because it is inside a popup that is closed. Open the
  menu first, then query again: a closed menu's items are not in the tree at
  all.
- **A Cell is not in the tree.** It never will be. See below.
- **`screenshot` times out or returns nothing.** Screenshots need a rendered
  frame, and the OS does not produce one for an occluded or minimised window —
  on macOS the GPU surface is not available at all. Bring the window to the
  foreground. Reading the tree and injecting input work in the background;
  screenshots do not. This is the one step that cannot run on a headless
  machine, which is why no test depends on it.

## The Source view, and what inspection can reach

The console has two halves, and the boundary between them is where most
inspection mistakes happen.

**The chrome is ordinary egui.** The menu bar, the menus (`File`, `MIDI`,
`View`, `Theme`, `Tempo`), the Diagnostics checkbox, the colour pickers, the
sliders, the tempo `DragValue`, and every label and button inside the
Diagnostics window are real widgets. egui reports each to AccessKit with a role
and a label, so `query_tree` finds them and a locator is the right way to reach
them. This is what `console::kittest_tests` queries and what `egui-mcp` should
click.

**The Source Grid is one rectangle.** Sixty-four by forty Cells are painted
into a single interactive rectangle. `query_tree` will not find a Cell; the
Cursor is not in the tree either. Confirm selection from geometry, Diagnostics,
or a screenshot.

A `query_tree` against a default console bears this out: five `Button` nodes
labelled `File`, `MIDI`, `View`, `Theme` and `Tempo`, a handful of unlabelled
`GenericContainer`s, and one `Unknown` node whose bounds are the whole console
area below the menu bar — 1024 by 640 points at the default window, holding
10240 Cells, most of them past its edges, and reporting none of them.

So the Source view is reached four ways, and each answers a different question:

| Question | Tool |
| --- | --- |
| Is this control there, and does it say what it should? | Semantic query — `get_by_label`, `query_tree` with a role or text predicate. Chrome only. |
| Does a click at this Cell select this Cell? | Coordinate input, with the coordinate **derived from the live transform**: `presented_grid(view.to_global, source_bounds(grid), grid, ppp).cell_rect(col, row).center()`. Never a written-down screen position. |
| Is the geometry right — square Cells, the fit, the letterboxing, the visible range? | Assert on `GridViewport` directly. `console/src/grid_viewport.rs` is full of these and they need no window. |
| Did it paint the right thing? | Assert on the `Shape`s, as `console::tests` does. |
| Does it *look* right? | A human, or a screenshot in an inspection session. Not an automated gate — see below. |

Adding AccessKit metadata is right for a real control that is missing one —
`Response::widget_info` on a custom widget a viewer operates. It is wrong as a
way to make a painted Cell findable.

## Regression tests

`console::kittest_tests` (`console/src/console/kittest_tests.rs`) drives the
shipped `Console` through `Harness::build_eframe`, which calls `App::logic` and
`App::ui` with no wrapper of its own
(`egui_kittest-0.36.1/src/app_kind.rs:36-44`). It is a child module of
`console::console`, so it reads the private fields the assertions are about,
and it holds three things:

- a menu control found by label and asserted through both the console's state
  and the window it opens;
- the Source's keyboard path, which is not a widget and so has no locator;
- the pointer-to-Cell round trip after a resize and after a zoom, with every
  coordinate read back out of the transform in force at the moment of the click.

That third one is a round trip and not a geometry assertion, and the difference
decides where a new case belongs. The click target comes from the same
`presented_grid` call `show_source_scene` makes, so what it holds is that
`cell_rect` and `cell_at` still invert each other under a transform the console
has moved, and that a click at the coordinate `cell_rect` answers reaches the
Source as that Cell. A fault inside `presented_grid` itself — a mishandled
`pixels_per_point`, a letterbox origin off by a Cell — moves both sides of that
equality and passes here. Where the Grid actually lands is `console::tests`' and
`grid_viewport::tests`' to assert, and the module documentation names which
tests those are.

`console::tests` beside it is the older harness and is not superseded: it
asserts on `Shape`s and `GridViewport` at a finer grain than any tree query
reaches. Extend whichever fits. Add to `kittest_tests` when the question is
"can a viewer get at this" or "does the input path work end to end"; add to
`tests` or `grid_viewport::tests` when it is "what did it paint" or "where
exactly did it put it" — absolute geometry belongs where nothing cancels it.

Rules the module holds and a new test should too:

- Bounded frames. `Harness::run` loops until no immediate repaint is requested,
  and the console requests one whenever the cursor effect is animating — which,
  at `CursorEffectSettings::default`, is always. Use `step` (one frame per
  queued event) and `run_steps(n)`. Never sleep, never read the clock, never
  wait on Playback.
- Derive every coordinate, and state the claim at the height deriving reaches.
  A written-down position either keeps passing after the mapping breaks or fails
  for reasons unrelated to it; a position derived from the mapping under test
  can only ever prove the round trip, because a shared error moves both sides of
  it. Write the test's claim as a round trip and leave the absolute geometry to
  the modules that assert it without going through the mapping.
- No test-only seam in shipped code. A fixture production cannot construct is
  built in the test module, below the shipped entry point — `AGENTS.md`'s rule,
  and the reason `Console::new` takes eframe's own `CreationContext` here rather
  than growing a constructor for tests.
- `#[tokio::test]`. `Console::new` starts the Playback Engine as a task and a
  task needs a runtime (ADR 0041).

Run them with the rest of the console's tests:

```sh
cargo nextest run --package console --locked
cargo nextest run --package console --locked -E 'test(kittest_tests)'
```

### Why there is no snapshot test

`egui_kittest`'s `snapshot` feature has no renderer of its own
(`egui_kittest-0.36.1/src/renderer.rs:36-45`); producing an image needs the
`wgpu` feature as well. That would put the wgpu and naga trees into the dev
graph and a working GPU into every CI job that runs the console's tests, for
coverage the `Shape`-level assertions in `console::tests` already hold at a
finer grain — they can say *which* Shape, in what order, at what rounded
rectangle, which a pixel diff cannot. The console also ships the `glow`
renderer, and pulling wgpu in for tests alone is the renderer split this
repository has so far avoided.

So: no snapshots, and none are claimed. If a visual regression ever appears
that no Shape assertion can express — a font atlas fault, a blend or gamma
change — that is the argument for adding `egui_kittest`'s `snapshot` and `wgpu`
features, behind its own ticket, with the viewport, `pixels_per_point`, theme,
fonts and animation state all fixed by the harness builder, and with baseline
updates made deliberately rather than by accepting whatever the last run
produced. Until then, `screenshot` in an inspection session is the visual check,
and it is a human looking at it.

## Verification

The console gate is `AGENTS.md`'s (skill step 6).
