//! A setting that glides to each new value rather than steps, so an effect
//! changed mid-note does not click.

/// How long a glide by `1` takes, in seconds.
const GLIDE: f32 = 0.02;

/// A value that moves by at most `1 / GLIDE` a second and reaches each new
/// value exactly.
pub(crate) struct Glide {
    step: f32,
    value: f32,
    target: f32,
}

impl Glide {
    /// A value at `value` from the start.
    pub(crate) fn new(sample_rate: f32, value: f32) -> Self {
        Self {
            step: 1.0 / (GLIDE * sample_rate),
            value,
            target: value,
        }
    }

    pub(crate) fn set(&mut self, target: f32) {
        self.target = target;
    }

    /// The value for the next sample.
    pub(crate) fn next(&mut self) -> f32 {
        let gap = self.target - self.value;
        self.value = if gap.abs() <= self.step {
            self.target
        } else {
            self.value + self.step.copysign(gap)
        };
        self.value
    }
}
