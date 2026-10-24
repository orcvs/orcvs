//! The Panel's Diagnostics readouts: frame timing and Source View geometry.

use super::source_view::ShownSourceView;

const TARGET_FPS: f32 = 60.0;
const FRAME_BUDGET_SECONDS: f32 = 1.0 / TARGET_FPS;

/// How long the timing readouts hold a value: four changes a second, slow
/// enough to read while frames arrive at 60 FPS.
pub(super) const TIMING_READOUT_INTERVAL_SECONDS: f64 = 0.25;

/// The frame and CPU time one timing readout shows.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Timing {
    pub(super) frame_time: f32,
    pub(super) cpu_usage: Option<f32>,
}

/// The timing readouts, each held for `TIMING_READOUT_INTERVAL_SECONDS`.
#[derive(Debug, Default)]
pub(super) struct TimingReadouts {
    shown: Option<(f64, Timing)>,
}

impl TimingReadouts {
    /// What to show at `now` seconds: the held readout, or `frame` once the
    /// held one is due for replacement.
    pub(super) fn observe(&mut self, now: f64, frame: Timing) -> Timing {
        match self.shown {
            Some((shown_at, held)) if now - shown_at < TIMING_READOUT_INTERVAL_SECONDS => held,
            _ => {
                self.shown = Some((now, frame));
                frame
            }
        }
    }
}

pub(super) fn frames_per_second(frame_time: f32) -> Option<f32> {
    frame_time.is_normal().then(|| frame_time.recip())
}

/// Signed CPU time minus the 60 FPS budget: positive means over budget.
pub(super) fn format_cpu_budget_delta(cpu_usage: Option<f32>) -> String {
    cpu_usage
        .map(|seconds| format!("{:+.2} ms", (seconds - FRAME_BUDGET_SECONDS) * 1_000.0))
        .unwrap_or_else(|| "—".to_owned())
}

fn row_height(ui: &egui::Ui) -> f32 {
    ui.spacing()
        .interact_size
        .y
        .max(ui.text_style_height(&egui::TextStyle::Body))
        .max(ui.text_style_height(&egui::TextStyle::Monospace))
}

/// Space for seven single-line readouts and the six gaps between them.
pub(super) fn summary_height(ui: &egui::Ui) -> f32 {
    7.0 * row_height(ui) + 6.0 * ui.spacing().item_spacing.y
}

/// Fills the space the Panel reserved, after the Source View has laid out this
/// frame's geometry. Long values truncate within the Panel's clip rectangle,
/// and the readouts scroll when the Panel reserved less than all of them.
/// The timing readouts change at the pace `timing` holds them to; the geometry
/// readouts follow every frame.
pub(super) fn show_diagnostics(
    ui: &mut egui::Ui,
    frame: &eframe::Frame,
    timing: &mut TimingReadouts,
    source: &ShownSourceView,
) {
    let (now, frame_time) = ui.input(|input| (input.time, input.stable_dt));
    let shown = timing.observe(
        now,
        Timing {
            frame_time,
            cpu_usage: frame.info().cpu_usage,
        },
    );
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
    egui::ScrollArea::vertical().show(ui, |ui| summary(ui, shown, source));
}

fn summary(
    ui: &mut egui::Ui,
    Timing {
        frame_time,
        cpu_usage,
    }: Timing,
    source: &ShownSourceView,
) {
    let cell_size = source.screen().cell();
    egui::Grid::new("orcvs-diagnostics-summary")
        .num_columns(2)
        .min_col_width(ui.available_width() / 6.0)
        .min_row_height(row_height(ui))
        .show(ui, |ui| {
            ui.add(egui::Label::new("FPS").extend());
            ui.monospace(
                frames_per_second(frame_time)
                    .map(|fps| format!("{fps:.1}"))
                    .unwrap_or_else(|| "—".to_owned()),
            );
            ui.end_row();

            ui.add(egui::Label::new("Frame time").extend());
            ui.monospace(format!("{:.2} ms", frame_time * 1_000.0));
            ui.end_row();

            ui.add(egui::Label::new("CPU time").extend());
            ui.monospace(
                cpu_usage
                    .map(|seconds| format!("{:.2} ms", seconds * 1_000.0))
                    .unwrap_or_else(|| "—".to_owned()),
            );
            ui.end_row();

            ui.add(egui::Label::new("Frame budget").extend());
            ui.monospace(format!(
                "{:.2} ms ({TARGET_FPS:.0} FPS)",
                FRAME_BUDGET_SECONDS * 1_000.0
            ));
            ui.end_row();

            ui.add(egui::Label::new("CPU budget delta").extend());
            ui.monospace(format_cpu_budget_delta(cpu_usage));
            ui.end_row();

            ui.add(egui::Label::new("Display").extend());
            ui.monospace(format!(
                "{:.1}×{:.1} pt/cell · {:.2} px/pt",
                cell_size.x,
                cell_size.y,
                ui.ctx().pixels_per_point()
            ));
            ui.end_row();

            ui.add(egui::Label::new("Visible Source region").extend());
            ui.monospace(format!("{:.1?}", source.shown()));
            ui.end_row();
        });
}
