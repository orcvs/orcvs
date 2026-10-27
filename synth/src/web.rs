//! The synth as the AudioWorklet's WebAssembly module sees it (ADR 0087).
//!
//! The worklet instantiates the module with no imports and calls these three
//! functions from the audio thread, which is the module's only thread. Plain
//! C-ABI exports keep the module free of generated JavaScript glue, which an
//! AudioWorklet's scope could not load.

use std::cell::RefCell;

use crate::Synth;

thread_local! {
    static SYNTH: RefCell<Synth> = RefCell::new(Synth::new(48_000.0));
    static SAMPLES: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
}

/// Starts a silent synth rendering at `sample_rate`, its frame clock at zero.
// SAFETY: no other symbol in the module is named `synth_start`.
#[unsafe(no_mangle)]
pub extern "C" fn synth_start(sample_rate: f32) {
    SYNTH.with(|synth| *synth.borrow_mut() = Synth::new(sample_rate));
}

/// Queues the message `status`, `data1`, `data2` at `frame` on the synth's
/// clock. A frame before zero, or one already rendered, sounds at once.
// SAFETY: no other symbol in the module is named `synth_schedule`.
#[unsafe(no_mangle)]
pub extern "C" fn synth_schedule(frame: f64, status: u8, data1: u8, data2: u8) {
    // `as` saturates: a negative or NaN frame is zero.
    let frame = frame as u64;
    SYNTH.with(|synth| synth.borrow_mut().schedule(frame, [status, data1, data2]));
}

/// Renders the next `frames` samples and answers where in the module's memory
/// they start. They stay there until the next call.
// SAFETY: no other symbol in the module is named `synth_render`.
#[unsafe(no_mangle)]
pub extern "C" fn synth_render(frames: u32) -> *const f32 {
    SAMPLES.with(|samples| {
        let mut samples = samples.borrow_mut();
        samples.resize(frames as usize, 0.0);
        SYNTH.with(|synth| synth.borrow_mut().render(&mut samples));
        samples.as_ptr()
    })
}
