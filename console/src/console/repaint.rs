//! Repaint ownership: the wake-up that paints the Panel when the Playback
//! Engine publishes, and the timed Render Frame that keeps the Run Clock and
//! the Cursor Effect moving between publishes.

use std::time::Duration;

use orcvs::playback::{PlaybackObservation, PlaybackState};

use crate::readout_deadline::until_next;

///
/// Paints the Panel from a published Tick. A wait started from this Render
/// Frame is a second clock; this asks for a paint when the engine publishes.
///
pub(super) fn wake_panel_when_playback_publishes(
    ctx: egui::Context,
    observation: orcvs::playback::PlaybackObservationWatch,
) {
    let wake = panel_wake(ctx, observation);
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::spawn(wake);
    }
    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(wake);
    }
}

///
/// Requests a repaint of `ctx` each time `observation` publishes, and ends
/// once the Playback Engine that owns the observation's sender is gone. An
/// Open replaces the Orcvs, so each wake-up has to end with the Orcvs it
/// watches rather than outlive it.
///
/// The observation is marked seen before this returns, so only a publish
/// after the wake-up exists requests a repaint.
///
pub(super) fn panel_wake(
    ctx: egui::Context,
    mut observation: orcvs::playback::PlaybackObservationWatch,
) -> impl std::future::Future<Output = ()> {
    let _ = observation.borrow_and_update();
    async move {
        while observation.changed().await.is_ok() {
            ctx.request_repaint();
        }
    }
}

///
/// The Run Clock sample `run_clock` the Panel painted, while that Readout is
/// moving: Playing with a run under way. A stopped or unstarted clock needs no
/// timed Render Frame.
///
pub(super) fn moving_run_clock(
    observation: &PlaybackObservation,
    run_clock: Duration,
) -> Option<Duration> {
    (observation.state == PlaybackState::Playing && observation.run_started_at.is_some())
        .then_some(run_clock)
}

///
/// Requests the next timed Render Frame: the sooner of the next whole second
/// of a moving Run Clock and the Cursor Effect's next change, if either needs
/// one. The Playback Engine's publishes wake the Panel on their own through
/// [`panel_wake`], so nothing here polls for a Tick.
///
pub(super) fn request_timed_repaint(
    ctx: &egui::Context,
    run_clock: Option<Duration>,
    cursor_effect: Option<Duration>,
) {
    if let Some(delay) = until_next(
        run_clock,
        cursor_effect,
        Duration::from_secs_f32(ctx.input(|input| input.predicted_dt).max(0.0)),
    ) {
        ctx.request_repaint_after(delay);
    }
}
