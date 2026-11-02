//!
//! The Cursor Line beneath the Panel's row, as the shipped console draws it
//! (ADR 0100).
//!

use egui::Vec2;
use egui_kittest::kittest::Queryable as _;

use super::super::DEFAULT_VIEW_SIZE;
use super::running_console;

///
/// Moving the Cursor onto `.+0304` shows the Add it belongs to on the line
/// beneath the Panel's row; moving it off onto an empty Cell blanks the line
/// and leaves the Panel where it was.
///
#[tokio::test]
async fn the_cursor_line_describes_the_function_under_the_cursor() {
    let mut harness = running_console(Vec2::from(DEFAULT_VIEW_SIZE));
    {
        let orcvs = &mut harness.state_mut().orcvs;
        let grid = orcvs.grid();
        for (x, content) in ".+0304".chars().enumerate() {
            let cell = grid.index(grid.position(x, 0).expect("inside the Grid"));
            orcvs
                .source()
                .set(cell, &content.to_string())
                .expect("a Cell");
        }
        orcvs.select(grid.position(3, 0).expect("inside the Grid"));
    }
    harness.run_steps(2);

    let line = harness
        .get_by_label("Add  .+  answers left + right   left 03   right 04   answer ..  at 00 01")
        .rect();
    let play = harness.get_by_label("Play").rect();
    assert!(
        line.top() >= play.bottom(),
        "the line is at {} beside a Play Cell ending at {}",
        line.top(),
        play.bottom()
    );

    {
        let orcvs = &mut harness.state_mut().orcvs;
        let grid = orcvs.grid();
        orcvs.select(grid.position(0, 3).expect("inside the Grid"));
    }
    harness.run_steps(2);
    assert!(
        harness
            .query_by_label_contains("answers left + right")
            .is_none()
    );
    assert_eq!(harness.get_by_label("Play").rect(), play, "the Panel moved");
}
