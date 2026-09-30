//! The Diagnostics window: frame timing, where the Source View presents the
//! Source, and egui's own inspection.

use egui::{Pos2, Rect};

pub(super) fn frames_per_second(frame_time: f32) -> Option<f32> {
    frame_time.is_normal().then(|| frame_time.recip())
}

///
/// The part of the Source the console area shows, in the Source's own points:
/// `console` read back through the `origin` the Source's corner was presented
/// at.
///
pub(super) fn visible_source_region(console: Rect, origin: Pos2) -> Rect {
    console.translate(-origin.to_vec2())
}

///
/// The Diagnostics window, while `open`: this frame's timing, the Cell size
/// and the origin the Source was presented at in `console`, and
/// egui's inspection.
///
pub(super) fn show_diagnostics(
    ctx: &egui::Context,
    open: &mut bool,
    frame: &eframe::Frame,
    origin: Pos2,
    console: Rect,
    cell_size: f32,
) {
    let frame_time = ctx.input(|input| input.stable_dt);
    egui::Window::new("Diagnostics")
        .open(open)
        .default_width(360.0)
        .show(ctx, |ui| {
            egui::Grid::new("orcvs-diagnostics-summary")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("FPS");
                    ui.monospace(
                        frames_per_second(frame_time)
                            .map(|fps| format!("{fps:.1}"))
                            .unwrap_or_else(|| "—".to_owned()),
                    );
                    ui.end_row();

                    ui.label("Frame time");
                    ui.monospace(format!("{:.2} ms", frame_time * 1_000.0));
                    ui.end_row();

                    ui.label("CPU time");
                    ui.monospace(
                        frame
                            .info()
                            .cpu_usage
                            .map(|seconds| format!("{:.2} ms", seconds * 1_000.0))
                            .unwrap_or_else(|| "—".to_owned()),
                    );
                    ui.end_row();

                    ui.label("Cell size");
                    ui.monospace(format!("{cell_size:.1} pt"));
                    ui.end_row();

                    ui.label("Visible Source region");
                    ui.monospace(format!("{:.1?}", visible_source_region(console, origin)));
                    ui.end_row();

                    ui.label("Pixels per point");
                    ui.monospace(format!("{:.2}", ctx.pixels_per_point()));
                    ui.end_row();
                });

            ui.separator();
            egui::CollapsingHeader::new("egui inspection")
                .default_open(false)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(420.0)
                        .show(ui, |ui| ctx.inspection_ui(ui));
                });
        });
}
