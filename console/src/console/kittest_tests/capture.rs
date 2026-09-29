//!
//! Release captures: the running console rendered to images for a human
//! reviewer to judge against the release's visual checklist.
//!
//! Compiled only under `console`'s `release-capture` feature, which the
//! dispatch-only `.github/workflows/release-captures.yml` enables and no
//! verification tier does. It drives the shipped `Console` through the same
//! `Harness::build_eframe` the tests above use, with `egui_kittest`'s wgpu
//! renderer added, so what is rendered is what `Console::ui` painted.
//!
//! # What a capture proves before it renders
//!
//! Pixels are for the reviewer; the run's own claim is that the frame about to
//! be rendered is the one the checklist describes. Before each image is
//! rendered the test asserts that the seeded Source loaded, that the pinned
//! mode and Theme are the ones presented, that egui's zoom is 1.0, and that
//! every checklist state is among the Cells the console draws. A missing state
//! fails the run and writes no image.
//!
//! # How the pinned settings arrive
//!
//! Each one enters the way it enters the shipped console, so no shipped code
//! knows it is being captured:
//!
//! - the Source, from `orcvs_source` in an `app.ron` written with the native
//!   storage codec (`persistence::RonFileStorage`), read by `Console::new`;
//! - the mode and zoom, from egui memory stored under eframe's `"egui"` key in
//!   the same file and restored into the `Context` before the console is
//!   built, as eframe's native runner does
//!   (`eframe-0.36.2/src/native/winit_integration.rs:56-57`);
//! - the dark and light Theme selections, from a `config.toml` that
//!   `Config::read` reads, as `Config::start` does.
//!
//! The harness sets egui's theme preference to its builder's theme after the
//! app is built (`egui_kittest-0.36.2/src/lib.rs:142`), so the builder is given
//! the same mode the stored memory holds, and the assertion reads the
//! preference the frame was presented under.
//!
//! # Output
//!
//! `ORCVS_CAPTURE_DIR` names the directory the images and `manifest.json` are
//! written to. `ORCVS_CAPTURE_SHA` and `ORCVS_CAPTURE_RUNNER` are recorded in
//! the manifest as given. All three are required: the workflow sets them, and
//! a run without them has nowhere to put an image and nothing to name it by.
//!

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use eframe::egui_wgpu::{self, RenderState};
use egui::{Key, Modifiers, Rect, Vec2, pos2};
use egui_kittest::Harness;
use egui_kittest::wgpu::{WgpuTestRenderer, create_render_state, default_wgpu_setup};
use orcvs::render_frame::RenderFrame;
use orcvs::source::{OperandState, Source, SourceCommander, SourcePaint, Token, file};

use super::super::menu_bar::TOP_PANEL_HEIGHT;
use super::super::panel::BOTTOM_PANEL_HEIGHT;
use super::super::tests::start_console;
use super::{Console, presented_source};
use crate::config::{CONFIG_FILE, Config};
use crate::persistence::{IsolatedRonDir, RonFileStorage};
use crate::theme::{Appearance, Theme, okabe_ito, orcvs_light};
use crate::theme_registry::ThemeRegistry;
use crate::{FramePaint, Paint};

///
/// The seeded Source: one Source File reaching every checklist state, shared
/// with the web capture so both targets show the same Cells.
///
const FIXTURE: &str = include_str!("../../../tests/fixtures/release-capture.orcvs");

///
/// The settings file every capture starts from. Both identities are named
/// rather than left to the defaults, so a change of default cannot change
/// what a capture shows.
///
const SETTINGS: &str = "[theme]\ndark = \"okabe-ito\"\nlight = \"orcvs-light\"\n";

///
/// The key eframe stores egui memory under
/// (`eframe-0.36.2/src/native/epi_integration.rs:444`). eframe keeps it
/// private, so it is written out here.
///
const EGUI_MEMORY_KEY: &str = "egui";

///
/// Physical pixels per logical point. Two, so an 11.5-point glyph keeps a
/// body the reviewer can read rather than antialiased edge alone. It is the
/// display's density, not egui's zoom, which stays 1.0.
///
const PIXELS_PER_POINT: f32 = 2.0;

///
/// The Cell the Region is anchored at, reached from the Cursor's start at
/// `00 00` by arrow keys. It is empty ground to the right of the fixture, so
/// the lasso and the Region's fill are seen over the bare Source.
///
const REGION_ANCHOR: (usize, usize) = (20, 2);

///
/// The Region's extent: the anchor and the Cells one Shift-Right and one
/// Shift-Down span from it, with the Cursor at the live end.
///
const REGION_COLUMNS: std::ops::Range<usize> = REGION_ANCHOR.0..REGION_ANCHOR.0 + 2;
const REGION_ROWS: std::ops::Range<usize> = REGION_ANCHOR.1..REGION_ANCHOR.1 + 2;

///
/// Every state the release checklist names that a Render Frame can show, in
/// the checklist's order.
///
const CHECKLIST: [&str; 13] = [
    "occupied Cell",
    "empty Cell",
    "Function",
    "Number",
    "Note",
    "Bang",
    "Comment",
    "Sequence",
    "fitted Output Portal",
    "invalid operand's diagnostic",
    "Region",
    "Sector Seam",
    "Cursor",
];

///
/// How each capture is made, recorded in the manifest beside it.
///
const PROCEDURE: &str = "egui_kittest Harness::build_eframe over the shipped Console::new, \
    rendered by egui_kittest's wgpu renderer. Storage: an app.ron in eframe's native RON codec \
    holding orcvs_source (console/tests/fixtures/release-capture.orcvs) and egui memory with the \
    mode as theme_preference and zoom_factor 1.0, restored into the Context before the console \
    is built. Settings: config.toml with theme.dark = okabe-ito and theme.light = orcvs-light, \
    over the built-in Themes alone. Input: ArrowRight x20, ArrowDown x2, Shift+ArrowRight, \
    Shift+ArrowDown, raising a Region over four empty Cells. Asserted before rendering: the \
    seeded Source, the mode and Theme presented, egui zoom 1.0 and every checklist state among \
    the drawn Cells.";

#[derive(Clone, Copy)]
struct Viewport {
    name: &'static str,
    size: Vec2,
}

const VIEWPORTS: [Viewport; 2] = [
    Viewport {
        name: "wide",
        size: Vec2::new(1200.0, 700.0),
    },
    Viewport {
        name: "tall",
        size: Vec2::new(700.0, 1200.0),
    },
];

///
/// The two modes a capture pins, each with the built-in Theme the settings
/// select for it.
///
fn modes() -> [(egui::Theme, Theme); 2] {
    [
        (egui::Theme::Dark, okabe_ito()),
        (egui::Theme::Light, orcvs_light()),
    ]
}

///
/// What one image is of, for the manifest.
///
struct Captured {
    file: String,
    viewport: Viewport,
    mode: egui::Theme,
    theme: Theme,
    zoom: f32,
    adapter: eframe::wgpu::AdapterInfo,
    pixels: (u32, u32),
}

fn required(variable: &str) -> String {
    std::env::var(variable)
        .unwrap_or_else(|_| panic!("{variable} is required by the release capture"))
}

///
/// The seeded Source, read by the shipped Source File reader.
///
fn seeded_source() -> Source {
    file::read(FIXTURE.as_bytes()).expect("the release capture fixture is a Source File")
}

///
/// Writes what a start of the native console reads into `dir`: an `app.ron`
/// holding the seeded Source and egui memory pinned to `mode` at zoom 1.0,
/// and a `config.toml` naming both Theme selections. Answers the storage the
/// console starts over, read back from that file.
///
fn stored_start(dir: &Path, mode: egui::Theme) -> RonFileStorage {
    let mut memory = egui::Memory::default();
    memory.options.theme_preference = mode.into();
    memory.options.zoom_factor = 1.0;

    let mut file = RonFileStorage::create(dir);
    crate::persistence::save(&mut file, &SourceCommander::with_source(seeded_source()));
    eframe::set_value(&mut file, EGUI_MEMORY_KEY, &memory);
    eframe::Storage::flush(&mut file);

    std::fs::write(dir.join(CONFIG_FILE), SETTINGS).expect("the capture's settings file");

    RonFileStorage::from_file(dir)
}

///
/// A running console at `viewport` in `mode`, started from `storage` and the
/// settings file beside it, and rendered through `render_state`.
///
fn capture_harness<'a>(
    viewport: Viewport,
    mode: egui::Theme,
    storage: &'a RonFileStorage,
    settings: PathBuf,
    render_state: RenderState,
) -> Harness<'a, Console> {
    Harness::builder()
        .with_size(viewport.size)
        .with_pixels_per_point(PIXELS_PER_POINT)
        .with_theme(mode)
        .renderer(WgpuTestRenderer::from_render_state(render_state))
        .build_eframe(move |cc| {
            let memory = eframe::get_value::<egui::Memory>(storage, EGUI_MEMORY_KEY)
                .expect("the stored egui memory");
            cc.egui_ctx.memory_mut(|restored| *restored = memory);
            cc.storage = Some(storage);
            start_console(cc, ThemeRegistry::built_in(), Config::read(&settings))
        })
}

///
/// Raises the Region: the Cursor to [`REGION_ANCHOR`] by arrow keys, then one
/// Shift-Right and one Shift-Down. One frame per press, as a viewer's keys
/// arrive.
///
fn raise_region(harness: &mut Harness<'_, Console>) {
    let presses = std::iter::repeat_n((Modifiers::NONE, Key::ArrowRight), REGION_ANCHOR.0)
        .chain(std::iter::repeat_n(
            (Modifiers::NONE, Key::ArrowDown),
            REGION_ANCHOR.1,
        ))
        .chain([
            (Modifiers::SHIFT, Key::ArrowRight),
            (Modifiers::SHIFT, Key::ArrowDown),
        ]);
    for (modifiers, key) in presses {
        harness.key_press_modifiers(modifiers, key);
        harness.step();
    }
    harness.run_steps(2);
}

///
/// The part of the screen the Source Grid is drawn in: below the menu bar and
/// above the Panel.
///
fn source_area(ctx: &egui::Context) -> Rect {
    let screen = ctx.content_rect();
    Rect::from_min_max(
        pos2(screen.left(), screen.top() + TOP_PANEL_HEIGHT),
        pos2(screen.right(), screen.bottom() - BOTTOM_PANEL_HEIGHT),
    )
}

///
/// Which checklist states the Cells the console draws in `area` show, read
/// from the Render Frame and the Paint derived from it with `theme`.
///
fn states_drawn(
    frame: &RenderFrame,
    harness: &Harness<'_, Console>,
    area: Rect,
    theme: &Theme,
) -> BTreeSet<&'static str> {
    let grid = frame.grid();
    let visible = presented_source(harness).visible_positions(area, grid);
    let paint = Paint::derive_with_theme(FramePaint::new(frame, visible), theme);
    let mut drawn = BTreeSet::new();

    for (position, painted) in paint.cells() {
        let cell = frame.at(position);
        drawn.insert(if cell.content().is_some() {
            "occupied Cell"
        } else {
            "empty Cell"
        });
        match cell.source_paint() {
            SourcePaint::Function => {
                drawn.insert("Function");
            }
            SourcePaint::Bang => {
                drawn.insert("Bang");
            }
            SourcePaint::Comment => {
                drawn.insert("Comment");
            }
            SourcePaint::Operand { token, state } => {
                match (token, state) {
                    (Token::Number, OperandState::Valid) => {
                        drawn.insert("Number");
                    }
                    (Token::Note, OperandState::Valid) => {
                        drawn.insert("Note");
                    }
                    (Token::Sequence, _) => {
                        drawn.insert("Sequence");
                    }
                    _ => {}
                }
                if state == OperandState::Invalid && frame.diagnostic_covers(position) {
                    drawn.insert("invalid operand's diagnostic");
                }
            }
            SourcePaint::Unclaimed => {}
        }
        if cell.output_portal() {
            drawn.insert("fitted Output Portal");
        }
        if painted.sector_left.is_some() || painted.sector_top.is_some() {
            drawn.insert("Sector Seam");
        }
    }

    // The Region counts when every Cell of it is drawn, not merely one.
    let region = frame.region();
    let region_drawn = !region.is_one_cell()
        && region.columns().all(|column| {
            region.rows().all(|row| {
                grid.position(column, row)
                    .is_some_and(|position| paint.cells().any(|(drawn, _)| drawn == position))
            })
        });
    if paint.region_spans() && region_drawn {
        drawn.insert("Region");
    }
    // The Cursor's Cell among the drawn Cells, where its fill and its Effect
    // are painted.
    if paint.cursor().is_some() {
        drawn.insert("Cursor");
    }

    drawn
}

///
/// Asserts that the frame `harness` last ran is the one the capture claims to
/// be of: the seeded Source, `mode` and `theme` presented, egui zoom 1.0, and
/// every checklist state drawn.
///
fn assert_capturable(harness: &Harness<'_, Console>, mode: egui::Theme, theme: &Theme) {
    let situation = format!("{mode:?} capture");
    let console = harness.state();

    assert_eq!(
        console.orcvs.source().snapshot(),
        seeded_source().snapshot(),
        "{situation}: the console did not start the seeded Source its storage held"
    );

    assert_eq!(
        harness.ctx.options(|options| options.theme_preference),
        mode.into(),
        "{situation}: the mode is not the one pinned"
    );
    assert_eq!(
        harness.ctx.theme(),
        mode,
        "{situation}: the frame was not presented in the pinned mode"
    );
    assert_eq!(
        *console.themes.presented(Appearance::from(mode)),
        *theme,
        "{situation}: the settings did not select {}",
        theme.identity
    );
    assert_eq!(
        console.painted_backdrop, theme.window_background,
        "{situation}: the last frame was not painted from {}",
        theme.identity
    );
    // The harness turns the text cursor's blink off in every style after the
    // console installs them, so that one field is set the same way here.
    let mut chrome = crate::style::style(theme).visuals;
    chrome.text_cursor.blink = false;
    assert_eq!(
        harness.ctx.global_style().visuals,
        chrome,
        "{situation}: the chrome is not styled from {}",
        theme.identity
    );
    assert_eq!(
        console.themes.notice_count(),
        0,
        "{situation}: the settings raised notices: {:?}",
        console.themes.notice_list()
    );

    assert_eq!(
        harness.ctx.zoom_factor(),
        1.0,
        "{situation}: egui zoom is not 1.0"
    );

    let region = console.orcvs.region();
    assert_eq!(
        (region.columns(), region.rows()),
        (REGION_COLUMNS, REGION_ROWS),
        "{situation}: the keys did not raise the Region the capture describes"
    );

    let frame = console.orcvs.render_frame();
    let drawn = states_drawn(&frame, harness, source_area(&harness.ctx), theme);
    let missing: Vec<&str> = CHECKLIST
        .into_iter()
        .filter(|state| !drawn.contains(state))
        .collect();
    assert!(
        missing.is_empty(),
        "{situation}: the frame about to be rendered does not show {missing:?}"
    );
}

///
/// One capture: a console at `viewport` in `mode`, asserted, rendered and
/// written to `out`.
///
fn capture(out: &Path, viewport: Viewport, mode: egui::Theme, theme: &Theme) -> Captured {
    let dir = IsolatedRonDir::new();
    let storage = stored_start(dir.path(), mode);
    let render_state = create_render_state(
        default_wgpu_setup(),
        egui_wgpu::RendererOptions::PREDICTABLE,
    );
    let adapter = render_state.adapter.get_info();

    let mut harness = capture_harness(
        viewport,
        mode,
        &storage,
        dir.path().join(CONFIG_FILE),
        render_state,
    );
    harness.run_steps(2);
    raise_region(&mut harness);

    assert_capturable(&harness, mode, theme);

    let image = harness
        .render()
        .expect("the wgpu renderer rendered the frame");
    let pixels = (image.width(), image.height());
    assert_eq!(
        pixels,
        (
            (viewport.size.x * PIXELS_PER_POINT) as u32,
            (viewport.size.y * PIXELS_PER_POINT) as u32
        ),
        "the image is not the viewport at {PIXELS_PER_POINT} pixels per point"
    );

    let file = format!("native-{}-{}.png", viewport.name, theme.identity);
    image
        .save(out.join(&file))
        .expect("the capture's image was written");

    Captured {
        file,
        viewport,
        mode,
        theme: theme.clone(),
        zoom: harness.ctx.zoom_factor(),
        adapter,
        pixels,
    }
}

///
/// `text` as a JSON string literal.
///
fn json(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    for character in text.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            character if character.is_control() => {
                quoted.push_str(&format!("\\u{:04x}", u32::from(character)));
            }
            character => quoted.push(character),
        }
    }
    quoted.push('"');
    quoted
}

///
/// The manifest: for each image, what the release checklist asks a reviewer
/// to know about it.
///
fn manifest(sha: &str, runner: &str, captures: &[Captured]) -> String {
    let images: Vec<String> = captures
        .iter()
        .map(|captured| {
            let adapter = &captured.adapter;
            format!(
                concat!(
                    "    {{\n",
                    "      \"file\": {file},\n",
                    "      \"target\": \"native\",\n",
                    "      \"sha\": {sha},\n",
                    "      \"runner_os\": {runner},\n",
                    "      \"renderer\": {{\n",
                    "        \"harness\": \"egui_kittest 0.36.2 wgpu renderer\",\n",
                    "        \"adapter\": {adapter},\n",
                    "        \"backend\": {backend},\n",
                    "        \"device_type\": {device_type},\n",
                    "        \"driver\": {driver},\n",
                    "        \"driver_info\": {driver_info}\n",
                    "      }},\n",
                    "      \"viewport\": {{ \"name\": {viewport}, \"width_points\": {width}, ",
                    "\"height_points\": {height}, \"pixels_per_point\": {ppp:?}, ",
                    "\"width_pixels\": {width_pixels}, \"height_pixels\": {height_pixels} }},\n",
                    "      \"mode\": {mode},\n",
                    "      \"theme\": {{ \"identity\": {identity}, \"name\": {name} }},\n",
                    "      \"zoom\": {zoom:?},\n",
                    "      \"checklist\": [{checklist}],\n",
                    "      \"procedure\": {procedure}\n",
                    "    }}"
                ),
                file = json(&captured.file),
                sha = json(sha),
                runner = json(runner),
                adapter = json(&adapter.name),
                backend = json(&adapter.backend.to_string()),
                device_type = json(&format!("{:?}", adapter.device_type)),
                driver = json(&adapter.driver),
                driver_info = json(&adapter.driver_info),
                viewport = json(captured.viewport.name),
                width = captured.viewport.size.x,
                height = captured.viewport.size.y,
                ppp = PIXELS_PER_POINT,
                width_pixels = captured.pixels.0,
                height_pixels = captured.pixels.1,
                mode = json(&format!("{:?}", captured.mode)),
                identity = json(&captured.theme.identity.to_string()),
                name = json(&captured.theme.name),
                zoom = captured.zoom,
                checklist = CHECKLIST.map(json).join(", "),
                procedure = json(PROCEDURE),
            )
        })
        .collect();
    format!(
        "{{\n  \"sha\": {},\n  \"runner_os\": {},\n  \"images\": [\n{}\n  ]\n}}\n",
        json(sha),
        json(runner),
        images.join(",\n")
    )
}

///
/// The four native release captures — {wide, tall} × {Okabe–Ito, Orcvs
/// Light} — and their manifest.
///
#[tokio::test]
async fn the_release_captures_show_every_checklist_state() {
    let out = PathBuf::from(required("ORCVS_CAPTURE_DIR"));
    let sha = required("ORCVS_CAPTURE_SHA");
    let runner = required("ORCVS_CAPTURE_RUNNER");
    std::fs::create_dir_all(&out).expect("the capture directory");

    let mut captures = Vec::new();
    for viewport in VIEWPORTS {
        for (mode, theme) in modes() {
            captures.push(capture(&out, viewport, mode, &theme));
        }
    }

    std::fs::write(
        out.join("manifest.json"),
        manifest(&sha, &runner, &captures),
    )
    .expect("the capture manifest was written");
}
