//! Chord Play through the Playback Engine.
//!
//! Each tone is an ordinary Raw or Timed Play Command, so the schedule treats
//! it exactly as it treats a note. These runs drive a real Source: the
//! Equality at (0, 0) Bangs the Chord Play on row 2 while it reads `.=0101`,
//! and stops Banging it once an edit makes it `.=0102`.

use super::{HandDrivenRun, commander, note_on, stop, write_at};
use crate::playback::{InMemoryOutputAdapter, OutputCommand};
use crate::source::SourceCommander;

/// A run with `expression` on row 2, beneath the Equality that Bangs it.
fn run_of(
    expression: &str,
) -> (
    HandDrivenRun<InMemoryOutputAdapter>,
    SourceCommander,
    InMemoryOutputAdapter,
) {
    let source = commander(&[".=0101", "", expression]);
    let adapter = InMemoryOutputAdapter::default();
    let mut run = HandDrivenRun::new(source.clone(), adapter.clone());
    run.begin_run();
    (run, source, adapter)
}

/// Stops the Equality Banging the Chord Play on later Ticks.
fn stop_banging(source: &SourceCommander) {
    write_at(source, 0, 0, ".=0102");
}

/// The C4 major triad, in Note Numbers.
const C_MAJOR: [u8; 3] = [0x3C, 0x40, 0x43];

fn on_each(tones: &[u8]) -> Vec<OutputCommand> {
    tones.iter().map(|&tone| note_on(0, 0x7F, tone)).collect()
}

fn stop_each(tones: &[u8]) -> Vec<OutputCommand> {
    tones.iter().map(|&tone| stop(0, tone)).collect()
}

#[test]
fn each_tone_of_a_timed_chord_starts_in_tick_plan_order_and_stops_at_its_own_note_off() {
    let (mut run, source, adapter) = run_of("!#007FC4MACL0002");

    run.run_tick(0);
    stop_banging(&source);
    run.run_tick(1);
    assert!(run.holds_note_ownership());
    run.run_tick(2);

    assert_eq!(
        adapter.command_lists(),
        vec![on_each(&C_MAJOR), vec![], stop_each(&C_MAJOR)]
    );
    assert!(!run.holds_note_ownership());
}

#[test]
fn velocity_zero_stops_every_tone_and_schedules_nothing() {
    let (mut run, source, adapter) = run_of("!#007FC4MACL0008");

    run.run_tick(0);
    // Velocity is the operand after the channel: `00` stops every tone,
    // whatever the length says.
    write_at(&source, 4, 2, "00");
    run.run_tick(1);
    stop_banging(&source);
    for tick in 2..10 {
        run.run_tick(tick);
    }

    let lists = adapter.command_lists();
    assert_eq!(lists[0], on_each(&C_MAJOR));
    assert_eq!(lists[1], stop_each(&C_MAJOR));
    assert!(lists[2..].iter().all(Vec::is_empty), "{lists:?}");
    assert!(!run.holds_note_ownership());
}

#[test]
fn length_zero_starts_nothing_and_leaves_sounding_tones_standing() {
    let (mut run, source, adapter) = run_of("!#007FC4MACL0003");

    run.run_tick(0);
    write_at(&source, 14, 2, "00");
    run.run_tick(1);
    assert!(run.holds_note_ownership(), "the tones keep sounding");
    run.run_tick(2);
    run.run_tick(3);

    assert_eq!(
        adapter.command_lists(),
        vec![on_each(&C_MAJOR), vec![], vec![], stop_each(&C_MAJOR)]
    );
}

#[test]
fn a_repeated_chord_replaces_the_tones_it_repeats_and_retires_their_expiries() {
    let (mut run, source, adapter) = run_of("!#007FC4MACL0002");

    run.run_tick(0);
    run.run_tick(1);
    stop_banging(&source);
    run.run_tick(2);
    run.run_tick(3);

    let replacement: Vec<OutputCommand> = C_MAJOR
        .iter()
        .flat_map(|&tone| [stop(0, tone), note_on(0, 0x7F, tone)])
        .collect();
    // Tick 0's stops were due at Tick 2 and the replacement retired them, so
    // only Tick 1's lifetime stops, at Tick 3.
    assert_eq!(
        adapter.command_lists(),
        vec![on_each(&C_MAJOR), replacement, vec![], stop_each(&C_MAJOR)]
    );
}

#[test]
fn a_chord_overlapping_another_by_some_tones_releases_only_those() {
    let (mut run, source, adapter) = run_of("!#007FC4MACL0004");

    run.run_tick(0);
    // E4 major shares only E4 with C4 major: the replacement stops that one
    // tone, and the C4 and G4 of the first chord run to their own stops.
    write_at(&source, 6, 2, "E4");
    run.run_tick(1);
    stop_banging(&source);
    for tick in 2..6 {
        run.run_tick(tick);
    }

    // The second chord's tones, 40 44 47, are claimed at Tick 1 and last
    // until Tick 5; the first chord's 3C and 43 keep their Tick 0 claims and
    // stop at Tick 4.
    assert_eq!(
        adapter.command_lists(),
        vec![
            on_each(&C_MAJOR),
            vec![
                stop(0, 0x40),
                note_on(0, 0x7F, 0x40),
                note_on(0, 0x7F, 0x44),
                note_on(0, 0x7F, 0x47),
            ],
            vec![],
            vec![],
            vec![stop(0, 0x3C), stop(0, 0x43)],
            vec![stop(0, 0x40), stop(0, 0x44), stop(0, 0x47)],
        ]
    );
}

#[test]
fn a_raw_chord_play_delivers_one_note_on_per_tone_and_owns_nothing() {
    let (mut run, _source, adapter) = run_of("!:007FC4MACL00");

    run.run_tick(0);

    assert_eq!(adapter.command_lists(), vec![on_each(&C_MAJOR)]);
    assert!(!run.holds_note_ownership());
}

#[test]
fn every_lifecycle_action_that_silences_output_clears_a_sounding_chord() {
    // A chord is only ordinary claims, so these are the lifecycle actions the
    // schedule answers for a note. Asserted for a chord because the claims are
    // several, and a clear that took only the first would leave tones hanging.
    let sounding = || {
        let (mut run, _source, adapter) = run_of("!#007FC4MACL000A");
        run.run_tick(0);
        assert!(run.holds_note_ownership());
        (run, adapter)
    };

    let (mut run, adapter) = sounding();
    run.inner.stop();
    assert!(!run.holds_note_ownership());
    assert_eq!(adapter.safety_reset_count(), 1, "stopping");

    let (mut run, adapter) = sounding();
    run.inner.disconnect();
    assert!(!run.holds_note_ownership());
    assert_eq!(adapter.safety_reset_count(), 1, "disconnecting");

    // A live run is rewound by stopping it and beginning the next.
    let (mut run, adapter) = sounding();
    run.inner.stop();
    run.inner.rewind();
    run.begin_run();
    assert!(!run.holds_note_ownership(), "rewinding");
    assert_eq!(adapter.safety_reset_count(), 1, "rewinding");

    // Tearing the run down still sends the safety action and no Note Off.
    let (run, adapter) = sounding();
    drop(run);
    assert_eq!(adapter.safety_reset_count(), 1, "dropping");
    assert_eq!(adapter.command_lists(), vec![on_each(&C_MAJOR)]);
}

#[test]
fn a_chord_and_a_monophonic_play_on_one_channel_keep_separate_ownership() {
    // Row 2 holds a chord and a Mono Play, each Banged by the Equality above
    // it on row 0.
    let rows = [
        ".=0101              .=0101",
        "",
        "!#007FC4MACL0008    !%007FC404  ",
    ];
    let mono = rows[2].find("!%").expect("a Mono Play on row 2");
    assert_eq!(rows[0].rfind(".="), Some(mono), "the Mono Play's Equality");
    // The note follows the Mono Play's spelling, channel and velocity.
    let mono_note = mono + "!%007F".len();
    let source = commander(&rows);
    let adapter = InMemoryOutputAdapter::default();
    let mut run = HandDrivenRun::new(source.clone(), adapter.clone());
    run.begin_run();

    run.run_tick(0);
    // The chord's Bang stops. The Mono Play goes on and changes its note, so
    // it replaces only its own voice: the chord's C4 is not the Mono C4.
    write_at(&source, 0, 0, ".=0102");
    write_at(&source, mono_note, 2, "D4");
    run.run_tick(1);
    write_at(&source, mono, 0, ".=0102");
    for tick in 2..9 {
        run.run_tick(tick);
    }

    let lists = adapter.command_lists();
    let mut expected_start = on_each(&C_MAJOR);
    expected_start.push(note_on(0, 0x7F, 0x3C));
    assert_eq!(lists[0], expected_start);
    // The Mono voice stops its own C4 before D4, and the chord loses nothing.
    assert_eq!(lists[1], vec![stop(0, 0x3C), note_on(0, 0x7F, 0x3E)]);
    assert_eq!(lists[5], vec![stop(0, 0x3E)], "the Mono D4's lifetime");
    assert_eq!(lists[8], stop_each(&C_MAJOR), "the chord's lifetime");
    assert!(!run.holds_note_ownership());
}
