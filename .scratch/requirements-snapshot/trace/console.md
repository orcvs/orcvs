# Trace — Console (CONS)

Inventory at `c8cc197c32cfd7414f617cbc8fc5e4a846a44af4`. Owned ADRs: 0022, 0038, 0040, 0042, 0044,
0045, 0046, 0047, 0048, 0050, 0051, 0052, 0053, 0058, 0078, 0079, 0080, 0081, 0082, 0083, 0084,
0089, 0092, 0095, 0096, 0098, 0100, 0102. No Rust comment in `console/` cites a console-topic
`CONTEXT.md` entry; `console/src/function_reference.rs` cites only the language entries Halt and
Terminal Output Function (`grep -rn 'CONTEXT.md' --include=*.rs`).

## Source disposition

| Source at c8cc197 | Claim | Disposition | Owner / reason | Basis |
| --- | --- | --- | --- | --- |
| ADR 0022 | `orcvs` does not depend on `egui`, `eframe` or `winit`; no module moves into `orcvs` until free of them | retained CONS-010 Core crate free of the UI toolkit | | source-stated (0042 says it stands) |
| ADR 0022 | Crate split `lang` / `orcvs` / console; console owns window, style, presentation | retained CONS-010 | | source-stated |
| ADR 0022 | MIDI device selection stays in the console; choosing an output port is configuration, not part of a running Orcvs | retained CONS-012 Output device choice is console configuration | | source-stated |
| ADR 0022 | Application state is named `Orcvs`, not `App` | retained CONS-010 (naming note) | CONTEXT "Orcvs (running instance)" avoids App | source-stated |
| ADR 0022 | Measurements (5,900 lines, 48 s cold check); further Source/Playback split waits | historical | reasoning | source-stated |
| ADR 0022 | `shell` name left as written | historical | rename record | source-stated |
| ADR 0042 | Freedom from the toolkit is necessary, not sufficient: facts about a Source in `orcvs`, decisions about how the console shows it in console; `Glyph` stays in `orcvs` | retained CONS-011 Facts in `orcvs`, presentation in the console | enforced at review, not by compiler | source-stated |
| ADR 0042 | Three drift examples (`SECTOR_SEAM_STRENGTHS`, `CursorBloom`, `DEFAULT_FONT_SIZE`); numbering note; rejected alternative | historical | evidence | source-stated |
| ADR 0038 | Console holds the Source's translation itself rather than an `egui::Scene`, applies it when computing each Cell rect; no transformed layer | retained CONS-020 The console owns the Source translation; no layer transform | scale half superseded by 0058 | source-stated |
| ADR 0038 | Owned scale and `FontId` derived from it; quantised glyph step | superseded | ADR 0058 (scale gone; Glyph at Source's own size) | source-stated |
| ADR 0038 | One Glyph laid out per Cell; never batch runs (ligatures shift columns) | retained CONS-021 One Glyph per Cell at the size it is drawn | | source-stated |
| ADR 0038 | Square-Cell fit and letterboxing in `grid_viewport.rs` | superseded | ADR 0045 (fit, letterbox) and ADR 0079 (rectangle) | source-stated |
| ADR 0038 | `Arc::make_mut` mechanism, survey of `horizon`, hand-built glyph Mesh and `set_zoom_factor` rejected; egui-0.36.2 line citations (workspace now pins 0.37.0) | historical | reasoning; `set_zoom_factor` later adopted by 0058 | source-stated |
| ADR 0038 | Does not reopen ADR 0005 / reachability | historical | 0049 ended the deferral | source-stated |
| ADR 0040 | `Paint::derive(FramePaint)` answers a per-Cell value with no `Orcvs` and no `egui::Context`; carries no geometry; a separate step builds Shapes | retained CONS-022 Paint is a geometry-free value | | source-stated |
| ADR 0040 | Paint covers only the Positions the viewport draws (`VisiblePositions`); cull changes no answer | retained CONS-023 Paint covers the drawn Positions only | | source-stated |
| ADR 0040 | `Paint::cursor()` is `Option<Position>`; `RenderFrame::derive` stays `pub(crate)`; `show_source` answers only its click | retained CONS-022 (seam constraint) | repository rule "no test-only inputs" | source-stated |
| ADR 0040 | Backgrounds coalesce into column runs drawn as epaint rects; single background Mesh refused | retained CONS-024 Backgrounds coalesce into runs | 0078 keeps the refusal for fills | source-stated |
| ADR 0040 | Colour assertions need no `Context`; cost claimed by counting, not timing | retained CONS-022 (test obligation) | | source-stated |
| ADR 0040 | Scale argument to `SourceShapes::new` | superseded | ADR 0058 | source-stated |
| ADR 0040 | "Five ordered groups"; per-Cell stroked rectangle count | superseded | ADR 0078 edges (proposed, built) replace per-Cell strokes | proposed |
| ADR 0040 | Screenshot tests, publishing `RenderFrame::derive`, accesskit colour fields | historical | rejected alternatives | source-stated |
| ADR 0044 | A Render Frame Cell carries the parser's Claim (`cells`, `token`, `atom`), not scalar projections; `parent` does not cross; `bound()` removed | retained CONS-025 A Render Frame Cell carries the parser's Claim | refined by 0052 | source-stated |
| ADR 0044 | Pending decided per slot by reading Cell contents in Paint | superseded | ADR 0052 (`written` on the Claim) | source-stated |
| ADR 0044 | Paint decision (colour per Token, tint, Cursor precedence) stays in console | retained CONS-027 Paint decisions belong to the console | | source-stated |
| ADR 0044 | Result destination not on the Claim | redirected | LANG/EVAL (syntax-highlighting 05/10) | source-stated |
| ADR 0050 | Language Map answers finished Paint facts per Cell | superseded | ADR 0052 (rejected on benchmarks) | source-stated |
| ADR 0051 | A Cell carries facts; console draws through channels; theme is the mapping; `style.rs` does not choose channels in control flow | retained CONS-060 Facts are drawn through channels the Theme chooses | 0053 says framing stands | source-stated |
| ADR 0051 | base16 syntax namespace, affordance named keys, storage of choice + per-scheme overrides, chrome to `Visuals`, Okabe–Ito default | superseded | ADR 0053 | source-stated |
| ADR 0052 | Claim gains `written`, computed once per Claim while the Render Frame builds Claims from the revision's bytes; space counts as blank; no per-Cell walk, no pointer-keyed cache | retained CONS-026 A Claim answers whether its slot is written | | source-stated |
| ADR 0052 | Language Map keeps only parser Claims, no whole-Grid Paint view; no revision rebuild computes `written` | retained CONS-026 | | source-stated |
| ADR 0052 | Output Portal fitting stays on `SourceRevision`; first-blank-Cell and gutter unchanged | redirected | EVAL (Output Portal Reservation) | proposed |
| ADR 0052 | Side-table, map-cache alternatives; benchmark figures | historical | | source-stated |
| ADR 0045 | Source View is a bounded space at the Source's own Cell; resize changes how much shows, never Cell size | retained CONS-030 The Source View shows the Source at its own Cell size | 16 pt Cell amended by 0079 | source-stated |
| ADR 0045 | Pan bounded by the Grid; smaller axis rests at top-left; resize/zoom settle back inside | retained CONS-031 Pan stops at the Grid's edges | margin added by 0047, removed by 0084 | source-stated |
| ADR 0045 (+2026-10-23) | Pan gestures: wheel/two-finger scroll, middle-drag, Alt+primary drag, Minimap; Pan never moves the Cursor | retained CONS-032 Pan gestures | finger drag added by 0102 | source-stated |
| ADR 0045 | Cursor move that would leave the view Pans just far enough | retained CONS-033 Cursor follow | | source-stated |
| ADR 0045 | Keyboard Zoom of Cell size; command-shift whole-UI zoom; 16 pt Cell for whole-point steps | superseded | ADR 0058 (zoom); ADR 0079 (Cell) | source-stated |
| ADR 0045 | Space refused as drag modifier; ScrollArea, infinite canvas, fitted canvas rejected; stored Source keeps its Grid | historical | rejected / superseded by 0054 | source-stated |
| ADR 0046 | Region: rectangle from anchor to live end, always exists, Cursor one Cell at the live end except after command `A` | retained CONS-050 Region model | CONTEXT Region | source-stated |
| ADR 0046 | Primary press/drag/click/Shift-click; drag past edge scrolls ≤ one Cell a frame, never past Grid | retained CONS-051 Pointer Region gestures | | source-stated |
| ADR 0046 | Shift+arrow extends; bare arrow collapses; command `A` spans Grid, Cursor stays; Escape collapses | retained CONS-052 Keyboard Region commands | | source-stated |
| ADR 0046 | Typing writes at Cursor and collapses; command Enter + char fills; Backspace/Delete empty a multi-Cell Region; copy/cut/paste rules | retained CONS-053 A Region is written as a block | | source-stated |
| ADR 0046 | Region lives beside the Cursor in `Orcvs`, reaches console on the Render Frame, not stored; multi-Cell Region tinted | retained CONS-054 Region is not stored with the Source | | source-stated |
| ADR 0046 | Cursor-with-extent, typing-fills, Orca alternatives | historical | | source-stated |
| ADR 0047 | Default Grid 128 by 80; stored Source keeps its Grid | superseded | ADR 0054 | source-stated |
| ADR 0047 | Two-Cell margin, Pan reach past edges | superseded | ADR 0084 | source-stated |
| ADR 0047 | Default window 1024 by 640 points | retained CONS-035 Default window size in points | Cell count amended by 0079 | source-stated |
| ADR 0047 | Grab hand while Alt held; grabbing hand during Alt-drag or middle-drag Pan | retained CONS-034 Pointer shows drag Pan | | source-stated |
| ADR 0048 | Source gets every key while no control holds egui focus and no popup is open; answer latched from the previous frame | retained CONS-055 The Source takes the keys no control holds | | source-stated |
| ADR 0048 | Keys a control took disarm a pending fill | retained CONS-056 Keys elsewhere disarm a fill | | source-stated |
| ADR 0048 | Nothing in the console area is focusable; Tab belongs to the Source (Sector step) and never moves egui focus at the same press | retained CONS-057 The console area stays out of the Tab order | | source-stated |
| ADR 0048 | Grid-holds-focus, widget per Cell, named-control list | historical | | source-stated |
| ADR 0053 | One Theme styles Source Grid and chrome together; never themed separately | retained CONS-061 One Theme styles the whole console | | source-stated |
| ADR 0053 (+10-10, +10-25) | Settings name a dark and a light Theme by identity in `~/.orcvs/config.toml` `[theme]`; read once at startup; console never writes; absent key selects `grayscale-dark`/`grayscale-light` | retained CONS-062 Themes are selected by identity in the config file | | source-stated |
| ADR 0053 (+10-31) | Mode is one icon toggle at the right of the top bar (`☾`/`☼`), egui `ThemePreference`, default `System`, no return to System | retained CONS-063 Appearance mode is one toggle | three-button control (10-10) superseded | source-stated |
| ADR 0053 | Theme pickers, `Theme →` menu items, reset to defaults | superseded | 0053 amendment 2026-10-10 | source-stated |
| ADR 0053 | No overrides; custom Theme inherits exactly one built-in, lists changes; no chains; appearance follows parent, conflicting declaration rejected; no in-app editor | retained CONS-064 Custom Themes inherit one built-in | | source-stated |
| ADR 0053 | Native Theme files in `~/.orcvs/themes/`, scanned each startup; no watcher, no XDG | retained CONS-065 Native Theme files are read at startup | | source-stated |
| ADR 0053 | Identity is filename stem; built-in identities reserved; duplicate stems all refused; extension case-insensitive, stem case kept | retained CONS-066 A Theme's identity is its filename stem | | source-stated |
| ADR 0053 | Missing/malformed/conflicted selection uses appearance default, reports, keeps saved selection | retained CONS-067 Fallback never replaces a selection | | source-stated |
| ADR 0053 | TOML only, `format = "orcvs-theme"`, `version = 1`, strict derived document; unknown keys and out-of-range widths reject whole; error names key and line, 1024-byte cap | retained CONS-068 Theme documents are strict TOML | `schema.md` holds catalogue | source-stated |
| ADR 0053 | Planned controls (colours, opacity, Grid/Cell colours, borders, widths); width bounds 0–1 Grid, 0–2 chrome; display points, not scaled by zoom | retained CONS-069 Theme controls and width bounds | | source-stated |
| ADR 0053 | Opaque window backdrop; separate Grid background; uniform transparent `cell.background`; named role backgrounds; per-channel fact keys | retained CONS-070 Theme layers composite over an opaque backdrop | | source-stated |
| ADR 0053 (+10-20) | Optional `cursor.background`, `region.cursor.background`, `cursor.foreground`, `region.cursor.foreground`: omission / `none` / transparent | retained CONS-071 Optional Cursor colours | | source-stated |
| ADR 0053 (+10-22) | Fixed painting precedence; Bang keeps own paint inside a Portal | retained CONS-072 Painting precedence is fixed | | source-stated |
| ADR 0053 | Animated stroke variation 0.45–1.25× | superseded | ADR 0079 (effect removed) | source-stated |
| ADR 0053 (+10-01, +10-23, +10-25) | Contrast validation of painted text states at 4.5:1; Comment measured not held; no accepted exceptions | retained CONS-073 Contrast gate | Sequence exception and dark-failures table superseded | source-stated |
| ADR 0053 (+10-10, +10-25) | Test-only colour-vision gate over built-ins (ΔE00 5.0, protan/deutan); neutral-grey pairs exempt; close glyph pairs must differ in fill or border | retained CONS-074 Colour-vision gate | figures for retired Themes historical | source-stated |
| ADR 0053 (+10-25) | Only `grayscale-dark` and `grayscale-light` ship; retired identity is missing | retained CONS-075 Only the grayscale defaults ship | | source-stated |
| ADR 0053 | A Theme decides look, never motion | retained CONS-076 A Theme never moves anything | 0079 removes motion entirely | source-stated |
| ADR 0053 status | Web Theme import disabled for v1; web has built-ins only | retained CONS-077 The web console runs on the built-ins | | source-stated |
| ADR 0053 | "Web Themes are imported as files" design | open | deferred design, not built | source-stated |
| ADR 0053 | Base16 import deferred; fonts, spacing, radii out of scope | historical | out of scope | source-stated |
| ADR 0053 | JSON/YAML rejected; `SourcePaintSettings` replaced | historical | | source-stated |
| ADR 0058 | One zoom, egui's: command `+`/`=`, `-`, `0`, step 0.1, range 0.2–5.0; View menu Zoom In/Out/Reset | retained CONS-040 Zoom is egui's whole-UI zoom | | source-stated |
| ADR 0058 | Cells sized in points; zoom answers Pan clamp and Cursor follow as a resize; zoom after viewer Panned Cursor away keeps Pan | retained CONS-030, CONS-033 | | source-stated |
| ADR 0058 | Zoom persists in egui memory; no console key; non-persistence build starts 1.0 | retained CONS-041 Zoom persists in egui memory | | source-stated |
| ADR 0058 | Web: browser page zoom only; no web View zoom items | retained CONS-042 The web zooms with the browser | | source-stated |
| ADR 0058 | Command-shift withdrawn; bare `+ - = 0` are Source characters; pinch and command-wheel never zoom | retained CONS-043 Zoom never takes Source characters or pinch | | source-stated |
| ADR 0058 | 16 pt square Cell, 11.5 pt Glyph | superseded | ADR 0079 (proposed; inline-amended into 0058) | source-stated |
| ADR 0078 (proposed) | Each Cell edge drawn once, highest-ranked fact either side; ranking; width zero withdraws; colour composited once; seams over edge except selection | retained CONS-122 Each Cell edge is drawn once (proposed ADR, built) | see Conflicts O1, O2 | proposed |
| ADR 0078 (proposed) | Paint decides edge facts; shapes step places on whole pixels with junction rule, one Mesh | retained CONS-123 Edge lines on whole physical pixels | | proposed |
| ADR 0079 (proposed) | Cell is a rectangle, 12 by 18 pt default, `[cell] width/height` 8–64, per-axis snap; Glyph = min(0.75 h, w / 0.62) | retained CONS-120 Source Cells are configurable rectangles (proposed ADR, built) | see O1 | proposed |
| ADR 0079 (proposed) | Default window stays 1024 by 640 pt (≈85 × 35 Cells) | retained CONS-035 | | proposed |
| ADR 0079 (proposed) | Cursor Effect removed; still outline; `[cursor_effects]` and `cursor.area` refused | retained CONS-121 The Cursor has a still outline | | proposed |
| ADR 0080 | Minimap press centres, drag carries with grab offset, one pending-Pan before the clamp; never moves Cursor or Region | retained CONS-090 The Minimap is a Pan gesture | | source-stated |
| ADR 0080 | Picture cached in console keyed by revision, Theme, appearance; nearest-filtered texture; not on Language Map or `orcvs`; benchmark with a floor | retained CONS-091 The Minimap picture is a console cache | | source-stated |
| ADR 0080 | One Theme key (colour + width, chrome bounds) styles the outline | retained CONS-092 The outline has its own Theme key | spelled `minimap.source_view` by 0098 | source-stated |
| ADR 0080 | Docked beside Source; picture includes margin | superseded | ADR 0081 (placement), ADR 0084 (margin) | source-stated |
| ADR 0081 | Floating `Area`; layer is the bordered picture rect; no pointer taken outside it; not clipped or scaled below 1 px/texel | retained CONS-093 Minimap placement and pointer footprint | | source-stated |
| ADR 0081 | Window bottom-right corner; half window / below menu bar sizing | superseded | ADR 0096 | source-stated |
| ADR 0096 | Corner is Source's bottom-right, above the bottom panel; floats over Source only | retained CONS-093 | | source-stated |
| ADR 0096 | One physical pixel per texel at every scale; hidden if it and border do not fit half Source width and whole height; View menu keeps choice | retained CONS-094 Minimap sizing | | source-stated |
| ADR 0098 | Term is Minimap: View item, label "Source minimap", keys `minimap.source_view(.width)`, `[minimap]`, bench `paint_minimap/picture/256x256`; `overview.*` and `[overview]` not read | retained CONS-095 The old Overview names are not read | | source-stated |
| ADR 0082 | No code, test or bench depends on a particular Theme; fixtures via registry fallback; gates iterate registry built-ins | retained CONS-078 Nothing depends on a particular Theme | | source-stated |
| ADR 0083 | Native splash is the console's own borderless root window, grown to the console; logo fades over the Grid | retained CONS-080 The splash is the console's own window | | source-stated |
| ADR 0083 | Console stores content size, position, maximised/fullscreen under its own key; `persist_window` off; save during splash stores target geometry; size clamped to monitor | retained CONS-081 The console keeps its own window geometry | | source-stated |
| ADR 0084 | Source View shows the Grid and nothing past it; Pan stops flush; Minimap extent is the Grid | retained CONS-031 | | source-stated |
| ADR 0089 | Panel opens with `P` and `R` Cells, one Cell apart, before **B**, on Cell columns; `button`/`button.pressed` roles; `P` = Space, `R` = command-shift-R; every console incl. small shows the Panel | retained CONS-100 Play and Rewind are Panel Cells | | source-stated |
| ADR 0089 | Page resumes the Built-in Synth's audio context on each gesture; media playback request | redirected | SITE / PLAY | source-stated |
| ADR 0092 | One console Playback status routes each frame's diagnostics; MIDI selection and network keep only their own | retained CONS-101 One status routes Playback failures | | source-stated |
| ADR 0092 | Clock/start/retune failures have own status; latest shown until next run begins | retained CONS-102 Engine failures last until the next run | | source-stated |
| ADR 0092 | Observation publishes `midi_failed`, cleared by a delivery or install | redirected | PLAY (engine observation); console reads it under CONS-101 | proposed |
| ADR 0092 | No build branches on MIDI; network failures also reported to developer console | retained CONS-103 No build branches on MIDI presence | | source-stated |
| ADR 0095 | Output list: labelled MIDI and UDP/OSC sections; unusable section hidden; `destination_presentation` gone | retained CONS-104 Output list sections | | source-stated |
| ADR 0095 | Closed field names live outputs joined by ` · `, `None` if none; failure never in field | retained CONS-105 The closed Output field names live outputs | | source-stated |
| ADR 0095 | Status line beneath Panel row | superseded | ADR 0100 | source-stated |
| ADR 0100 | Each standing failure is a Toast top-right beneath menu bar, labelled `MIDI`/`UDP/OSC`/`ENGINE`, dismissable, fades after `[toasts] seconds`; `egui::Area`, no dependency | retained CONS-106 Failures are Toasts | | source-stated |
| ADR 0100 | **O** readout drawn in error colour while any message stands | retained CONS-107 O signals a standing failure | | source-stated |
| ADR 0100 (+2026-11-12) | Info: Cells of what the Cursor's Cell belongs to painted as the Source with Cursor outline, labels beneath; entries two Cells apart; answer Position not shown; diagnostic at Cursor; follows Cursor not pointer | retained CONS-108 Info describes the Cursor's Cell | | source-stated |
| ADR 0100 (+2026-11-12) | Info is a View toggle, shown by default, `[info] open`, ⌘I native; menu change not saved; Panel laid on Cell rows; Panel height independent of messages | retained CONS-109 Info and Panel layout | | source-stated |
| ADR 0100 | "Cursor Line" name | superseded | 2026-11-12 amendment: Info | source-stated |
| ADR 0102 | Finger drag Pans; tap selects; long press (0.8 s) selects a Region; mouse and pen keep 0045 gestures | retained CONS-110 Finger gestures | | source-stated |
| ADR 0102 | Finger press while Source holds keys reports Cursor Cell as IME rect; mouse press withdraws | retained CONS-111 A finger asks for the keyboard | | source-stated |
| ADR 0102 | Committed text one Cell per char; `DeleteSurrounding` = Backspaces; composition writes nothing; autocorrect/spellcheck off; `interactive-widget=resizes-content` | retained CONS-112 Phone keyboard input is Source input | viewport meta in `console/index.html` | source-stated |
| ADR 0102 | Phone Cell 16 by 24 (amends 0091) | redirected | SITE | source-stated |
| CONTEXT "Orcvs (running instance)" | Output device choice is console configuration | retained CONS-012 | not cited in Rust | source-stated |
| CONTEXT "Cursor" | One Cell, still outline on its edges, no clamping of its own | retained CONS-050, CONS-121 | not cited | source-stated |
| CONTEXT "Region" | Matches 0046 | retained CONS-050 | not cited | source-stated |
| CONTEXT "Sector" | Tab / Shift Tab step the Cursor a Sector along its row; console presentation, no language meaning | retained CONS-057 | ruling in `.scratch/sector-navigation/issues/01`, no ADR; not cited | source-stated |
| CONTEXT "Sector Seam" | Composited over each edge's line except the selection outline's | retained CONS-122 | from 0078 (proposed); not cited | source-stated |
| CONTEXT "Render Frame" | Reads the Source, never advances Playback, not a Tick | redirected | EVAL (Render Frame vs Tick) | proposed |
| CONTEXT "Paint" | Per-Cell decision, edge rule, no geometry | retained CONS-022, CONS-122 | not cited | source-stated |
| CONTEXT "Theme" | Matches 0053; "follows the OS or holds one" | retained CONS-061..064 | see O4; not cited | source-stated |
| CONTEXT "Source View" / "Pan" | Bounded; Pan list wheel, middle-drag, Alt-drag, Minimap | retained CONS-030..032 | omits finger drag, see O3; not cited | source-stated |
| CONTEXT "Minimap", "Zoom", "Panel", "Info", "Toast", "Readout" | Match ADRs 0096/0058/0100; Panel is static | retained CONS-093, CONS-040, CONS-106..109 | not cited | source-stated |
| CONTEXT "BPM" | Performer's display of the Tick Grid's period | retained CONS-099 Panel Readouts | Tick Grid itself PLAY | proposed |
| CONTEXT "Run Clock" | Held while stopped, zero after Rewind | redirected | PLAY | proposed |
| `.scratch/menu-structure/spec.md` | Settings config-file only, no settings UI, no reload; native document model (title, unsaved marker, ask before discarding on New/Open/Quit/close); web file I/O deferred | retained CONS-130 Native document model and config-file settings | no ADR records it; 0053 amendments cover the Theme part | source-stated (.scratch) |
| `.scratch/menu-structure/spec.md` | Three-state mode control; two-state switch rejected | superseded | ADR 0053 amendment 2026-10-31 | source-stated |
| `.scratch/menu-structure/spec.md` | Source File format | redirected | LANG (CONTEXT Source File) | proposed |
| `console/src/config.rs` / `.scratch/cursor-line-and-toasts`, `docked-diagnostics` | `[diagnostics] open`, `[minimap] open`, `[toasts] seconds` 1–60; per-key fallback with notice | retained CONS-131 Console settings keys | partly 0079/0100; rest .scratch only | proposed |
| `.scratch/source-undo/spec.md` | Command Z undo (needs-triage); 0046 relies on "Orcvs has no undo" | open | not decided | source-stated |

## Evidence and gaps

Test assertions marked (i) were read; others are inferred from the test name.

| Rule | Code | Tests | Status |
| --- | --- | --- | --- |
| CONS-010 | `orcvs/Cargo.toml` (no egui/eframe/winit) | none; compiler-enforced | built, no verified test (compiler is the gate) |
| CONS-011 | review-enforced | none | code differs: `orcvs/src/opts.rs` `CursorBloomRadius` and `RenderFrame::cursor_bloom_radius` remain in `orcvs` with no console reader after 0079 removed the effect; no open issue found (`render-frame-derivation/02` is wontfix on cost) |
| CONS-012 | `console/src/console_midi.rs`, `midi.rs`, `web_midi.rs` | `console_midi/loopback_tests.rs` | built, no verified test (boundary) |
| CONS-020 | `console/src/console/source_view.rs` `show_source_scene`, `grid_viewport.rs` | `console/tests.rs` `no_layer_carrying_the_source_grid_is_transformed` (i), `a_translation_moves_the_presented_grid_by_itself_at_any_device_scale` | built and covered |
| CONS-021 | `console/src/console/glyphs.rs`, `grid_viewport.rs` `glyph_size` | `a_glyph_is_laid_out_at_its_cells_own_size_at_every_device_scale`, `the_glyph_table_covers_the_printable_ascii_a_cell_can_hold` | built and covered (per-Cell layout not asserted against batching) |
| CONS-022 | `console/src/paint.rs` `Paint::derive_with_theme`, `FramePaint` | `paint.rs` tests (67, no `Context`); `the_cursor_reaches_the_paint_of_a_cell_and_never_its_geometry` | built and covered |
| CONS-023 | `paint.rs`, `grid_viewport.rs` `visible_positions` | `the_draw_loop_paints_the_visible_range_rather_than_the_whole_source`, `the_visible_range_is_the_shown_positions_and_one_cell_more_each_way` | built and covered |
| CONS-024 | `paint.rs` `BackgroundRun` | `a_background_run_covers_exactly_the_cells_it_replaces`, `a_culled_row_covers_exactly_the_cells_its_runs_replace` | built and covered |
| CONS-025 | `orcvs/src/render_frame.rs` `RenderCell::claim`, `orcvs/src/source/language_map.rs` `Claim` | `render_frame.rs` `a_valid_function_with_unbound_operands_shares_each_parsers_claim`, `a_comment_is_one_claim_that_records_no_atom` | built and covered |
| CONS-026 | `LanguageMap::claims_by_cell`, `Claim::written` | `every_cell_of_a_claim_reads_the_written_answer_its_claim_was_built_with`, `a_partly_written_operand_is_invalid_on_every_cell` | built and covered |
| CONS-027 | `console/src/style.rs`, `paint.rs` | `style.rs` precedence tests | built and covered |
| CONS-030 | `source_view.rs`, `grid_viewport.rs` | `a_resize_keeps_the_cell_size_and_shows_more_or_less_of_the_source`, `the_default_window_presents_the_grid_at_its_own_scale` | built and covered |
| CONS-031 | `source_view.rs` clamp | `clamp_pan_pins_a_smaller_source_at_the_origin_and_a_larger_one_to_its_edges`, `the_extent_is_exactly_the_grid`, `a_far_edge_pan_leaves_no_gap_past_a_snapped_grid` | built and covered |
| CONS-032 | `source_view.rs` | `wheel_pans_a_larger_source_to_its_edges_and_a_smaller_one_nowhere`, `alt_held_with_a_primary_drag_pans_by_exactly_what_the_pointer_moved`, `a_middle_drag_that_starts_on_a_cell_still_pans_the_source`, `an_alt_drag_and_a_middle_drag_leave_the_region` | built and covered |
| CONS-033 | `source_view.rs` follow | `source_view.rs` `a_cursor_move_that_would_leave_it_outside_the_view_pans_to_show_it`, `a_resize_after_a_pan_away_from_the_cursor_keeps_the_pan`, `a_cursor_follow_to_an_edge_cell_brings_it_flush_with_the_console_edge` | built and covered |
| CONS-034 | `source_view.rs` | `the_pointer_shows_a_grab_hand_for_alt_and_a_grabbing_hand_while_a_drag_pans`, `the_pointer_shows_no_grab_hand_where_alt_offers_no_pan` | built and covered (code adds: no hand where no Pan is possible) |
| CONS-035 | `console/src/main.rs` | `the_default_window_is_a_fixed_size_in_points` | built and covered |
| CONS-040 | `console/src/console/menu_bar.rs` `zoom_menu_buttons` | `command_zoom_chords_change_egui_zoom_and_never_the_source`, `the_view_menu_zooms_egui_as_its_chords_do` | built and covered |
| CONS-041 | egui memory | `a_zoom_survives_a_save_and_a_restart` | built and covered |
| CONS-042 | `menu_bar.rs` (wasm branch, line ~341) | `a_browser_zoom_that_would_leave_the_cursor_outside_the_view_pans_to_show_it`; no test that the web menu lacks zoom items | built, no verified test |
| CONS-043 | `console/src/console/input.rs` | `pinch_and_command_wheel_do_not_zoom`, `command_zoom_chords_change_egui_zoom_and_never_the_source` | built and covered |
| CONS-050..053 | `orcvs/src/app.rs`, `orcvs/src/region.rs` | `app.rs` `shift_arrows_extend_the_region_and_a_bare_arrow_collapses_it`, `select_all_spans_the_grid_and_collapse_returns_to_the_cursor`, `command_enter_then_a_character_fills_the_region`, `paste_writes_from_the_top_left_clipped_and_spans_what_landed`; `console/tests.rs` `a_primary_drag_spans_a_region_that_release_keeps`, `a_drag_past_the_edge_scrolls_faster_the_further_out_and_stops_at_the_grid` | built and covered |
| CONS-054 | `app.rs` `region()`, `render_frame.rs` `region()`; persistence stores Source only | `render_frame_carries_the_region_it_was_derived_for`; `persistence.rs` `a_save_writes_the_source_and_no_settings` | built, no verified test that Region is unstored |
| CONS-055 | `console/src/console/input.rs` `latch_keyboard_owner` | `a_focused_menu_item_keeps_region_and_clipboard_commands_from_the_source`, `typing_with_a_menu_open_leaves_the_source_unwritten`, `escape_with_a_menu_open_closes_it_and_keeps_the_region` | built and covered |
| CONS-056 | `app.rs` `disarm_fill` | `keys_the_bpm_field_took_disarm_a_fill_armed_before_it`, `a_fill_disarmed_for_input_elsewhere_types_the_next_character` | built and covered |
| CONS-057 | `input.rs`, `app.rs` | `tab_never_focuses_the_console_area_the_source_is_shown_in`, `tab_with_a_menu_open_moves_focus_and_not_the_cursor`, `tab_and_shift_tab_move_the_cursor_by_sector_through_the_source_input_path` | built and covered |
| CONS-060 | `style.rs` `CellColours`, `theme.rs` | `an_invalid_operand_draws_diagnostic_but_keeps_its_declared_tint`, `a_transparent_diagnostic_foreground_reveals_the_declared_role` | built and covered |
| CONS-061 | `style.rs` `style()`, `install` | `the_mode_switches_source_and_chrome_together`, `an_os_appearance_change_switches_source_and_chrome_together`, `chrome_colours_change_with_the_resolved_theme` | built and covered |
| CONS-062 | `console/src/config.rs`, `theme_selection.rs` | `config.rs` `every_key_is_read`, `an_absent_key_is_its_default`; `theme_selection.rs` `a_default_selection_presents_each_appearances_built_in_and_raises_no_notice`; `no_menu_offers_a_setting` | built and covered |
| CONS-063 | `console/src/console/menu_bar.rs` mode toggle | `the_mode_toggle_flips_the_appearance`, `a_restored_mode_is_the_one_the_toggle_shows`, `console_new_leaves_a_fresh_context_on_the_system_preference` | built and covered |
| CONS-064 | `console/src/theme_document.rs` resolve | `a_custom_theme_cannot_chain_from_another_custom_theme`, `appearance_mismatch_is_rejected`, `omitted_properties_inherit_from_the_parent` | built and covered |
| CONS-065..067 | `console/src/theme_registry.rs` | `theme_registry.rs` `duplicate_identities_conflict_whatever_the_enumeration_order`, `a_reserved_identity_is_refused_and_never_replaces_the_built_in`, `a_missing_selected_file_falls_back_and_returns_when_the_file_does`, `stems_that_differ_only_in_case_are_two_identities` | built and covered |
| CONS-068 | `theme_document.rs` | `an_unknown_style_property_lists_the_valid_names`, `the_format_marker_must_be_orcvs_theme`, `only_a_toml_file_is_a_theme_file`, `a_long_single_line_document_yields_a_bounded_message` | built and covered |
| CONS-069 | `theme.rs` `GridWidth`, chrome widths | `widths_accept_integers_and_floats_within_their_bounds`, `grid_and_sector_widths_stay_fixed_display_points_across_device_scales`, `a_zoomed_console_paints_whole_pixel_cells_at_the_themes_widths` | built and covered |
| CONS-070 | `theme_document.rs`, `style.rs` | `resolve_rejects_a_nonopaque_window_background`, `explicit_background_alpha_composites_over_the_cell_base`, `a_custom_themes_transparent_or_translucent_grid_composites_over_the_window_backdrop` | built and covered |
| CONS-071 | `theme_document.rs` optional colours | `optional_colours_are_omitted_cleared_or_a_supplied_transparent_colour`, `none_is_refused_for_every_other_colour`, `a_theme_document_paints_the_region_cursor_foreground_it_sets` | built and covered |
| CONS-072 | `style.rs` | `a_bang_keeps_its_own_paint_inside_every_portal_role`, `a_bound_function_spelling_wins_over_output_portal`, `region_fallback_only_reaches_a_fully_transparent_role_background` | built and covered |
| CONS-073 | `console/src/contrast.rs` | `shipped_theme_gate`, `a_comment_below_the_floor_is_reported_and_does_not_fail`, `a_hidden_cursor_glyph_is_not_measured`; `theme_registry.rs` `a_theme_below_the_contrast_floor_is_loaded_and_its_report_shown` | built and covered |
| CONS-074 | `contrast/colour_vision.rs` | `shipped_theme_colour_vision_gate`, `simulation_leaves_neutrals_alone`, `the_colour_difference_matches_the_published_test_vectors` | built and covered |
| CONS-075 | `theme_registry.rs` built-ins | `a_selection_of_a_retired_built_in_falls_back_to_the_default_and_is_kept`, `a_theme_file_inheriting_a_retired_built_in_is_refused`, `every_built_in_chrome_matches_its_record` | built and covered |
| CONS-076 | no motion keys | `config.rs` `the_removed_glitch_keys_are_unknown` | built and covered |
| CONS-077 | web startup reads no theme dir | none found asserting absence of web import | built, no verified test |
| CONS-078 | `ThemeRegistry::built_in().fallback`, `ThemeIdentity::default_for` | gates iterate registry (`shipped_theme_gate`) | built; rule is review-enforced (0082 refuses a grep gate) |
| CONS-080 | `console/src/console/splash.rs` | `kittest_tests/splash.rs` `the_hold_ends_by_itself_and_grows_the_window_to_the_default_console`, `the_logo_fades_and_then_the_splash_is_gone` | built and covered |
| CONS-081 | `console/src/window_geometry.rs`, `main.rs` `persist_window: false` | `a_save_during_the_hold_stores_the_consoles_geometry`, `a_quit_during_the_hold_stores_the_consoles_geometry`, `the_size_fits_between_the_minimum_and_the_monitor` | built and covered |
| CONS-090 | `console/src/console/minimap.rs` | `a_press_outside_the_outline_centres_the_source_view_there_and_it_stays`, `a_drag_from_inside_the_outline_moves_the_source_view_by_the_drag_without_a_jump`, `a_press_on_the_minimap_answers_a_centre_and_leaves_the_source_view_to_apply_it` | built and covered |
| CONS-091 | `minimap.rs` cache; `console/benches/minimap.rs` `paint_minimap` | `the_minimap_uploads_its_picture_only_when_the_source_or_appearance_changes`, `another_theme_draws_another_picture` | built and covered |
| CONS-092 | `theme_document.rs` `ColorKey::MinimapSourceView` | `the_minimap_outlines_the_source_view_where_a_pan_leaves_it` | built and covered |
| CONS-093 | `minimap.rs` | `the_minimap_starts_pinned_in_the_sources_bottom_right_corner_and_the_view_menu_toggles_it`, `the_picture_is_one_physical_pixel_per_texel_in_the_sources_corner`; no test that a press beside the picture reaches the Source | built and covered (pointer footprint not verified) |
| CONS-094 | `minimap.rs` | `the_minimap_hides_where_one_pixel_per_texel_does_not_fit_in_the_source`, `a_hidden_minimap_returns_at_its_usual_size_once_the_window_holds_it` | built and covered |
| CONS-095 | `theme_document.rs` | `the_overview_keys_are_refused`, `config.rs` `an_unknown_key_is_a_notice_and_leaves_the_known_keys_applied` | built and covered |
| CONS-100 | `console/src/console/panel.rs` | `play_and_rewind_are_source_cells_one_cell_apart`, `the_play_cell_plays_and_stops_playback_as_space_does`, `the_rewind_cell_rewinds_a_paused_playback_to_tick_zero`, `the_site_suites_click_on_p_lands_on_the_play_cell` | built and covered |
| CONS-101..103 | `console/src/playback_status.rs` | `each_output_failure_goes_to_its_own_destination`, `an_engine_failure_is_shown_until_the_next_run_begins`, `a_rewind_while_playing_begins_a_new_run`, `a_midi_failure_is_shown_until_the_observation_clears_it` | built and covered |
| CONS-104 | `panel.rs` Output list | `the_output_list_has_a_section_for_each_output_type` (i; native fake MIDI) | built and covered (web/no-MIDI hiding not verified) |
| CONS-105 | `panel.rs` | `the_output_list_has_a_section_for_each_output_type` (i, value `First · 127.0.0.1`) | built and covered |
| CONS-106 | `console/src/console/toasts.rs` | `a_failure_floats_as_a_toast_in_the_top_right_corner`, `a_toast_lasts_the_configured_seconds`, `a_dismissed_toast_stays_gone_until_its_reporter_clears` | built and covered |
| CONS-107 | `panel.rs` (~line 509) | `a_toast_fades_after_its_seconds_and_o_keeps_the_failure` | built and covered |
| CONS-108 | `console/src/info.rs` | `info.rs` tests; `kittest_tests/info.rs` `info_describes_the_function_under_the_cursor`, `info_entries_start_on_cell_columns_two_cells_apart` | built and covered |
| CONS-109 | `info.rs`, `input.rs`, `config.rs` | `only_a_fresh_command_i_toggles_info`, `hiding_info_gives_its_rows_back_to_the_source`, `the_panel_lays_its_rows_on_the_grids_cells`, `the_panel_keeps_its_height_whatever_the_messages` | built and covered ("menu change not saved" not verified) |
| CONS-110 | `source_view.rs` `touch_pressed` | `a_finger_drag_pans_the_source_and_leaves_the_region`, `a_finger_held_then_dragged_spans_a_region_without_panning`, `a_finger_tap_selects_the_cell_under_it` | built and covered |
| CONS-111 | `input.rs` `ask_for_touch_keyboard` | `a_finger_tap_asks_for_the_keyboard_and_a_mouse_click_does_not`, `a_finger_tap_outside_the_source_does_not_ask_for_the_keyboard` | built and covered |
| CONS-112 | `input.rs`; `console/src/web_startup.rs` (autocorrect off); `console/index.html` viewport | `a_phone_keyboard_writes_a_cell_per_character_and_deletes_by_backspace` | built and covered (composition-writes-nothing not verified) |
| CONS-120 | `grid_viewport.rs` `CELL_SIZE`, `glyph_size`; `config.rs` `[cell]` | `the_source_cell_is_twelve_by_eighteen_points`, `a_glyph_is_three_quarters_of_its_cells_height_or_what_its_width_fits`, `every_cell_is_a_whole_number_of_physical_pixels`, `an_out_of_range_or_mistyped_value_falls_back_alone` | built and covered (ADR proposed) |
| CONS-121 | Cursor Effect code absent; `persistence.rs` retired keys | `the_removed_glitch_keys_are_unknown`, `the_cursor_outline_is_one_box_the_same_on_all_four_sides` | built and covered (ADR proposed) |
| CONS-122 | `paint.rs` `EdgeRank`, `EdgeLine`, `Edges` | `an_empty_grid_draws_each_grid_line_once`, `neighbouring_diagnostic_cells_share_one_line_composited_once`, `a_spanning_regions_outline_owns_its_perimeter_and_its_cursor_cell_ranks_next`, `a_zero_width_border_falls_to_the_fact_below_it` | built and covered (ADR proposed). Gap: `cell.selection.border.width` still accepted and inert — `.scratch/cell-edges/issues/07` (needs-triage); `cell-edges/02` still "claimed" and `06` ready-for-human though code is built |
| CONS-123 | `console/src/console/shapes.rs`; `console/benches/paint.rs` `paint_shapes` | `a_one_pixel_crossing_is_drawn`, `a_narrow_border_keeps_its_width_through_a_wider_grid_lines_junction`, `a_grid_line_under_one_pixel_covers_one_pixel_faded_by_its_width` | built and covered (ADR proposed) |
| CONS-130 | `console/src/console/files.rs`, `config.rs` | `the_window_title_names_the_source_and_marks_unsaved_changes`, `closing_the_window_asks_before_discarding_unsaved_changes` | built and covered |
| CONS-131 | `config.rs` | `a_toast_lasts_the_seconds_the_file_says`, `the_minimap_and_the_diagnostics_open_as_the_file_says` | built and covered |

### Gaps (one line each)

- CONS-011: `CursorBloomRadius` / `RenderFrame::cursor_bloom_radius` linger in `orcvs` with no reader since 0079; no open issue.
- CONS-122: `cell.selection.border.width` accepted but inert, contrary to 0078's "border_width of zero no longer hides anything" — `.scratch/cell-edges/issues/07` (needs-triage).
- CONS-122: `.scratch/cell-edges/issues/02` status "claimed" and `06` "ready-for-human" although the behaviour is built; `08` (role borders, needs-triage) appears superseded by shipped `source.*.border` keys (`style.rs` `each_role_strokes_its_own_border`).
- CONS-042 / CONS-077 / CONS-054: built, no test asserts the absence (web zoom items, web Theme import, Region persistence).
- CONS-093: no test that a press beside the Minimap picture reaches the Source.
- CONS-104: web and no-MIDI section hiding not asserted.
- `.scratch/source-undo` (needs-triage): ADR 0046's "no undo" premise stays true; open, not a gap.

## Conflicts and open questions

**O1. Proposed ADRs 0078 and 0079 are built and already written into accepted sources.**
ADR 0078 and 0079 say "Status: proposed". Yet accepted ADRs carry inline amendments citing them
(0038 "amended by ADR 0079: a Cell is a width and a height"; 0047 "shows about 85 by 35 of the
default 12 by 18 point Cells"; 0053 "Cells are configurable rectangles, 12 by 18 points by
default", and the Cursor Effect removed; 0058's Status: "its 16 point Cell is amended by ADR
0079"). `CONTEXT.md` Paint ("the line each Cell edge shows ... highest-ranked fact"), Cursor ("a
still outline on its Cell's edges") and Sector Seam follow 0078/0079. The code builds both, and
the tests cover them (CONS-120..123). All `source-cell-size` issues are resolved. The sources
cannot settle whether the 16 pt square (0045/0058 bodies) or the 12 by 18 rectangle binds.
Recommendation: the snapshot states CONS-120..123 as current behaviour, labelled "proposed ADR,
built", and the run asks Toby to mark 0078 and 0079 accepted. If they are not accepted, the
accepted ADRs' inline notes and `CONTEXT.md` describe behaviour that has no binding decision.

**O2. (Resolved on drafting.) Edge order among Portal and Locked borders.** ADR 0078 defers to
ADR 0077, and ADR 0077 ¶ "A Cell's roles combine" does state the paint order: "The console paints
Output Portal over Input Portal over Locked". The inventory missed that sentence. The fill and glyph
precedence is binding as CONS-029. Because 0078 is proposed, the edge order is stated as observed
under "Proposed, built" (CONS-122), per the coordinator's instruction.

**O3. `CONTEXT.md` Pan lists the Pan gestures as if complete and omits the finger drag.** It says
"A Pan is made by wheel, middle-drag, Alt-drag, or a press or drag on the Minimap". ADR 0102 says
"A finger's drag Pans". Code and tests (`a_finger_drag_pans_the_source_and_leaves_the_region`)
follow 0102. Recommendation: the requirement follows 0102, and the glossary adds "a finger's
drag". If it is left as is, a reader of the glossary concludes that touch cannot Pan.

**O4. Return to following the OS.** ADR 0053's body and `CONTEXT.md` Theme say the settings hold
"whether the console follows the operating system's appearance or holds one of the two". The
2026-10-31 amendment says "no control returns it to following the operating system". 0053's
Status line does not list the 2026-10-31 amendment. Code does what the amendment says
(`the_mode_toggle_flips_the_appearance`). Recommendation: follow the amendment, state that
`System` is only the pre-first-press default, and note the omission from the Status line. This
is low severity: it is wording, not a contest between decisions.

## Fresh-reader questions

1. **"I want to ship a third built-in Theme. What must it pass, and which tests do I edit?"**
   Add its definition, registry entry and record in `console/src/theme.md`. Edit no test. The
   contrast gate (4.5:1 in every reachable painted text state; Comment is measured but not held;
   no accepted-exception list) and the colour-vision gate (ΔE00 5.0 for protan and deutan;
   neutral greys exempt; close glyph pairs must differ in fill or border) iterate the registry's
   built-ins. Sources: ADR 0053 amendments 2026-10-01, 10-10, 10-23 and 10-25; ADR 0082;
   `console/src/contrast.rs` `shipped_theme_gate`; `contrast/colour_vision.rs`
   `shipped_theme_colour_vision_gate`.
2. **"Drawing is slow. Can I cache each Cell's Paint facts on the Language Map?"** (crosses
   EVAL/LANG) No. ADR 0050 did that, and it was rejected on the benchmarks because it moved
   whole-Grid work into every revision rebuild. ADR 0052 has each Claim answer `written` once,
   while the Render Frame builds the Claims. The Language Map keeps only parser Claims. The
   Minimap's whole-Grid picture is cached in the console for the same reason (ADR 0080). Sources:
   ADRs 0050, 0052, 0080; `orcvs/src/source/language_map.rs` `claims_by_cell`.
3. **"How does a performer move around the Source on a laptop, and on a phone? Can they pinch to
   zoom?"** On a laptop: the wheel or a two-finger scroll, a middle-drag, Alt with a primary drag,
   or a press or drag on the Minimap. On a phone, a finger drag. Every Pan is bounded by the Grid
   and leaves the Cursor and the Region where they are. Pinch and command-wheel never zoom. Zoom
   is egui's command `+`/`-`/`0` on native, and the browser's page zoom on the web. Sources: ADRs
   0045 (with its amendment), 0058, 0080, 0084, 0102.
4. **"A MIDI clock failure shows as a MIDI problem. Where should it appear, and when does it
   clear?"** (crosses PLAY) Clock, start and retune failures are engine failures. One console
   Playback status routes them to their own status, and they show as a Toast labelled `ENGINE`.
   The latest one stands until the next run begins. While it stands, **O** is drawn in the error
   colour, even after the Toast fades. MIDI failures clear on `midi_failed`, which the engine
   publishes. Sources: ADRs 0092, 0100; `console/src/playback_status.rs`
   `an_engine_failure_is_shown_until_the_next_run_begins`.
5. **"Typing into a value box in a menu wipes my Grid. Which controls should keep the keys?"**
   Every focused control, and any open popup. The Source asks egui each frame and latches the
   answer from the previous frame; there is no list of named controls. Keys that a control took
   also disarm a pending command-Enter fill. Nothing in the console area takes focus. Sources:
   ADR 0048; `console/src/console/input.rs` `latch_keyboard_owner`;
   `a_focused_menu_item_keeps_region_and_clipboard_commands_from_the_source`.
6. **"Can a phone visitor to the site play an example?"** (crosses SITE) Yes. Every console,
   including the small one, shows the Panel's `P` and `R` Cells: `P` does what Space does, and
   `R` does what command-shift-R does. A gesture resumes the Built-in Synth's audio context. A
   finger press opens the keyboard. Phone Cells are 16 by 24 points (the page settings; SITE).
   Sources: ADRs 0089, 0102, 0091, 0086.

## Drafting map (docs/requirements/console.md)

Every retained row keeps its provisional ID as its final rule ID. There are no merges: rows citing
`CONS-050..053`, `CONS-061..064`, `CONS-065..067` or `CONS-101..103` land in those individual rules.

- **Moved to "Proposed, built" (not binding):** CONS-120, 121, 122, 123. CONS-035's Cell count
  (0079) is stated only under CONS-120.
- **New rules from rows other topics redirected here:**
  - CONS-029 Tick Portal roles combine, and one is painted (EVAL redirect, ADR 0077 ¶10–12).
  - CONS-099 Panel Readouts (PLAY redirect, ADR 0076 ¶6; CONTEXT BPM, Readout).
  - CONS-132 A stored Source of another shape is discarded (LANG redirect, ADR 0054).
  - CONS-133 Help menu reaches Learn and Reference (SITE redirect, `.scratch/site-as-console`).
- **Redirected rows folded into existing rules:**
  - CONS-027 takes ADR 0034 (empty operand tint, no character), 0061, 0069, 0085 and 0105 paint
    clauses (LANG and PLAY redirects).
  - CONS-100 takes ADR 0076 ¶7 (Command Shift R native only, no Playback menu).
  - CONS-101 takes ADR 0055 (Omitted maps to no message) and ADR 0059 ("Opening …").
  - CONS-131 takes ADR 0091's `[minimap]` and `[diagnostics]` settings (SITE redirect).
- **Not stated here:**
  - ADR 0088 `sound.magnitude`: the console Theme gains it only when the console draws the Sound
    Module, so SITE owns it.
  - ADR 0043's "presentation is not the Grid" is covered by CONS-030 and CONS-031.
- Follow-up: CONS-028 states the ADR 0052 Output Portal fit row (previously marked redirected to EVAL), using the fit rule from `.scratch/syntax-highlighting/issues/12`. CONS-079 states ADR 0088's `sound.magnitude` Theme colour.
- Audit fixes:
  - F2: CONS-028 now states the built highlight, which covers each value answer's Output Portal
    Reservation (root or nested), cited to real tests. ADR 0052's "fitted highlight on
    SourceRevision / first-blank-Cell / gutter" clause is superseded in effect by ADRs 0063 and
    0067, which leave no Sequence-capable root (basis: proposed). The code computes the highlight
    in `LanguageMap` and delegates through `SourceRevision`. The syntax-highlighting/12 citation is
    dropped (a bug report, not a ruling).
  - F3: CONS-076 keeps only "no Theme property controls motion". "Nothing animates" moves to
    CONS-121 (proposed). ADR 0053's animated-stroke clause is listed as built differently.
  - F6: CONS-130 is narrowed to read-once with no settings UI (ADRs 0053, 0091). CONS-131 is
    narrowed to the setting keys (ADRs 0091, 0100, 0098). The document model and generic per-key
    fallback (menu-structure spec, undated) move to Open decisions. CONS-133 is dated 2026-10-26
    (orcvs-site/13).
  - F9: anchors fixed (`eguis`, `cells`).
  - CONS-029 now carries ADR 0077's full retained-record conditions (anchor still claimed for the
    same Function; subtree unedited; Selected-descendant reads; lock conditions). EVAL-106 defers
    to it.
- CONS-079 is reduced to the console guarantee: the console has no `sound.magnitude` key and a Theme that sets it is refused as unknown. The colour rule is owned by SITE-038.
