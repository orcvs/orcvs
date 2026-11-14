//! The MIDI controller numbers that set a channel's effects, under the
//! effects' Orcvs names. ADR 0088's table gives each one's MIDI name.

/// Modulation: vibrato on the channel.
pub(crate) const MD: u8 = 1;
/// Brightness: one tilt on the channel.
pub(crate) const EQ: u8 = 74;
/// Decay Time: how long each new sound on the channel decays.
pub(crate) const LN: u8 = 75;
/// How far into its course each new sound on the channel starts.
pub(crate) const OF: u8 = 85;
/// Effects 1 Depth: the send to the small reverb.
pub(crate) const R1: u8 = 91;
/// Effects 2 Depth: the send to the large reverb.
pub(crate) const R2: u8 = 92;
/// Effects 3 Depth: the send to the chorus.
pub(crate) const CH: u8 = 93;
/// Effects 4 Depth: the send to the echo.
pub(crate) const DL: u8 = 94;
