//!
//! The Panel's Info beneath its readouts, as the shipped console draws it.
//!

use egui::Vec2;
use egui_kittest::kittest::Queryable as _;

use super::super::DEFAULT_VIEW_SIZE;
use super::{running_console, write_an_add_and_select};

///
/// The bottom Panel's outer rect on the last frame.
///
fn panel_rect(harness: &egui_kittest::Harness<'_, super::Console>) -> egui::Rect {
    egui::containers::panel::PanelState::load(
        &harness.ctx,
        super::super::tests::panel_id("bottom_panel"),
    )
    .expect("the Panel is present")
    .outer_rect
}

///
/// Moving the Cursor onto `.+0304` shows the Add it belongs to beneath the
/// Panel's readouts: its Cells over its name and its Roles. Moving it off
/// onto an empty Cell blanks Info and leaves the Panel where it was.
///
#[tokio::test]
async fn info_describes_the_function_under_the_cursor() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    write_an_add_and_select(&mut harness, 3);
    harness.run_steps(2);

    let cells = harness.get_by_label_contains(".+ 03 04").rect();
    let play = harness.get_by_label("Play").rect();
    assert!(
        cells.top() >= play.bottom(),
        "the Cells are at {} beside a Play Cell ending at {}",
        cells.top(),
        play.bottom()
    );
    for label in ["Add", "left", "right", "left + right"] {
        let rect = harness.get_by_label(label).rect();
        assert!(
            rect.top() >= cells.bottom(),
            "`{label}` at {rect:?} is not beneath the Cells at {cells:?}"
        );
    }

    {
        let orcvs = &mut harness.state_mut().orcvs;
        let grid = orcvs.grid();
        orcvs.select(grid.position(0, 3).expect("inside the Grid"));
    }
    harness.run_steps(2);
    assert!(harness.query_by_label("left + right").is_none());
    assert_eq!(harness.get_by_label("Play").rect(), play, "the Panel moved");
}

///
/// Each entry starts on a Cell column, at least two Cells past the wider of
/// the entry before's Cells and label.
///
#[tokio::test]
async fn info_entries_start_on_cell_columns_two_cells_apart() {
    let cell = crate::grid_viewport::CELL_SIZE;
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    write_an_add_and_select(&mut harness, 0);
    harness.run_steps(2);

    let labels =
        ["Add", "left", "right", "left + right"].map(|label| harness.get_by_label(label).rect());
    for rect in &labels {
        assert!(
            (rect.left() / cell.x).fract().abs() < 0.01,
            "an entry starts at {}, off the Cell columns",
            rect.left()
        );
    }
    for pair in labels.windows(2) {
        let (before, after) = (pair[0], pair[1]);
        let end = before.right().max(before.left() + 2.0 * cell.x);
        assert!(
            after.left() - end >= 2.0 * cell.x - 0.01,
            "an entry at {} starts {} past the one before",
            after.left(),
            after.left() - end
        );
    }
}

///
/// The Panel is Cell rows of the Cell the Source presents: one of room
/// beneath its top edge, the readouts, one of room, Info's Cells, its labels,
/// and one of room before the window's edge, at the default Cell and at a
/// configured one.
///
#[tokio::test]
async fn the_panel_lays_its_rows_on_the_grids_cells() {
    for cell in [crate::grid_viewport::CELL_SIZE, Vec2::new(16.0, 24.0)] {
        let config = crate::config::Config {
            cell,
            ..crate::config::Config::default()
        };
        let mut harness = super::console_harness(
            Vec2::from(DEFAULT_VIEW_SIZE),
            None,
            crate::theme_registry::ThemeRegistry::built_in(),
            config,
        );
        write_an_add_and_select(&mut harness, 0);
        harness.run_steps(2);

        let panel = panel_rect(&harness);
        assert_eq!(panel.height(), 6.0 * cell.y, "the Panel at Cell {cell:?}");
        let play = harness.get_by_label("Play").rect();
        assert_eq!(
            (play.top(), play.bottom()),
            (panel.top() + cell.y, panel.top() + 2.0 * cell.y),
            "the readouts' row at Cell {cell:?}"
        );
        let cells = harness.get_by_label_contains(".+ 03 04").rect();
        assert_eq!(
            (cells.top(), cells.bottom()),
            (panel.top() + 3.0 * cell.y, panel.top() + 4.0 * cell.y),
            "Info's Cells at Cell {cell:?}"
        );
        let label = harness.get_by_label("left + right").rect();
        assert!(
            label.top() >= cells.bottom() && label.bottom() <= panel.bottom() - cell.y,
            "the label at {label:?} is not in the Cell row beneath {cells:?} at Cell {cell:?}"
        );
    }
}

///
/// With Info hidden the Panel is three Cell rows, its readouts where they
/// were, and Info says nothing.
///
#[tokio::test]
async fn hiding_info_gives_its_rows_back_to_the_source() {
    let cell = crate::grid_viewport::CELL_SIZE;
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    write_an_add_and_select(&mut harness, 0);
    harness.run_steps(2);
    let shown = panel_rect(&harness);

    harness.state_mut().info_open = false;
    harness.run_steps(2);
    let hidden = panel_rect(&harness);
    assert_eq!(hidden.height(), 3.0 * cell.y, "the Panel with Info hidden");
    assert_eq!(
        hidden.bottom(),
        shown.bottom(),
        "the Panel left the window's edge"
    );
    let play = harness.get_by_label("Play").rect();
    assert_eq!(play.top(), hidden.top() + cell.y, "the readouts' row");
    assert!(harness.query_by_label("left + right").is_none());
}
