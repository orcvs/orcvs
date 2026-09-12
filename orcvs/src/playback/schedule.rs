//!
//! The notes Timed and Monophonic Play own, and the Tick each is stopped at.
//!
//! ADR 0016 gives those two spellings a Tick lifetime, and resolving a
//! lifetime into a start now and a stop later is the whole of what this module
//! does. It is pure: two maps, a claim counter, and four methods over them.
//! No clock, no lock and no output adapter reaches in here, which is why the
//! ownership, expiry and delivery-order rules are stated against it directly
//! rather than through the engine that drives it.
//!
//! The engine beside it supplies the two things this module deliberately does
//! not have — the absolute Tick each delivery is resolved at, and the adapter
//! the delivery is submitted to.
//!

use std::collections::BTreeMap;

use super::OutputCommand;
use crate::source::{Length, MidiChannel, Note, PlayCommand, Tick, Velocity};

///
/// One voice this schedule can own, and what counts as the same voice.
///
/// The key is the whole of the difference between the two Play spellings that
/// own anything. ADR 0016 makes Timed Play polyphonic, so a channel sounds as
/// many Timed notes at once as the Source starts on it and each is owned in
/// its own right; Monophonic Play owns one voice per channel, so the note is
/// not part of what identifies the voice but what the voice is currently
/// sounding. Two variants of one key rather than a schedule each, because
/// everything that follows a claim — the generation token, the Tick its stop
/// is due at, the staleness check, and the lifecycle actions that clear the
/// lot — is identical for both, and only what a replacement replaces differs.
///
/// Being distinct variants is also what keeps the two ownerships apart: a Mono
/// command cannot find a Timed claim to stop, and a Timed expiry cannot stop a
/// Mono note, because neither key can name the other's voice. Raw Play has no
/// variant here at all, since ADR 0016 leaves its Note Off under Source
/// control and nothing this schedule owns may stop it.
///
/// Each variant carries the domain types the interpreter proved rather than
/// their bytes, so the stop this module delivers re-derives neither.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Voice {
    Timed { channel: MidiChannel, note: Note },
    Mono { channel: MidiChannel },
}

impl Voice {
    /// The channel this voice sounds on, which every stop needs and which is
    /// the only field both variants share.
    fn channel(self) -> MidiChannel {
        match self {
            Self::Timed { channel, .. } | Self::Mono { channel } => channel,
        }
    }
}

///
/// What a voice is sounding: the note, and the claim that started it.
///
/// The note is recorded rather than read back off the key because a Mono
/// voice's key does not name one — its channel is the voice and its note is
/// only what that voice happens to sound — and a stop needs the note either
/// way. A Timed voice restates its note here, which is the price of one
/// schedule instead of two.
///
#[derive(Clone, Copy, Debug)]
struct Sounding {
    note: Note,
    claim: Claim,
}

///
/// Which claim on a voice a scheduled stop belongs to.
///
/// ADR 0016's generation token. An expiry is scheduled at the Tick it is due
/// at and cannot be found again when the note it would stop is replaced or
/// stopped early, so a stale one is left in the schedule and refused when it
/// comes due: it carries the claim that scheduled it, and only the claim still
/// standing then is stopped. Without it a Source that stops a note and starts
/// it again would have the first command's expiry cut the second note short.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Claim(u64);

/// One scheduled stop, and the claim it belongs to.
#[derive(Clone, Copy, Debug)]
struct Expiry {
    voice: Voice,
    claim: Claim,
}

///
/// Every note a Timed or Monophonic Play command owns, and the Tick each is
/// stopped at.
///
/// ADR 0001 keeps musical intent out of the output adapter and ADR 0016 puts a
/// Play's whole lifetime in the Tick Plan, which leaves exactly this between
/// them: the engine reads the length, delivers the start in Tick Plan order,
/// and delivers the stop when the run reaches the Tick it is due at.
///
/// It is the only state the engine keeps that outlives one Tick, and it
/// describes notes that are sounding, so everything that silences output
/// clears it: beginning a run, stopping, disconnecting, and changing
/// destination.
///
#[derive(Clone, Default)]
pub(crate) struct OwnedNotes {
    voices: BTreeMap<Voice, Sounding>,
    expiries: BTreeMap<Tick, Vec<Expiry>>,
    next_claim: u64,
}

///
/// The explicit stop for `note` on `channel`.
///
/// MIDI's zero-velocity Note On, which is the stop Raw Play already gives the
/// Source through velocity `00`: a scheduled expiry is delivered as a message
/// a Source could have written for itself rather than as a shape of its own.
///
fn note_off(channel: MidiChannel, note: Note) -> OutputCommand {
    OutputCommand::NoteOn {
        channel,
        velocity: Velocity::ZERO,
        note,
    }
}

impl OwnedNotes {
    ///
    /// This Tick's delivery: every stop due at `tick`, then `commands`
    /// resolved against ownership, in Tick Plan order.
    ///
    /// One list rather than two submissions, because the order is the whole of
    /// what ADR 0016 requires here — a scheduled Note Off arrives at the
    /// beginning of executed Tick `T + length`, before that Tick's new Play
    /// Commands — and a Tick that submitted twice would leave that order to
    /// the adapter to keep.
    ///
    pub(crate) fn deliver(&mut self, tick: Tick, commands: &[PlayCommand]) -> Vec<OutputCommand> {
        let mut delivery = self.expired_at(tick);

        for command in commands {
            match *command {
                // Raw notes do not enter Timed ownership: what the Source
                // wrote is delivered, and nothing stops it that the Source did
                // not ask to stop.
                PlayCommand::Raw {
                    channel,
                    velocity,
                    note,
                } => delivery.push(OutputCommand::NoteOn {
                    channel,
                    velocity,
                    note,
                }),
                PlayCommand::Timed {
                    channel,
                    velocity,
                    note,
                    length,
                } => {
                    let voice = Voice::Timed { channel, note };
                    if velocity == Velocity::ZERO {
                        // An explicit stop, whatever length accompanies it,
                        // scheduling no expiry. Releasing the claim is what
                        // keeps the expiry this note already had from stopping
                        // whatever sounds on the voice next.
                        self.release(voice);
                        delivery.push(note_off(channel, note));
                    } else if length == Length::ZERO {
                        // A lifetime of no Ticks never starts, and is not a
                        // stop: the note this voice owns and the expiry it is
                        // due both stand.
                    } else {
                        // A replacement stops the instance it replaces before
                        // it starts, and retires that instance's expiry with it.
                        if self.release(voice).is_some() {
                            delivery.push(note_off(channel, note));
                        }
                        delivery.push(OutputCommand::NoteOn {
                            channel,
                            velocity,
                            note,
                        });
                        self.claim(voice, note, tick.after(length.ticks()));
                    }
                }
                PlayCommand::Mono {
                    channel,
                    velocity,
                    note,
                    length,
                } => {
                    let voice = Voice::Mono { channel };
                    // Every Monophonic command stops the note its channel was
                    // sounding before it does anything else, and whether or
                    // not it goes on to start one. The note comes back out of
                    // the claim rather than off the command, which is what
                    // monophony means here: the voice is the channel, and the
                    // Source need not remember what it last put on it.
                    if let Some(stopped) = self.release(voice) {
                        delivery.push(note_off(channel, stopped));
                    }
                    // Velocity `00` and length `00` both leave the channel
                    // silent, and ADR 0016 makes that the end of the command.
                    // Timed Play's length `00` is a no-op instead, and the
                    // difference is not an inconsistency: a Timed command
                    // claims the note it names, so a note that never starts
                    // claims nothing and disturbs nothing, while a Monophonic
                    // command claims the channel whether or not it sounds, so
                    // one that starts nothing has replaced the voice with
                    // silence.
                    if velocity != Velocity::ZERO && length != Length::ZERO {
                        delivery.push(OutputCommand::NoteOn {
                            channel,
                            velocity,
                            note,
                        });
                        self.claim(voice, note, tick.after(length.ticks()));
                    }
                }
                // Neither of these owns a voice or is due at another Tick, so
                // there is nothing here to resolve and nothing to schedule:
                // each is carried through in its Tick Plan order, which is the
                // whole of what this module owes them. Written out field by
                // field rather than passed through as one value, because a
                // Play Command and an Output Command are separate types on
                // purpose — the day one of them differs, the difference is an
                // edit here rather than a conversion nobody can see.
                PlayCommand::ControlChange {
                    channel,
                    controller,
                    value,
                } => delivery.push(OutputCommand::ControlChange {
                    channel,
                    controller,
                    value,
                }),
                PlayCommand::PitchBend { channel, lsb, msb } => {
                    delivery.push(OutputCommand::PitchBend { channel, lsb, msb })
                }
            }
        }

        delivery
    }

    ///
    /// The stops due at `tick`, in the order they were scheduled.
    ///
    /// Everything due at or before it, though an ordinary run reaches every
    /// Tick in turn: a Tick the engine declines consumes no absolute Tick, so
    /// nothing is skipped, and draining the whole range regardless is what
    /// keeps an expiry from outliving the Tick it was due at by the Ticks a
    /// future scheduling rule might skip. One expiry is beyond it — a stop
    /// `Tick::after` saturated at the last Tick, which the counter it is
    /// compared against can no longer reach — and a run whose absolute Tick
    /// has stopped advancing has already lost more than a Note Off.
    ///
    fn expired_at(&mut self, tick: Tick) -> Vec<OutputCommand> {
        let later = self.expiries.split_off(&tick.next());
        let due = std::mem::replace(&mut self.expiries, later);

        let mut stops = Vec::new();
        for expiry in due.into_values().flatten() {
            // A stale expiry stops nothing: its claim was released when the
            // voice was replaced or stopped, so what sounds there now is not
            // what it was scheduled for. The note comes from the claim rather
            // than from the expiry for the reason a Mono voice needs it to —
            // the key names a channel, not a note — and reading it there means
            // only a claim that is still standing can name a note to stop.
            if let Some(sounding) = self.voices.get(&expiry.voice).copied()
                && sounding.claim == expiry.claim
            {
                self.voices.remove(&expiry.voice);
                stops.push(note_off(expiry.voice.channel(), sounding.note));
            }
        }
        stops
    }

    ///
    /// Claims `voice` for `note` until `due`, so the Tick it is due at stops
    /// it.
    ///
    fn claim(&mut self, voice: Voice, note: Note, due: Tick) {
        // A Timed key names the note it sounds, and the claim records it
        // again, so the two must agree: `expired_at` reads the note from the
        // claim, and a disagreement here would stop a note this voice never
        // sounded and leave the one it did standing. A Mono key names no note
        // and has nothing to agree with.
        debug_assert!(
            !matches!(voice, Voice::Timed { note: keyed, .. } if keyed != note),
            "a Timed voice claimed a note its key does not name"
        );
        // Unreachable for the reason `Tick::next`'s saturation is unreachable:
        // a run would have to claim a voice every nanosecond for five hundred
        // years to wrap this counter.
        let claim = Claim(self.next_claim);
        self.next_claim = self.next_claim.wrapping_add(1);
        self.voices.insert(voice, Sounding { note, claim });
        self.expiries
            .entry(due)
            .or_default()
            .push(Expiry { voice, claim });
    }

    ///
    /// Gives up any claim on `voice`, invalidating the stop it scheduled, and
    /// answers the note that was standing on it.
    ///
    fn release(&mut self, voice: Voice) -> Option<Note> {
        self.voices.remove(&voice).map(|sounding| sounding.note)
    }

    ///
    /// Forgets every claim and every scheduled stop.
    ///
    pub(crate) fn clear(&mut self) {
        self.voices.clear();
        self.expiries.clear();
    }

    ///
    /// Whether any note is claimed or any stop still scheduled.
    ///
    /// The lifecycle rule ADR 0016 leaves to the engine is that nothing
    /// survives a run, and a run that has ended delivers nothing more for a
    /// test to read: what is left to observe is the state itself.
    ///
    #[cfg(test)]
    pub(crate) fn holds_note_ownership(&self) -> bool {
        !self.voices.is_empty() || !self.expiries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{BendLsb, BendMsb, ControlValue, Controller};

    ///
    /// The Note On a delivery carries for `channel`, `velocity` and `note`,
    /// stated as the three Numbers a Source writes.
    ///
    fn note_on(channel: u8, velocity: u8, note: u8) -> OutputCommand {
        OutputCommand::NoteOn {
            channel: MidiChannel::try_from(channel).expect("a MIDI channel"),
            velocity: Velocity::try_from(velocity).expect("a MIDI data byte"),
            note: Note::try_from(note).expect("a MIDI note"),
        }
    }

    ///
    /// The stop a delivery carries for `channel` and `note`: MIDI's
    /// zero-velocity Note On, named for what it does rather than what it is.
    ///
    fn stop(channel: u8, note: u8) -> OutputCommand {
        note_on(channel, 0, note)
    }

    ///
    /// One Raw Play Command, stated as the three Numbers a Source writes.
    ///
    fn raw_play(channel: u8, velocity: u8, note: u8) -> PlayCommand {
        PlayCommand::Raw {
            channel: MidiChannel::try_from(channel).expect("a MIDI channel"),
            velocity: Velocity::try_from(velocity).expect("a MIDI data byte"),
            note: Note::try_from(note).expect("a MIDI note"),
        }
    }

    ///
    /// One Timed Play Command, stated as the four Numbers a Source writes.
    ///
    fn timed_play(channel: u8, velocity: u8, note: u8, length: u8) -> PlayCommand {
        PlayCommand::Timed {
            channel: MidiChannel::try_from(channel).expect("a MIDI channel"),
            velocity: Velocity::try_from(velocity).expect("a MIDI data byte"),
            note: Note::try_from(note).expect("a MIDI note"),
            length: Length::from(length),
        }
    }

    ///
    /// One Monophonic Play Command, stated as the four Numbers a Source
    /// writes.
    ///
    fn mono_play(channel: u8, velocity: u8, note: u8, length: u8) -> PlayCommand {
        PlayCommand::Mono {
            channel: MidiChannel::try_from(channel).expect("a MIDI channel"),
            velocity: Velocity::try_from(velocity).expect("a MIDI data byte"),
            note: Note::try_from(note).expect("a MIDI note"),
            length: Length::from(length),
        }
    }

    ///
    /// The Control Change a delivery carries, stated as the three Numbers a
    /// Source writes.
    ///
    fn control_change(channel: u8, controller: u8, value: u8) -> OutputCommand {
        OutputCommand::ControlChange {
            channel: MidiChannel::try_from(channel).expect("a MIDI channel"),
            controller: Controller::try_from(controller).expect("a MIDI data byte"),
            value: ControlValue::try_from(value).expect("a MIDI data byte"),
        }
    }

    ///
    /// The Pitch Bend a delivery carries, LSB before MSB as the wire takes
    /// them.
    ///
    fn pitch_bend(channel: u8, lsb: u8, msb: u8) -> OutputCommand {
        OutputCommand::PitchBend {
            channel: MidiChannel::try_from(channel).expect("a MIDI channel"),
            lsb: BendLsb::try_from(lsb).expect("a MIDI data byte"),
            msb: BendMsb::try_from(msb).expect("a MIDI data byte"),
        }
    }

    ///
    /// The delivery for Tick `tick`, with no Play Commands planned: the Ticks
    /// between a start and the stop it is due, stated as the absolute Tick
    /// numbers ADR 0016 counts in.
    ///
    fn idle(schedule: &mut OwnedNotes, tick: u64) -> Vec<OutputCommand> {
        schedule.deliver(Tick::new(tick), &[])
    }

    ///
    /// The delivery for Tick `tick` of a Tick Plan carrying `commands`.
    ///
    fn plan(schedule: &mut OwnedNotes, tick: u64, commands: &[PlayCommand]) -> Vec<OutputCommand> {
        schedule.deliver(Tick::new(tick), commands)
    }

    #[test]
    fn a_chord_of_timed_plays_stops_each_element_at_its_own_length() {
        // ADR 0030 widens `!~` over a Sequence, so one Expression hands the
        // schedule several Timed Play commands at once, in element index
        // order. Ownership is already keyed by channel and note and each
        // command already carries its own length, so nothing here was added
        // for the group — this states that nothing needed to be, rather than
        // assuming it.
        //
        // Three notes with three different lengths in one delivery: a schedule
        // that read one length for the whole group, or that let a later
        // element's claim displace an earlier one, stops the wrong notes at the
        // wrong Ticks.
        let mut schedule = OwnedNotes::default();

        let started = plan(
            &mut schedule,
            0,
            &[
                timed_play(0, 0x7F, 60, 1),
                timed_play(0, 0x7F, 64, 2),
                timed_play(0, 0x7F, 67, 3),
            ],
        );

        // Delivery is in slice order, which is the element index order the Tick
        // Plan carried across the seam.
        assert_eq!(
            started,
            vec![
                note_on(0, 0x7F, 60),
                note_on(0, 0x7F, 64),
                note_on(0, 0x7F, 67),
            ]
        );

        assert_eq!(idle(&mut schedule, 1), vec![stop(0, 60)]);
        assert_eq!(idle(&mut schedule, 2), vec![stop(0, 64)]);
        assert_eq!(idle(&mut schedule, 3), vec![stop(0, 67)]);
        assert_eq!(idle(&mut schedule, 4), Vec::new());
    }

    #[test]
    fn control_change_and_pitch_bend_are_delivered_unresolved_and_in_tick_plan_order() {
        let mut schedule = OwnedNotes::default();

        // Neither spelling has a lifetime, so there is nothing here to resolve
        // and nothing to schedule: the two commands are carried through in
        // Tick Plan order, and the Ticks after them owe nothing at all. Every
        // operand differs from every other, here as in the Function's own role
        // test, so a transposition anywhere along the way changes this list.
        assert_eq!(
            plan(
                &mut schedule,
                0,
                &[
                    PlayCommand::ControlChange {
                        channel: MidiChannel::try_from(1).expect("a MIDI channel"),
                        controller: Controller::try_from(2).expect("a MIDI data byte"),
                        value: ControlValue::try_from(7).expect("a MIDI data byte"),
                    },
                    PlayCommand::PitchBend {
                        channel: MidiChannel::try_from(3).expect("a MIDI channel"),
                        lsb: BendLsb::try_from(0x2A).expect("a MIDI data byte"),
                        msb: BendMsb::try_from(0x33).expect("a MIDI data byte"),
                    },
                ],
            ),
            vec![control_change(1, 2, 7), pitch_bend(3, 0x2A, 0x33)]
        );
        assert_eq!(idle(&mut schedule, 1), Vec::new());
        assert_eq!(idle(&mut schedule, 2), Vec::new());
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn a_timed_play_starts_in_tick_plan_order_and_stops_at_the_tick_its_length_names() {
        let mut schedule = OwnedNotes::default();

        // The start is delivered in the Tick that planned it and the stop at
        // the beginning of Tick `0 + 02`, with the Tick between them carrying
        // neither.
        assert_eq!(
            plan(&mut schedule, 0, &[timed_play(0, 0x7F, 60, 2)]),
            vec![note_on(0, 0x7F, 60)]
        );
        assert_eq!(idle(&mut schedule, 1), Vec::new());
        assert_eq!(idle(&mut schedule, 2), vec![stop(0, 60)]);
        assert_eq!(idle(&mut schedule, 3), Vec::new());
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn a_repeated_timed_play_stops_the_instance_it_replaces_and_retires_its_expiry() {
        let mut schedule = OwnedNotes::default();

        // The same command every Tick, replacing the note instance owned by
        // the previous Tick.
        assert_eq!(
            plan(&mut schedule, 0, &[timed_play(0, 0x7F, 60, 3)]),
            vec![note_on(0, 0x7F, 60)]
        );
        assert_eq!(
            plan(&mut schedule, 1, &[timed_play(0, 0x7F, 60, 3)]),
            vec![stop(0, 60), note_on(0, 0x7F, 60)]
        );
        assert_eq!(
            plan(&mut schedule, 2, &[timed_play(0, 0x7F, 60, 3)]),
            vec![stop(0, 60), note_on(0, 0x7F, 60)]
        );

        // Ticks 3 and 4 are where the first two commands scheduled their
        // stops. Both claims were retired by the replacement that followed
        // them, so neither stop is delivered — and only the surviving claim,
        // from Tick 2, stops at Tick 5.
        assert_eq!(idle(&mut schedule, 3), Vec::new());
        assert_eq!(idle(&mut schedule, 4), Vec::new());
        assert_eq!(idle(&mut schedule, 5), vec![stop(0, 60)]);
    }

    #[test]
    fn a_timed_play_with_velocity_zero_stops_the_note_and_schedules_nothing() {
        let mut schedule = OwnedNotes::default();

        // A stop still carries and validates its length, and the length still
        // schedules nothing: ADR 0016 keeps the arity fixed either way.
        assert_eq!(
            plan(&mut schedule, 0, &[timed_play(0, 0x00, 60, 5)]),
            vec![stop(0, 60)]
        );
        for tick in 1..=6 {
            assert_eq!(idle(&mut schedule, tick), Vec::new(), "Tick {tick}");
        }
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn a_timed_play_with_no_length_emits_nothing_and_leaves_the_note_it_finds_standing() {
        let mut schedule = OwnedNotes::default();

        assert_eq!(
            plan(&mut schedule, 0, &[timed_play(0, 0x7F, 60, 3)]),
            vec![note_on(0, 0x7F, 60)]
        );
        // A lifetime of no Ticks. It is not a stop, so the note started at
        // Tick 0 keeps both its sound and the stop it is due.
        assert_eq!(
            plan(&mut schedule, 1, &[timed_play(0, 0x7F, 60, 0)]),
            Vec::new()
        );
        assert_eq!(idle(&mut schedule, 2), Vec::new());
        assert_eq!(idle(&mut schedule, 3), vec![stop(0, 60)]);
    }

    #[test]
    fn a_stale_expiry_cannot_stop_the_note_claimed_after_it() {
        let mut schedule = OwnedNotes::default();

        // Tick 0 claims the voice until Tick 3. Tick 1 stops it explicitly,
        // which retires that claim while leaving its scheduled stop where it
        // was, and Tick 2 claims the same voice again until Tick 7.
        assert_eq!(
            plan(&mut schedule, 0, &[timed_play(0, 0x7F, 60, 3)]),
            vec![note_on(0, 0x7F, 60)]
        );
        assert_eq!(
            plan(&mut schedule, 1, &[timed_play(0, 0x00, 60, 3)]),
            vec![stop(0, 60)]
        );
        assert_eq!(
            plan(&mut schedule, 2, &[timed_play(0, 0x7F, 60, 5)]),
            vec![note_on(0, 0x7F, 60)]
        );

        // Tick 3 is where the first claim's stop was due. Delivering it here
        // would cut the note claimed at Tick 2 short by four Ticks, which is
        // exactly what its claim exists to prevent.
        for tick in 3..=6 {
            assert_eq!(idle(&mut schedule, tick), Vec::new(), "Tick {tick}");
        }
        assert_eq!(idle(&mut schedule, 7), vec![stop(0, 60)]);
    }

    #[test]
    fn a_stop_due_this_tick_is_delivered_before_the_play_commands_that_tick_plans() {
        let mut schedule = OwnedNotes::default();

        assert_eq!(
            plan(&mut schedule, 0, &[timed_play(0, 0x7F, 60, 1)]),
            vec![note_on(0, 0x7F, 60)]
        );

        // Tick 1 carries both a stop due from Tick 0 and a command of its own
        // for the voice that stop names. The order is the whole of what ADR
        // 0016 asks of the Tick a stop comes due at: delivered the other way
        // round, the note this Tick sounds is silenced by the stop of the note
        // it succeeds.
        assert_eq!(
            plan(&mut schedule, 1, &[raw_play(0, 0x7F, 60)]),
            vec![stop(0, 60), note_on(0, 0x7F, 60)]
        );
        // The Raw note that outlives the stop is the Source's to end.
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn two_notes_on_one_channel_are_owned_and_stopped_independently() {
        let mut schedule = OwnedNotes::default();

        assert_eq!(
            plan(&mut schedule, 0, &[timed_play(0, 0x7F, 60, 3)]),
            vec![note_on(0, 0x7F, 60)]
        );
        // A second note on the channel the first is sounding on. Timed Play is
        // polyphonic, and ADR 0016 gives one voice per channel to Monophonic
        // Play alone, so this starts a note rather than replacing one. Owned
        // per channel alone, this Tick would stop C4 to sound E4, cutting a
        // note the Source gave three Ticks short by two.
        assert_eq!(
            plan(&mut schedule, 1, &[timed_play(0, 0x7F, 64, 3)]),
            vec![note_on(0, 0x7F, 64)]
        );
        assert_eq!(idle(&mut schedule, 2), Vec::new());
        assert_eq!(idle(&mut schedule, 3), vec![stop(0, 60)]);
        assert_eq!(idle(&mut schedule, 4), vec![stop(0, 64)]);
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn one_note_on_two_channels_is_owned_and_stopped_independently() {
        let mut schedule = OwnedNotes::default();

        assert_eq!(
            plan(&mut schedule, 0, &[timed_play(0, 0x7F, 60, 3)]),
            vec![note_on(0, 0x7F, 60)]
        );
        // The same note on a second channel, which is a second instrument
        // sounding it: the channel discriminates as the note does.
        assert_eq!(
            plan(&mut schedule, 1, &[timed_play(1, 0x7F, 60, 3)]),
            vec![note_on(1, 0x7F, 60)]
        );
        assert_eq!(idle(&mut schedule, 2), Vec::new());
        assert_eq!(idle(&mut schedule, 3), vec![stop(0, 60)]);
        assert_eq!(idle(&mut schedule, 4), vec![stop(1, 60)]);
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn two_timed_plays_for_one_voice_within_one_tick_leave_the_second_owning_it() {
        let mut schedule = OwnedNotes::default();

        // The second command replaces what the first started, inside the one
        // delivery the Tick makes: ownership is resolved in Tick Plan order,
        // not once per Tick.
        assert_eq!(
            plan(
                &mut schedule,
                0,
                &[timed_play(0, 0x7F, 60, 5), timed_play(0, 0x7F, 60, 2)],
            ),
            vec![note_on(0, 0x7F, 60), stop(0, 60), note_on(0, 0x7F, 60)]
        );
        assert_eq!(idle(&mut schedule, 1), Vec::new());
        assert_eq!(idle(&mut schedule, 2), vec![stop(0, 60)]);
        // Tick 5 is where the first command's stop was due. Its claim was
        // retired before the Tick that scheduled it had ended.
        for tick in 3..=5 {
            assert_eq!(idle(&mut schedule, tick), Vec::new(), "Tick {tick}");
        }
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn a_monophonic_play_starts_in_tick_plan_order_and_stops_at_the_tick_its_length_names() {
        let mut schedule = OwnedNotes::default();

        // ADR 0016 gives `!%` Timed Play's lifetime as well as its operands:
        // the start is delivered in the Tick that planned it and the stop at
        // the beginning of Tick `0 + 02`.
        assert_eq!(
            plan(&mut schedule, 0, &[mono_play(0, 0x7F, 60, 2)]),
            vec![note_on(0, 0x7F, 60)]
        );
        assert_eq!(idle(&mut schedule, 1), Vec::new());
        assert_eq!(idle(&mut schedule, 2), vec![stop(0, 60)]);
        assert_eq!(idle(&mut schedule, 3), Vec::new());
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn a_monophonic_play_stops_whatever_note_its_channel_was_sounding() {
        let mut schedule = OwnedNotes::default();

        assert_eq!(
            plan(&mut schedule, 0, &[mono_play(0, 0x7F, 60, 5)]),
            vec![note_on(0, 0x7F, 60)]
        );
        // A different note on the channel the first is sounding. Mono
        // ownership is keyed by channel alone, so this replaces the voice
        // rather than joining it, and what it stops is the note the claim
        // recorded rather than the note this command names. Keyed as Timed
        // Play is, by channel and note, this Tick would start E4 over a C4
        // that nothing would stop until Tick 5.
        assert_eq!(
            plan(&mut schedule, 1, &[mono_play(0, 0x7F, 64, 5)]),
            vec![stop(0, 60), note_on(0, 0x7F, 64)]
        );
        // Tick 5 is where the replaced claim's stop was due. Its generation
        // token was retired at Tick 1, so delivering it there would cut the
        // replacement short by a Tick.
        for tick in 2..=5 {
            assert_eq!(idle(&mut schedule, tick), Vec::new(), "Tick {tick}");
        }
        assert_eq!(idle(&mut schedule, 6), vec![stop(0, 64)]);
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn a_monophonic_play_with_velocity_zero_replaces_the_voice_with_silence() {
        let mut schedule = OwnedNotes::default();

        assert_eq!(
            plan(&mut schedule, 0, &[mono_play(0, 0x7F, 60, 5)]),
            vec![note_on(0, 0x7F, 60)]
        );
        // Velocity `00`, and a note operand that is not the note sounding, so
        // the stop can only have come from the claim.
        assert_eq!(
            plan(&mut schedule, 1, &[mono_play(0, 0x00, 69, 5)]),
            vec![stop(0, 60)]
        );
        for tick in 2..=6 {
            assert_eq!(idle(&mut schedule, tick), Vec::new(), "Tick {tick}");
        }
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn a_monophonic_stop_on_a_channel_it_never_owned_delivers_nothing() {
        let mut schedule = OwnedNotes::default();

        // Velocity `00` on a channel this schedule holds no claim on. Timed
        // Play's velocity `00` is an explicit stop and is delivered whether or
        // not a claim stands, because the Source named the note it stops.
        // Monophonic Play stops the note its claim recorded, so with no claim
        // there is no note to name and nothing to send: the voice was already
        // silent, and a Note Off here would stop whatever else is sounding
        // that pitch on the channel.
        assert_eq!(
            plan(&mut schedule, 0, &[mono_play(0, 0x00, 60, 5)]),
            Vec::new()
        );
        for tick in 1..=5 {
            assert_eq!(idle(&mut schedule, tick), Vec::new(), "Tick {tick}");
        }
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn a_monophonic_play_with_no_length_replaces_the_voice_with_silence() {
        let mut schedule = OwnedNotes::default();

        assert_eq!(
            plan(&mut schedule, 0, &[mono_play(0, 0x7F, 60, 5)]),
            vec![note_on(0, 0x7F, 60)]
        );
        // The operand Timed Play treats as a no-op. Monophonic Play claims its
        // channel rather than its note, so a command that starts nothing has
        // still replaced the voice — with silence — and the note it replaced
        // is stopped rather than left standing until its own expiry.
        assert_eq!(
            plan(&mut schedule, 1, &[mono_play(0, 0x7F, 60, 0)]),
            vec![stop(0, 60)]
        );
        for tick in 2..=6 {
            assert_eq!(idle(&mut schedule, tick), Vec::new(), "Tick {tick}");
        }
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn a_stale_mono_expiry_cannot_stop_the_voice_claimed_after_it() {
        let mut schedule = OwnedNotes::default();

        // Tick 0 claims the channel until Tick 3. Tick 1 silences it, which
        // retires that claim while leaving its scheduled stop where it was,
        // and Tick 2 claims the same channel again until Tick 7.
        assert_eq!(
            plan(&mut schedule, 0, &[mono_play(0, 0x7F, 60, 3)]),
            vec![note_on(0, 0x7F, 60)]
        );
        assert_eq!(
            plan(&mut schedule, 1, &[mono_play(0, 0x00, 60, 3)]),
            vec![stop(0, 60)]
        );
        assert_eq!(
            plan(&mut schedule, 2, &[mono_play(0, 0x7F, 60, 5)]),
            vec![note_on(0, 0x7F, 60)]
        );

        // Tick 3 is where the first claim's stop was due. Delivering it here
        // would cut the note claimed at Tick 2 short by four Ticks, which is
        // exactly what its token exists to prevent.
        for tick in 3..=6 {
            assert_eq!(idle(&mut schedule, tick), Vec::new(), "Tick {tick}");
        }
        assert_eq!(idle(&mut schedule, 7), vec![stop(0, 60)]);
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn a_mono_voice_is_owned_per_channel_and_channels_do_not_steal_from_one_another() {
        let mut schedule = OwnedNotes::default();

        assert_eq!(
            plan(&mut schedule, 0, &[mono_play(0, 0x7F, 60, 3)]),
            vec![note_on(0, 0x7F, 60)]
        );
        // The same note on a second channel, which is a second instrument
        // sounding it. One voice per channel is one voice each: a single Mono
        // voice across every channel would stop channel 0 here to sound
        // channel 1, one Tick into a note the Source gave three.
        assert_eq!(
            plan(&mut schedule, 1, &[mono_play(1, 0x7F, 60, 3)]),
            vec![note_on(1, 0x7F, 60)]
        );
        assert_eq!(idle(&mut schedule, 2), Vec::new());
        assert_eq!(idle(&mut schedule, 3), vec![stop(0, 60)]);
        assert_eq!(idle(&mut schedule, 4), vec![stop(1, 60)]);
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn two_monophonic_plays_for_one_channel_within_one_tick_leave_the_second_owning_it() {
        let mut schedule = OwnedNotes::default();

        // The second command replaces what the first started, inside the one
        // delivery the Tick makes: the channel is owned in Tick Plan order,
        // not once per Tick.
        assert_eq!(
            plan(
                &mut schedule,
                0,
                &[mono_play(0, 0x7F, 60, 5), mono_play(0, 0x7F, 64, 2)],
            ),
            vec![note_on(0, 0x7F, 60), stop(0, 60), note_on(0, 0x7F, 64)]
        );
        assert_eq!(idle(&mut schedule, 1), Vec::new());
        assert_eq!(idle(&mut schedule, 2), vec![stop(0, 64)]);
        // Tick 5 is where the first command's stop was due. Its claim was
        // retired before the Tick that scheduled it had ended.
        for tick in 3..=5 {
            assert_eq!(idle(&mut schedule, tick), Vec::new(), "Tick {tick}");
        }
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn a_mono_voice_due_to_expire_is_stopped_once_by_the_tick_that_replaces_it() {
        let mut schedule = OwnedNotes::default();

        // A lifetime of one Tick, replayed every Tick, so every Tick after the
        // first carries both a due stop and a command for the voice that stop
        // names.
        //
        // One stop, not two: the expiry drains before the Tick Plan and takes
        // the claim with it, so the command that follows finds nothing left to
        // release and the note it starts is not immediately silenced.
        assert_eq!(
            plan(&mut schedule, 0, &[mono_play(0, 0x7F, 60, 1)]),
            vec![note_on(0, 0x7F, 60)]
        );
        assert_eq!(
            plan(&mut schedule, 1, &[mono_play(0, 0x7F, 60, 1)]),
            vec![stop(0, 60), note_on(0, 0x7F, 60)]
        );
        assert_eq!(
            plan(&mut schedule, 2, &[mono_play(0, 0x7F, 60, 1)]),
            vec![stop(0, 60), note_on(0, 0x7F, 60)]
        );
    }

    #[test]
    fn timed_and_mono_own_separately_and_neither_owns_a_raw_note() {
        let mut schedule = OwnedNotes::default();

        // The same note on the same channel, started a Tick apart by all three
        // Play spellings. One channel and one note is the whole point: a
        // schedule that keyed the two owning spellings together would find a
        // claim to replace here, where ADR 0016 gives Timed and Mono
        // ownerships that cannot see one another.
        assert_eq!(
            plan(&mut schedule, 0, &[raw_play(0, 0x7F, 60)]),
            vec![note_on(0, 0x7F, 60)]
        );
        // The Timed command owns nothing yet, and the Raw note is the Source's
        // to end, so nothing is stopped to start this.
        assert_eq!(
            plan(&mut schedule, 1, &[timed_play(0, 0x7F, 60, 3)]),
            vec![note_on(0, 0x7F, 60)]
        );
        // Nor does the Mono command find the Timed claim beside it.
        assert_eq!(
            plan(&mut schedule, 2, &[mono_play(0, 0x7F, 60, 3)]),
            vec![note_on(0, 0x7F, 60)]
        );
        assert_eq!(idle(&mut schedule, 3), Vec::new());
        // Two lifetimes were written and two stops are delivered, one per
        // owning spelling. Sharing a key would deliver one.
        assert_eq!(idle(&mut schedule, 4), vec![stop(0, 60)]);
        assert_eq!(idle(&mut schedule, 5), vec![stop(0, 60)]);
        assert_eq!(idle(&mut schedule, 6), Vec::new());
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn raw_play_notes_never_enter_timed_ownership() {
        let mut schedule = OwnedNotes::default();

        // Raw Play leaves Note Off under Source control, so nothing this
        // schedule owns can stop a note the Source did not ask to stop.
        assert_eq!(
            plan(&mut schedule, 0, &[raw_play(0, 0x7F, 60)]),
            vec![note_on(0, 0x7F, 60)]
        );
        for tick in 1..=4 {
            assert_eq!(idle(&mut schedule, tick), Vec::new(), "Tick {tick}");
        }
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn two_mono_claims_due_at_one_tick_stop_only_the_voice_still_standing() {
        let mut schedule = OwnedNotes::default();

        // Two Monophonic claims on one channel whose lifetimes end at the same
        // Tick: `03` from Tick 0 and `02` from Tick 1 are both due at Tick 3.
        // Only the second is still standing, so Tick 3 holds two expiries for
        // one voice, the stale one first.
        assert_eq!(
            plan(&mut schedule, 0, &[mono_play(0, 0x7F, 60, 3)]),
            vec![note_on(0, 0x7F, 60)]
        );
        assert_eq!(
            plan(&mut schedule, 1, &[mono_play(0, 0x7F, 64, 2)]),
            vec![stop(0, 60), note_on(0, 0x7F, 64)]
        );
        assert_eq!(idle(&mut schedule, 2), Vec::new());

        // One stop, for the note the surviving claim recorded rather than for
        // the note the stale claim started, and the voice retired with it.
        // Two expiries reach one voice here, so both halves of the refusal are
        // load-bearing: the claim comparison declines the stale one, and
        // draining takes the voice out of `voices` so the live one cannot be
        // read twice. Without either, E4 is stopped twice in the one delivery.
        assert_eq!(idle(&mut schedule, 3), vec![stop(0, 64)]);
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn a_lifetime_that_saturates_the_last_tick_leaves_a_stop_no_tick_can_reach() {
        let mut schedule = OwnedNotes::default();

        // `Tick::after` saturates rather than wraps, so a length added to a
        // Tick near the end of the counter names the last Tick rather than
        // returning to the beginning of the run, where it would stop a note
        // that has not started.
        let last = u64::MAX;
        assert_eq!(
            plan(&mut schedule, last - 1, &[timed_play(0, 0x7F, 60, 5)]),
            vec![note_on(0, 0x7F, 60)]
        );

        // `expired_at` drains everything strictly before the Tick after the
        // one it is asked for, and the Tick after the last Tick is the last
        // Tick, so this one stop is beyond every Tick the counter can reach.
        // Its doc comment says so; a run whose absolute Tick has stopped
        // advancing has already lost more than a Note Off.
        assert_eq!(idle(&mut schedule, last), Vec::new());
        assert_eq!(idle(&mut schedule, last), Vec::new());
        assert!(schedule.holds_note_ownership());
    }

    #[test]
    fn a_delivery_drains_every_stop_due_before_it_and_not_only_this_tick_s() {
        let mut schedule = OwnedNotes::default();

        // Two lifetimes ending at two different Ticks, neither of which the
        // next delivery is for. `expired_at` drains the whole range up to the
        // Tick it is asked for, which is what keeps an expiry from outliving
        // the Tick it was due at by the Ticks a future scheduling rule might
        // skip.
        assert_eq!(
            plan(
                &mut schedule,
                0,
                &[timed_play(0, 0x7F, 60, 2), timed_play(0, 0x7F, 64, 4)],
            ),
            vec![note_on(0, 0x7F, 60), note_on(0, 0x7F, 64)]
        );

        // Both stops, in the Tick order they were due at rather than the order
        // they were claimed in, delivered by the one Tick that passes them.
        assert_eq!(idle(&mut schedule, 9), vec![stop(0, 60), stop(0, 64)]);
        assert!(!schedule.holds_note_ownership());
    }

    #[test]
    fn clearing_forgets_every_claim_and_every_scheduled_stop() {
        let mut schedule = OwnedNotes::default();

        assert_eq!(
            plan(
                &mut schedule,
                0,
                &[timed_play(0, 0x7F, 60, 3), mono_play(1, 0x7F, 64, 3)],
            ),
            vec![note_on(0, 0x7F, 60), note_on(1, 0x7F, 64)]
        );
        assert!(schedule.holds_note_ownership());

        schedule.clear();

        assert!(!schedule.holds_note_ownership());
        // Nothing is left to come due: the stops both claims scheduled went
        // with the claims.
        assert_eq!(idle(&mut schedule, 3), Vec::new());
    }
}
