//! What a Control Change does on a channel: the MIDI controller numbers the
//! synth answers, under the effects' Orcvs names, what each one means for the
//! channel's `Part`, and the value each loads at.

use crate::Part;

/// Modulation: vibrato on the channel.
pub const MD: u8 = 1;
/// Brightness: one tilt on the channel. On the acid channel, the voice's
/// cutoff.
pub const EQ: u8 = 74;
/// Decay Time: how long each new sound on the channel decays. On the acid
/// channel, the voice's decay.
pub(crate) const LN: u8 = 75;
/// How far into its course each new sound on the channel starts.
pub(crate) const OF: u8 = 85;
/// Effects 1 Depth: the send to the small reverb.
pub const R1: u8 = 91;
/// Effects 2 Depth: the send to the large reverb.
pub const R2: u8 = 92;
/// Effects 3 Depth: the send to the chorus.
pub const CH: u8 = 93;
/// Effects 4 Depth: the send to the echo.
pub const DL: u8 = 94;
/// All Sound Off: releases the channel's tones and its acid voice. A drum
/// rings out its own decay.
pub(crate) const ALL_SOUND_OFF: u8 = 120;
/// Reset All Controllers: clears `MD` on its channel and leaves the other
/// effects as they are.
pub(crate) const RESET: u8 = 121;
/// All Notes Off: releases the channel's tones and its acid voice. A drum
/// rings out its own decay.
pub(crate) const ALL_NOTES_OFF: u8 = 123;
/// Sound Controller 1: the acid voice's waveform.
pub(crate) const WAVEFORM: u8 = 70;
/// Sound Controller 2 (Timbre/Harmonic Intensity): the acid voice's
/// resonance.
pub(crate) const RESONANCE: u8 = 71;
/// Sound Controller 10: the acid voice's envelope depth.
pub(crate) const ENV_MOD: u8 = 79;

/// What a Control Change means on a channel.
#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) enum Control {
    /// Releases the channel's tones and its acid voice. A drum rings out its
    /// own decay.
    Release,
    /// Returns the channel's performance controllers to their load values.
    Reset,
    /// Sets one of the channel's own effects.
    Effect(Effect),
    /// Sets one of the acid voice's controls.
    Acid(AcidControl),
}

/// One of a channel's own effects.
#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) enum Effect {
    /// `MD`.
    Vibrato,
    /// `EQ`.
    Tilt,
    /// `LN`.
    Length,
    /// `OF`.
    Offset,
    /// The channel's send to a shared effect: `CH`, `DL`, `R1` or `R2`.
    Send(SharedEffect),
}

/// One of the four shared effects a channel sends to.
#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) enum SharedEffect {
    Chorus,
    Delay,
    Small,
    Large,
}

/// One of the acid voice's controls. They are Sound Controllers, which Reset
/// All Controllers leaves alone.
#[derive(Clone, Copy)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) enum AcidControl {
    Waveform,
    Resonance,
    Cutoff,
    Decay,
    EnvMod,
}

impl Control {
    /// What `controller` means on a channel playing `part`, or `None` for a
    /// controller that changes nothing there.
    ///
    /// The acid voice's own controls come first, so on its channel `EQ` and
    /// `LN` are its cutoff and decay and never the channel's tilt and length.
    pub(crate) fn of(part: Part, controller: u8) -> Option<Self> {
        if part == Part::Acid
            && let Some(control) = AcidControl::on(controller)
        {
            return Some(Self::Acid(control));
        }
        Some(match controller {
            ALL_SOUND_OFF | ALL_NOTES_OFF => Self::Release,
            RESET => Self::Reset,
            MD => Self::Effect(Effect::Vibrato),
            EQ => Self::Effect(Effect::Tilt),
            LN => Self::Effect(Effect::Length),
            OF => Self::Effect(Effect::Offset),
            CH => Self::Effect(Effect::Send(SharedEffect::Chorus)),
            DL => Self::Effect(Effect::Send(SharedEffect::Delay)),
            R1 => Self::Effect(Effect::Send(SharedEffect::Small)),
            R2 => Self::Effect(Effect::Send(SharedEffect::Large)),
            _ => return None,
        })
    }
}

impl Effect {
    /// The effects Reset All Controllers returns to their load values:
    /// Modulation, a performance controller. The Sound and Effect
    /// Controllers, and `OF` beside them, stay as they are, as RP-015 has it.
    pub(crate) const CLEARED_BY_RESET: [Self; 1] = [Self::Vibrato];

    /// The Control Change value the effect holds until one is sent: no
    /// vibrato, a flat tilt, a sound's whole length from its start, and
    /// nothing sent.
    pub(crate) const fn load(self) -> u8 {
        match self {
            Self::Tilt => 0x40,
            Self::Length => 0x7F,
            Self::Vibrato | Self::Offset | Self::Send(_) => 0x00,
        }
    }
}

// Each effect a channel's rest reads loads at the value that passes its input
// unchanged, so a fresh channel rests and is skipped while silent.
const _: () = assert!(
    crate::vibrato::swing(Effect::Vibrato.load()) == 0.0
        && crate::tilt::amount(Effect::Tilt.load()) == 0.0
        && crate::channel::gain(Effect::Send(SharedEffect::Chorus).load()) == 0.0
        && crate::channel::gain(Effect::Send(SharedEffect::Delay).load()) == 0.0
        && crate::channel::gain(Effect::Send(SharedEffect::Small).load()) == 0.0
        && crate::channel::gain(Effect::Send(SharedEffect::Large).load()) == 0.0
);

impl AcidControl {
    /// The control `controller` sets, if it is one of the voice's.
    fn on(controller: u8) -> Option<Self> {
        Some(match controller {
            WAVEFORM => Self::Waveform,
            RESONANCE => Self::Resonance,
            EQ => Self::Cutoff,
            LN => Self::Decay,
            ENV_MOD => Self::EnvMod,
            _ => return None,
        })
    }

    /// The Control Change value the control holds until one is sent. Together
    /// they make an acid sound with no Control Change.
    pub(crate) const fn load(self) -> u8 {
        match self {
            Self::Waveform => 0x00,
            Self::Resonance | Self::EnvMod => 0x50,
            Self::Cutoff | Self::Decay => 0x30,
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    const PARTS: [Part; 3] = [Part::Tone, Part::Acid, Part::Kit];

    /// Every controller a channel's own effects answer, with its meaning.
    pub(crate) const EFFECTS: [(u8, Effect); 8] = [
        (MD, Effect::Vibrato),
        (EQ, Effect::Tilt),
        (LN, Effect::Length),
        (OF, Effect::Offset),
        (CH, Effect::Send(SharedEffect::Chorus)),
        (DL, Effect::Send(SharedEffect::Delay)),
        (R1, Effect::Send(SharedEffect::Small)),
        (R2, Effect::Send(SharedEffect::Large)),
    ];

    /// Every controller some channel answers.
    const KNOWN: [u8; 14] = [
        MD,
        EQ,
        LN,
        OF,
        R1,
        R2,
        CH,
        DL,
        ALL_SOUND_OFF,
        RESET,
        ALL_NOTES_OFF,
        WAVEFORM,
        RESONANCE,
        ENV_MOD,
    ];

    /// Every controller the acid voice answers, with its meaning.
    pub(crate) const ACID: [(u8, AcidControl); 5] = [
        (WAVEFORM, AcidControl::Waveform),
        (RESONANCE, AcidControl::Resonance),
        (EQ, AcidControl::Cutoff),
        (LN, AcidControl::Decay),
        (ENV_MOD, AcidControl::EnvMod),
    ];

    #[test]
    fn a_tone_or_kit_channel_answers_each_effect_controller_as_its_effect() {
        for part in [Part::Tone, Part::Kit] {
            for (controller, effect) in EFFECTS {
                assert_eq!(
                    Control::of(part, controller),
                    Some(Control::Effect(effect)),
                    "{part:?}, controller {controller}"
                );
            }
        }
    }

    #[test]
    fn the_acid_channel_answers_its_voices_controllers_as_the_voices_controls() {
        for (controller, control) in ACID {
            assert_eq!(
                Control::of(Part::Acid, controller),
                Some(Control::Acid(control)),
                "controller {controller}"
            );
        }
    }

    #[test]
    fn the_acid_channel_answers_every_other_effect_controller_as_its_effect() {
        for (controller, effect) in EFFECTS {
            if ACID.iter().any(|&(taken, _)| taken == controller) {
                continue;
            }
            assert_eq!(
                Control::of(Part::Acid, controller),
                Some(Control::Effect(effect)),
                "controller {controller}"
            );
        }
    }

    #[test]
    fn the_acid_voices_own_controllers_change_nothing_on_another_channel() {
        for part in [Part::Tone, Part::Kit] {
            for controller in [WAVEFORM, RESONANCE, ENV_MOD] {
                assert_eq!(
                    Control::of(part, controller),
                    None,
                    "{part:?}, controller {controller}"
                );
            }
        }
    }

    #[test]
    fn every_channel_answers_the_channel_mode_controllers_alike() {
        for part in PARTS {
            assert_eq!(Control::of(part, ALL_SOUND_OFF), Some(Control::Release));
            assert_eq!(Control::of(part, ALL_NOTES_OFF), Some(Control::Release));
            assert_eq!(Control::of(part, RESET), Some(Control::Reset));
        }
    }

    #[test]
    fn every_controller_outside_the_table_changes_nothing() {
        for part in PARTS {
            for controller in (0..=0x7F).filter(|controller| !KNOWN.contains(controller)) {
                assert_eq!(
                    Control::of(part, controller),
                    None,
                    "{part:?}, controller {controller}"
                );
            }
        }
    }

    #[test]
    fn every_controller_in_the_table_means_only_what_the_tests_list() {
        // Each controller in `KNOWN` means something on some channel, and
        // only what `EFFECTS`, `ACID` or the channel mode controllers list.
        for controller in KNOWN {
            let meanings: Vec<Control> = PARTS
                .iter()
                .filter_map(|&part| Control::of(part, controller))
                .collect();
            assert!(!meanings.is_empty(), "controller {controller}");
            for meaning in meanings {
                let listed = match meaning {
                    Control::Release => [ALL_SOUND_OFF, ALL_NOTES_OFF].contains(&controller),
                    Control::Reset => controller == RESET,
                    Control::Effect(effect) => EFFECTS.contains(&(controller, effect)),
                    Control::Acid(control) => ACID.contains(&(controller, control)),
                };
                assert!(listed, "controller {controller} as {meaning:?}");
            }
        }
    }

    #[test]
    fn each_control_loads_at_its_tables_value() {
        let effects = [
            (Effect::Vibrato, 0x00),
            (Effect::Tilt, 0x40),
            (Effect::Length, 0x7F),
            (Effect::Offset, 0x00),
            (Effect::Send(SharedEffect::Chorus), 0x00),
            (Effect::Send(SharedEffect::Delay), 0x00),
            (Effect::Send(SharedEffect::Small), 0x00),
            (Effect::Send(SharedEffect::Large), 0x00),
        ];
        for (_, effect) in EFFECTS {
            let value = effects.iter().find(|&&(listed, _)| listed == effect);
            assert_eq!(
                value.map(|&(_, value)| value),
                Some(effect.load()),
                "{effect:?}"
            );
        }
        let acid = [
            (AcidControl::Waveform, 0x00),
            (AcidControl::Resonance, 0x50),
            (AcidControl::Cutoff, 0x30),
            (AcidControl::Decay, 0x30),
            (AcidControl::EnvMod, 0x50),
        ];
        for (_, control) in ACID {
            let value = acid.iter().find(|&&(listed, _)| listed == control);
            assert_eq!(
                value.map(|&(_, value)| value),
                Some(control.load()),
                "{control:?}"
            );
        }
    }
}
