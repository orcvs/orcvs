//!
//! The top bar's and View menu's layout: the gaps between menus and where
//! a toggle's box sits.
//!

use egui::Vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT as _, Queryable as _};

use super::super::{Console, DEFAULT_VIEW_SIZE};
use super::{menu_item, running_console};

///
/// The check boxes the last frame painted inside `item`: square rects the
/// size of egui's checkbox box.
///
fn boxes_in(harness: &Harness<'_, Console>, item: egui::Rect) -> Vec<egui::Rect> {
    fn collect(shape: &egui::Shape, side: f32, item: egui::Rect, boxes: &mut Vec<egui::Rect>) {
        match shape {
            egui::Shape::Rect(rect)
                if item.contains_rect(rect.rect)
                    && (rect.rect.width() - rect.rect.height()).abs() < 0.5
                    && rect.rect.width() >= side
                    && rect.rect.width() <= side + 4.0 =>
            {
                boxes.push(rect.rect);
            }
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, side, item, boxes);
                }
            }
            _ => {}
        }
    }

    let side = harness.ctx.global_style().spacing.icon_width;
    let mut boxes = Vec::new();
    for clipped in &harness.output().shapes {
        collect(&clipped.shape, side, item, &mut boxes);
    }
    boxes
}

///
/// View → Overview's check box trails its label, as View → Diagnostics' does,
/// and the item spans the menu as Diagnostics does.
///
#[tokio::test]
async fn the_overview_item_trails_its_check_box_as_diagnostics_does() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    harness.get_by_label("View").click();
    harness.step();
    harness.run_steps(1);

    let diagnostics = menu_item(&harness, "Diagnostics").rect();
    let overview = harness.get_by_label("Overview");
    assert_eq!(
        overview.accesskit_node().role(),
        egui::accesskit::Role::CheckBox
    );
    let overview = overview.rect();
    for (label, item) in [("Diagnostics", diagnostics), ("Overview", overview)] {
        let boxes = boxes_in(&harness, item);
        assert_eq!(
            boxes.len(),
            1,
            "{label}: expected one check box, found {boxes:?}"
        );
        assert!(
            boxes[0].left() > item.center().x,
            "{label}'s check box {:?} does not trail its label in {item:?}",
            boxes[0]
        );
    }
    assert_eq!(
        (overview.left(), overview.right()),
        (diagnostics.left(), diagnostics.right()),
        "the Overview item is not laid out across the menu as Diagnostics is"
    );
}

///
/// The space between two of the top bar's labels, edge to edge: the bottom
/// Panel's Readout group gap, three monospace cells, of which each menu
/// button's padding is part.
///
fn label_gap(harness: &Harness<'_, Console>) -> (f32, f32) {
    let mut style = (*harness.ctx.global_style()).clone();
    egui::containers::menu::menu_style(&mut style);
    let font = style.text_styles[&egui::TextStyle::Monospace].clone();
    let cell = harness.ctx.fonts_mut(|fonts| {
        fonts
            .layout("0".to_owned(), font, egui::Color32::WHITE, f32::INFINITY)
            .size()
            .x
    });
    (3.0 * cell, 2.0 * style.spacing.button_padding.x)
}

///
/// File, View and Help sit the bottom Panel's Readout group gap apart, label
/// to label, not that plus egui's item spacing.
///
#[tokio::test]
async fn the_top_bar_menus_sit_one_readout_group_gap_apart() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    harness.run_steps(2);
    let (gap, padding) = label_gap(&harness);
    let [file, view, help] = ["File", "View", "Help"].map(|menu| harness.get_by_label(menu).rect());
    for (left, right) in [(file, view), (view, help)] {
        assert!(
            (right.left() - left.right() + padding - gap).abs() < 0.01,
            "the menus are not {gap} apart label to label: {left:?} {right:?}"
        );
    }
}

///
/// The Notices menu sits the same gap left of the mode control, whose own
/// buttons keep egui's tighter spacing.
///
#[tokio::test]
async fn the_notices_sit_one_readout_group_gap_from_the_mode_control() {
    use crate::theme_registry::ThemeRegistry;
    use crate::theme_registry::tests_support::TempDir;

    let dir = TempDir::new();
    dir.write("broken.toml", "format = \"orcvs-theme\"\nversion = 2\n");
    let mut harness = super::running_console_with(
        Vec2::from(DEFAULT_VIEW_SIZE),
        ThemeRegistry::discover(dir.path()),
    );
    harness.run_steps(2);
    let (gap, padding) = label_gap(&harness);
    let notices = harness.get_by_label("Notices (1)").rect();
    let [follow, dark] = ["Follow the OS", "Dark"].map(|mode| harness.get_by_label(mode).rect());
    assert!(
        (follow.left() - notices.right() + padding - gap).abs() < 0.01,
        "the notices are not {gap} from the mode control: {notices:?} {follow:?}"
    );
    assert!(
        dark.left() - follow.right() + padding < gap,
        "the mode buttons take the menu gap: {follow:?} {dark:?}"
    );
}

///
/// The separator a bar's edge carries lies just outside the frame's outline,
/// as egui places its own line: touching the fill, whatever the line's width
/// and whatever outer margin the frame already has on that edge.
///
#[test]
fn a_bar_separator_touches_the_frame_outline() {
    use super::super::panel::{BarEdge, show_bar};

    const FILL: egui::Color32 = egui::Color32::from_rgb(200, 0, 0);
    const LINE: egui::Color32 = egui::Color32::from_rgb(0, 0, 200);

    fn collect(shape: &egui::Shape, fills: &mut Vec<egui::Rect>, lines: &mut Vec<(f32, f32)>) {
        match shape {
            egui::Shape::Rect(rect) if rect.fill == FILL => fills.push(rect.rect),
            egui::Shape::LineSegment { points, stroke } if stroke.color == LINE => {
                lines.push((points[0].y, stroke.width));
            }
            egui::Shape::Vec(shapes) => {
                for shape in shapes {
                    collect(shape, fills, lines);
                }
            }
            _ => {}
        }
    }

    for (width, outer_margin) in [(0.5, 0), (1.0, 4), (0.5, 4)] {
        for edge in [BarEdge::Top, BarEdge::Bottom] {
            let ctx = egui::Context::default();
            let mut output = ctx.run_ui(egui::RawInput::default(), |root| {
                let frame = egui::Frame::NONE
                    .fill(FILL)
                    .outer_margin(egui::Margin::same(outer_margin));
                let bar = match edge {
                    BarEdge::Top => egui::Panel::bottom("bar"),
                    BarEdge::Bottom => egui::Panel::top("bar"),
                }
                .resizable(false)
                .exact_size(24.0);
                show_bar(
                    root,
                    bar,
                    frame,
                    edge,
                    egui::Stroke::new(width, LINE),
                    |_| {},
                );
            });
            // The font atlas this frame uploaded is no part of the check.
            output.textures_delta.clear();
            let (mut fills, mut lines) = (Vec::new(), Vec::new());
            for clipped in &output.shapes {
                collect(&clipped.shape, &mut fills, &mut lines);
            }
            let [fill] = fills[..] else {
                panic!("expected one fill, found {fills:?}");
            };
            let [(y, stroke_width)] = lines[..] else {
                panic!("expected one separator, found {lines:?}");
            };
            let (outline, line_inner) = match edge {
                BarEdge::Top => (fill.top(), y + stroke_width / 2.0),
                BarEdge::Bottom => (fill.bottom(), y - stroke_width / 2.0),
            };
            assert!(
                (line_inner - outline).abs() < 0.01,
                "width {width}, outer margin {outer_margin}: the separator's inner edge \
                 {line_inner} is not at the outline {outline}"
            );
        }
    }
}
