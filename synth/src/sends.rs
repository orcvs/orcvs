//! One of something for each shared effect, and the mix the channels send
//! them.

use crate::chorus::Chorus;
use crate::controller::SharedEffect;
use crate::delay::Delay;
use crate::reverb::Reverb;

/// One of something for each shared effect: the chorus, the delay, and the
/// small and large reverbs. The processors differ in type, so each field may;
/// every other use holds one type in all four.
#[derive(Clone, Copy, Default)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) struct Sends<C, D = C, S = C, L = S> {
    pub(crate) chorus: C,
    pub(crate) delay: D,
    pub(crate) small: S,
    pub(crate) large: L,
}

impl<T> Sends<T> {
    /// Four values, each made by `make` for its shared effect.
    pub(crate) fn from_fn(mut make: impl FnMut(SharedEffect) -> T) -> Self {
        Self {
            chorus: make(SharedEffect::Chorus),
            delay: make(SharedEffect::Delay),
            small: make(SharedEffect::Small),
            large: make(SharedEffect::Large),
        }
    }

    /// The one for `effect`.
    pub(crate) fn of(&mut self, effect: SharedEffect) -> &mut T {
        match effect {
            SharedEffect::Chorus => &mut self.chorus,
            SharedEffect::Delay => &mut self.delay,
            SharedEffect::Small => &mut self.small,
            SharedEffect::Large => &mut self.large,
        }
    }

    /// Each of the four through `f`, in field order.
    pub(crate) fn map<U>(self, mut f: impl FnMut(T) -> U) -> Sends<U> {
        Sends {
            chorus: f(self.chorus),
            delay: f(self.delay),
            small: f(self.small),
            large: f(self.large),
        }
    }

    /// What `f` answers for each of the four, in field order.
    pub(crate) fn map_mut<U>(&mut self, mut f: impl FnMut(&mut T) -> U) -> Sends<U> {
        Sends {
            chorus: f(&mut self.chorus),
            delay: f(&mut self.delay),
            small: f(&mut self.small),
            large: f(&mut self.large),
        }
    }

    /// Whether `test` holds for all four.
    pub(crate) fn all(&self, test: impl Fn(&T) -> bool) -> bool {
        [&self.chorus, &self.delay, &self.small, &self.large]
            .into_iter()
            .all(test)
    }
}

impl std::ops::AddAssign for Sends<f32> {
    fn add_assign(&mut self, other: Self) {
        self.chorus += other.chorus;
        self.delay += other.delay;
        self.small += other.small;
        self.large += other.large;
    }
}

impl Sends<Chorus, Delay, Reverb> {
    /// `mix`'s dry signal with the shared effects side by side, each
    /// returning only what the channels sent it.
    pub(crate) fn process(&mut self, mix: Mix) -> f32 {
        let sent = mix.sends;
        mix.dry
            + 0.5 * self.chorus.process(sent.chorus)
            + self.delay.process(sent.delay)
            + self.small.process(sent.small)
            + self.large.process(sent.large)
    }
}

/// One sample of the mix ahead of the shared effects: every channel's dry
/// signal, and what the channels send each shared effect.
#[derive(Default)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) struct Mix {
    pub(crate) dry: f32,
    pub(crate) sends: Sends<f32>,
}

impl std::ops::AddAssign for Mix {
    fn add_assign(&mut self, other: Self) {
        self.dry += other.dry;
        self.sends += other.sends;
    }
}
