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
        .get_by_label(".+ Add               left 03  right 04   answers left + right")
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

///
/// However little width the Panel has, a Function's answer stays whole at the
/// right edge and the lead gives way to it.
///
#[tokio::test]
async fn the_answer_holds_the_right_edge_as_the_lead_gives_way() {
    for width in [DEFAULT_VIEW_SIZE[0], 420.0] {
        let size = Vec2::new(width, DEFAULT_VIEW_SIZE[1]);
        let mut harness = running_console(size);
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
            orcvs.select(grid.position(0, 0).expect("inside the Grid"));
        }
        harness.run_steps(2);

        let answer = harness.get_by_label("→ .. at 00 01").rect();
        let lead = harness.get_by_label_contains(".+ Add").rect();
        assert!(
            answer.right() > width - 40.0 && answer.right() <= width,
            "at width {width} the answer ends at {}",
            answer.right()
        );
        assert!(
            lead.right() <= answer.left(),
            "at width {width} the lead ends at {}, past the answer at {}",
            lead.right(),
            answer.left()
        );
    }
}
