//! The Presets Source File in `examples/`, loaded exactly as shipped and run
//! through Source/Tick and Playback, and again after each edit
//! `examples/presets.md` advertises.
//!
//! Every Tick a bassline plays its next step on the Built-in Synth's acid
//! channel, and two controls each send one Control Change on that channel:
//! cutoff and resonance. Each control's Slew moves its own value toward the
//! target the preset selector picks, by at most its own `amount` a Tick. The
//! expected values are written out Tick by Tick rather than derived, so a
//! misreading of the Slew shared by the test and Playback cannot pass both.

use super::{HandDrivenRun, assert_guide_draws, note_on, replace_at, stop, tick_diagnostics};
use crate::playback::{InMemoryOutputAdapter, OutputCommand};
use crate::source::{ControlValue, Controller, MidiChannel, SourceCommander, file};

/// The shipped Presets Source File's text.
const PRESETS: &str = include_str!("../../../../examples/presets.orcvs");

/// The guide beside it, whose diagrams the shipped and edited rows must match.
const GUIDE: &str = include_str!("../../../../examples/presets.md");

/// The acid channel, where the bassline plays and every control sends.
const ACID: u8 = 0x08;

/// The acid voice's controllers: cutoff, resonance and decay.
const CUTOFF: u8 = 0x4A;
const RESONANCE: u8 = 0x47;
const DECAY: u8 = 0x4B;

/// The bassline's eight steps as MIDI notes, one a Tick: A1 A1 A2 A1 C2 A1
/// G2 E2, each at velocity `64` for one Tick.
const BASS: [u8; 8] = [33, 33, 45, 33, 36, 33, 43, 40];

/// The bassline's velocity, below `78`, so no step is accented.
const BASS_VELOCITY: u8 = 0x64;

/// The selector: the Modulo's first operand, whose second Cell the guide
/// has you type over.
const SELECTOR_ROW: usize = 15;
const SELECTOR_COLUMN: usize = 4;

/// The rows of the cutoff and resonance Control Changes.
const CUT_ROW: usize = 19;
const RES_ROW: usize = 25;

/// The Control Change the acid channel is sent for `controller` at `value`.
fn change(controller: u8, value: u8) -> OutputCommand {
    OutputCommand::ControlChange {
        channel: MidiChannel::try_from(ACID).expect("a MIDI channel"),
        controller: Controller::try_from(controller).expect("a MIDI data byte"),
        value: ControlValue::try_from(value).expect("a MIDI data byte"),
    }
}

///
/// The bassline's part of Tick `tick`'s delivery: the Note Off of the step
/// before, whose one-Tick lifetime ends here, then the step's Note On.
///
fn bass(tick: u64) -> Vec<OutputCommand> {
    let step = |tick: u64| BASS[(tick % 8) as usize];
    let mut delivery = Vec::new();
    if tick > 0 {
        delivery.push(stop(ACID, step(tick - 1)));
    }
    delivery.push(note_on(ACID, BASS_VELOCITY, step(tick)));
    delivery
}

///
/// What a Tick delivers when the bassline plays and the controls send
/// `values`, each a controller and a value, in Source order.
///
fn delivery(tick: u64, values: &[(u8, u8)]) -> Vec<OutputCommand> {
    let mut delivery = bass(tick);
    delivery.extend(
        values
            .iter()
            .map(|&(controller, value)| change(controller, value)),
    );
    delivery
}

///
/// A hand-driven Playback run over one Source, counting the Ticks it runs.
///
struct Run {
    source: SourceCommander,
    adapter: InMemoryOutputAdapter,
    run: HandDrivenRun<InMemoryOutputAdapter>,
    /// The absolute Tick the run executes next.
    next: u64,
    /// How many Ticks have delivered, across Rewinds.
    delivered: usize,
}

impl Run {
    /// A run begun over `text`, with nothing executed yet.
    fn open(text: &str) -> Self {
        let source = SourceCommander::with_source(
            file::read(text.as_bytes()).expect("the Presets is a Source File"),
        );
        let adapter = InMemoryOutputAdapter::default();
        let mut run = HandDrivenRun::new(source.clone(), adapter.clone());
        run.begin_run();
        Self {
            source,
            adapter,
            run,
            next: 0,
            delivered: 0,
        }
    }

    ///
    /// Runs the next Tick and asserts it delivers the bassline's step and
    /// `values`, with nothing diagnosed on the Tick and the Language Map
    /// diagnosing what it diagnosed before it.
    ///
    fn expect(&mut self, values: &[(u8, u8)]) {
        let tick = self.next;
        let before = mapped(&self.source);
        let plan = self.run.run_tick(tick);
        assert_eq!(tick_diagnostics(&plan), vec![], "Tick {tick}'s diagnostics");
        assert_eq!(
            mapped(&self.source),
            before,
            "the Cells Tick {tick} wrote changed the Language Map"
        );
        assert_eq!(
            self.adapter.command_lists().last(),
            Some(&delivery(tick, values)),
            "Tick {tick}'s delivery"
        );
        self.next += 1;
        self.delivered += 1;
        assert_eq!(
            self.adapter.command_lists().len(),
            self.delivered,
            "deliveries after Tick {tick}"
        );
    }

    ///
    /// Runs the next Tick and asserts `CUT`'s Control Change is diagnosed for
    /// its Slew's `value` and sends nothing, while the bassline plays and
    /// `RES` sends `resonance`.
    ///
    fn expect_cutoff_out_of_range(&mut self, value: &str, resonance: u8) {
        let tick = self.next;
        let plan = self.run.run_tick(tick);
        assert_eq!(
            tick_diagnostics(&plan),
            vec![(
                2,
                CUT_ROW,
                format!("MIDI value {value} is outside the range 00–7F")
            )],
            "Tick {tick}'s diagnostics"
        );
        assert_eq!(
            self.adapter.command_lists().last(),
            Some(&delivery(tick, &[(RESONANCE, resonance)])),
            "Tick {tick}'s delivery"
        );
        self.next += 1;
        self.delivered += 1;
        assert_eq!(
            self.adapter.command_lists().len(),
            self.delivered,
            "deliveries after Tick {tick}"
        );
    }

    ///
    /// Runs the next Tick once for each of `cutoffs` and `resonances`, which
    /// must be as long as each other, asserting each Tick delivers them.
    ///
    fn expect_controls(&mut self, cutoffs: &[u8], resonances: &[u8]) {
        assert_eq!(cutoffs.len(), resonances.len(), "one value per Tick");
        for (&cutoff, &resonance) in cutoffs.iter().zip(resonances) {
            self.expect(&[(CUTOFF, cutoff), (RESONANCE, resonance)]);
        }
    }

    ///
    /// Stops the run and Rewinds it, as the Panel's `R` does to a stopped
    /// Playback, then begins the next run, whose first Tick is Tick `0`.
    ///
    fn rewind(&mut self) {
        self.run.inner.stop();
        self.run.inner.rewind();
        self.run.begin_run();
        self.next = 0;
    }

    ///
    /// Types `text` into the Source from (`column`, `row`) eastward, between
    /// Ticks, as a person editing while it plays does.
    ///
    fn type_at(&self, column: usize, row: usize, text: &str) {
        let grid = self.source.grid();
        for (offset, character) in text.chars().enumerate() {
            let position = grid
                .position(column + offset, row)
                .expect("inside the Grid");
            self.source
                .set(grid.index(position), &character.to_string())
                .expect("a Cell");
        }
    }

    /// Types `pair` over the selector.
    fn select(&self, pair: &str) {
        self.type_at(SELECTOR_COLUMN, SELECTOR_ROW, pair);
    }

    /// The text of `row` as the Source holds it now, trailing spaces trimmed.
    fn row(&self, row: usize) -> String {
        let mut line = String::new();
        self.source.read_source(|source| {
            line = file::write(source)
                .lines()
                .nth(row)
                .unwrap_or_default()
                .to_owned();
        });
        line
    }
}

///
/// `text` with each of `edits`, a row, a column, the Cells there and what
/// replaces them, applied in order: edits the guide documents, applied to the
/// shipped file. A row past the file's end is added empty first.
///
fn edited(text: &str, edits: &[(usize, usize, &str, &str)]) -> String {
    let mut rows: Vec<String> = text.lines().map(str::to_owned).collect();
    for &(row, column, before, after) in edits {
        if rows.len() <= row {
            rows.resize(row + 1, String::new());
        }
        replace_at(&mut rows, row, column, before, after);
    }
    let mut text = rows.join("\n");
    text.push('\n');
    text
}

///
/// Before the first Tick the Language Map diagnoses nothing: the bassline's
/// steps, the presets and the values the Functions write are Comment text,
/// and every other Cell is part of a Function. The guide draws every row of
/// the file as shipped.
///
#[test]
fn presets_open_with_nothing_diagnosed_and_draw_as_shipped() {
    let run = Run::open(PRESETS);
    run.source.read_source(|source| {
        assert_eq!(source.language_map().diagnostics().count(), 0);
    });
    for (row, line) in PRESETS.lines().enumerate() {
        assert_guide_draws(GUIDE, row, line);
    }
}

///
/// Preset `00` is in force from the first Tick: each control's Slew starts
/// from the value its Comment holds, which is preset `00`'s target, so it
/// sends that value on every Tick. Two loops of the bassline, with the
/// Language Map diagnosing nothing after any Tick, and each control's Comment
/// holding after each Tick what the file ships there.
///
#[test]
fn preset_00_is_delivered_from_the_first_tick_and_holds() {
    let mut run = Run::open(PRESETS);
    for tick in 0..16 {
        run.expect(&[(CUTOFF, 0x20), (RESONANCE, 0x20)]);
        assert_eq!(mapped(&run.source), vec![], "Tick {tick}'s Language Map");
        for row in [CUT_ROW + 1, RES_ROW + 1] {
            assert_eq!(run.row(row), "      ||20  2002  00||", "Tick {tick}");
        }
    }
}

///
/// The guide's preset change: `1` typed over the selector's second Cell
/// before Tick 4 turns both controls toward preset `01` on Tick 4. `CUT`
/// climbs `08` a Tick and arrives at `60` on Tick 11; `RES` climbs `05` a
/// Tick, lands on `50` with a last step of `03` on Tick 13, and both then
/// hold their targets.
///
#[test]
fn selecting_preset_01_turns_both_controls_toward_it_on_the_next_tick() {
    let mut run = Run::open(PRESETS);
    run.expect_controls(&[0x20; 4], &[0x20; 4]);
    run.type_at(SELECTOR_COLUMN + 1, SELECTOR_ROW, "1");
    assert_eq!(run.row(SELECTOR_ROW), "  .%0102");
    run.expect_controls(
        &[
            0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x60, 0x60, 0x60, 0x60,
        ],
        &[
            0x25, 0x2A, 0x2F, 0x34, 0x39, 0x3E, 0x43, 0x48, 0x4D, 0x50, 0x50, 0x50,
        ],
    );
    assert_eq!(run.row(CUT_ROW + 1), "      ||60  6003  01||");
    assert_eq!(run.row(RES_ROW + 1), "      ||50  5003  01||");
}

///
/// The guide's change back during a move: `0` typed over the selector
/// before Tick 13, when `CUT` has arrived at `60` and `RES` is at `4D` on
/// its way to `50`. Each turns from where it is: `CUT` covers `40` in 8
/// Ticks and `RES` covers `2D` in 9.
///
#[test]
fn changing_preset_during_a_move_turns_each_control_from_its_current_value() {
    let mut run = Run::open(PRESETS);
    run.expect_controls(&[0x20; 4], &[0x20; 4]);
    run.select("01");
    run.expect_controls(
        &[0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x60],
        &[0x25, 0x2A, 0x2F, 0x34, 0x39, 0x3E, 0x43, 0x48, 0x4D],
    );
    run.select("00");
    run.expect_controls(
        &[0x58, 0x50, 0x48, 0x40, 0x38, 0x30, 0x28, 0x20, 0x20, 0x20],
        &[0x48, 0x43, 0x3E, 0x39, 0x34, 0x2F, 0x2A, 0x25, 0x20, 0x20],
    );
}

///
/// The guide's hold: `0` typed over the `8` of `CUT`'s Slew `amount`, making
/// it `~>00`, holds `CUT` where it is through a preset change while `RES`
/// moves. Typing the `8` back sends `CUT` on toward the preset in force.
///
#[test]
fn an_amount_of_00_holds_its_control_through_a_preset_change() {
    let mut run = Run::open(PRESETS);
    run.expect_controls(&[0x20; 2], &[0x20; 2]);
    run.type_at(11, CUT_ROW, "0");
    assert_eq!(run.row(CUT_ROW), "**!c084A~>00&v.+02&$0210");
    run.select("01");
    run.expect_controls(&[0x20; 4], &[0x25, 0x2A, 0x2F, 0x34]);
    run.type_at(11, CUT_ROW, "8");
    run.expect_controls(&[0x28, 0x30, 0x38], &[0x39, 0x3E, 0x43]);
}

///
/// The selector wraps by the number of presets, `02`: `02` selects preset
/// `00`, and `03` and `FF` select preset `01`. The preset in force is what
/// each control reads and writes beside its distance.
///
#[test]
fn the_selector_wraps_by_the_number_of_presets() {
    let mut run = Run::open(PRESETS);
    run.select("02");
    run.expect_controls(&[0x20; 2], &[0x20; 2]);
    assert_eq!(run.row(SELECTOR_ROW + 1), "||00||");
    run.select("03");
    run.expect_controls(&[0x28, 0x30], &[0x25, 0x2A]);
    assert_eq!(run.row(SELECTOR_ROW + 1), "||01||");
    assert_eq!(run.row(CUT_ROW + 1), "      ||30  6003  01||");
    run.select("02");
    run.expect_controls(&[0x28, 0x20], &[0x25, 0x20]);
    run.select("FF");
    run.expect_controls(&[0x28], &[0x25]);
    assert_eq!(run.row(SELECTOR_ROW + 1), "||01||");
}

///
/// The Language Map's diagnoses of the Source as it stands, as anchor
/// column, anchor row and message, in Grid order.
///
fn mapped(source: &SourceCommander) -> Vec<(usize, usize, String)> {
    let mut diagnosed = Vec::new();
    source.read_source(|source| {
        diagnosed = source
            .language_map()
            .diagnostics()
            .map(|d| (d.anchor().x(), d.anchor().y(), d.message.clone()))
            .collect();
    });
    diagnosed.sort_by_key(|&(column, row, _)| (row, column));
    diagnosed
}

///
/// A selector that is not a Number is diagnosed on the Modulo, by the
/// diagnostics any Number operand gets: `0G`, and a selector whose second
/// Cell is emptied. The Modulo writes nothing, so the preset in force stays
/// and the controls carry on toward it; a Number typed back selects again.
///
#[test]
fn a_selector_that_is_not_a_number_is_diagnosed_and_keeps_the_preset_in_force() {
    let mut run = Run::open(PRESETS);
    run.select("01");
    run.expect_controls(&[0x28], &[0x25]);
    run.select("0G");
    let diagnosed = |found: &str, first: char, second: Option<char>| {
        let mut diagnosed = vec![(
            2,
            SELECTOR_ROW,
            format!("expected a number, found \"{found}\""),
        )];
        diagnosed.extend([(4, Some(first)), (5, second)].into_iter().filter_map(
            |(column, character)| {
                character.map(|character| {
                    (
                        column,
                        SELECTOR_ROW,
                        format!("invalid Language Unit character '{character}'"),
                    )
                })
            },
        ));
        diagnosed
    };
    assert_eq!(mapped(&run.source), diagnosed("0G", '0', Some('G')));
    run.expect_controls(&[0x30, 0x38], &[0x2A, 0x2F]);
    assert_eq!(run.row(SELECTOR_ROW + 1), "||01||");

    run.select("00");
    run.expect_controls(&[0x30], &[0x2A]);
    let grid = run.source.grid();
    let second = grid
        .position(SELECTOR_COLUMN + 1, SELECTOR_ROW)
        .expect("inside the Grid");
    run.source.unset(grid.index(second));
    assert_eq!(mapped(&run.source), diagnosed("0 ", '0', None));
    run.expect_controls(&[0x28, 0x20], &[0x25, 0x20]);
    assert_eq!(run.row(SELECTOR_ROW + 1), "||00||");

    run.select("01");
    assert_eq!(mapped(&run.source), vec![]);
    run.expect_controls(&[0x28], &[0x25]);
}

///
/// A target from `80` up is still a Number, so the Slew moves toward it, but
/// a value past `7F` is out of the Control Change's range: the Control
/// Change is diagnosed and sends nothing for that control, while the other
/// control and the bassline play on. A target back in range brings it back.
///
#[test]
fn a_target_past_7f_silences_its_control_with_a_diagnosis() {
    let text = edited(PRESETS, &[(CUT_ROW + 3, 14, "60", "90")]);
    let mut run = Run::open(&text);
    run.select("01");
    run.expect_controls(
        &[
            0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x68, 0x70, 0x78,
        ],
        &[
            0x25, 0x2A, 0x2F, 0x34, 0x39, 0x3E, 0x43, 0x48, 0x4D, 0x50, 0x50,
        ],
    );
    for value in ["80", "88", "90", "90"] {
        run.expect_cutoff_out_of_range(value, 0x50);
        assert_eq!(run.row(CUT_ROW + 1), format!("      ||{value}  9003  01||"));
    }
    run.select("00");
    for (value, resonance) in [("88", 0x4B), ("80", 0x46)] {
        run.expect_cutoff_out_of_range(value, resonance);
    }
    run.expect_controls(&[0x78, 0x70], &[0x41, 0x3C]);
}

///
/// A Slew moves only on a Tick its Control Change is banged. With `CUT`'s
/// Every at `~*0201`, `CUT` sends on even Ticks only, each time `08` further
/// on, and reaches preset `01` on Tick 14, its eighth send; `RES` keeps its every-Tick
/// cadence and the bassline its steps.
///
#[test]
fn a_control_banged_every_second_tick_moves_only_when_it_sends() {
    let text = edited(PRESETS, &[(CUT_ROW - 1, 3, "1", "2")]);
    let mut run = Run::open(&text);
    run.select("01");
    let resonances = [
        0x25, 0x2A, 0x2F, 0x34, 0x39, 0x3E, 0x43, 0x48, 0x4D, 0x50, 0x50, 0x50, 0x50, 0x50, 0x50,
        0x50, 0x50, 0x50,
    ];
    let cutoffs = [0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x60];
    for (tick, &resonance) in resonances.iter().enumerate() {
        if tick % 2 == 0 {
            run.expect(&[(CUTOFF, cutoffs[tick / 2]), (RESONANCE, resonance)]);
        } else {
            run.expect(&[(RESONANCE, resonance)]);
        }
    }
}

///
/// A Rewind changes no Cell of the Source. Stopped and Rewound partway
/// through a move to preset `01`, the run begins again at Tick `0` with the
/// bassline's first step, and each control carries on from the value its
/// Comment holds rather than from preset `00`, toward the preset the selector
/// still holds.
///
#[test]
fn a_rewind_keeps_each_controls_value_and_the_selector() {
    let mut run = Run::open(PRESETS);
    run.select("01");
    run.expect_controls(
        &[0x28, 0x30, 0x38, 0x40, 0x48],
        &[0x25, 0x2A, 0x2F, 0x34, 0x39],
    );
    let held = run.source.snapshot();

    run.rewind();

    assert_eq!(run.source.snapshot(), held, "the Rewind changed the Source");
    assert_eq!(run.row(SELECTOR_ROW), "  .%0102");
    assert_eq!(run.row(CUT_ROW + 1), "      ||48  6003  01||");
    assert_eq!(run.row(RES_ROW + 1), "      ||39  5003  01||");
    run.expect_controls(
        &[0x50, 0x58, 0x60, 0x60, 0x60, 0x60],
        &[0x3E, 0x43, 0x48, 0x4D, 0x50, 0x50],
    );
}

/// The row the guide pastes the copied `CUT` Region's top-left onto.
const DEC_ROW: usize = 30;

///
/// The shipped file with the guide's decay control: the `CUT` Region, rows
/// 18 to 22, pasted at (0, 30), then its label, controller, Slew `amount`
/// and two targets typed over.
///
fn with_decay() -> String {
    let region: Vec<&str> = PRESETS.lines().skip(CUT_ROW - 1).take(5).collect();
    let mut edits: Vec<(usize, usize, &str, &str)> = region
        .iter()
        .enumerate()
        .map(|(offset, &line)| (DEC_ROW + offset, 0, "", line))
        .collect();
    edits.extend([
        (DEC_ROW, 11, "CUT", "DEC"),
        (DEC_ROW + 1, 7, "A", "B"),
        (DEC_ROW + 1, 10, "08", "10"),
        (DEC_ROW + 3, 14, "2", "1"),
        (DEC_ROW + 4, 14, "6", "7"),
    ]);
    edited(PRESETS, &edits)
}

///
/// The guide draws the decay control as its edits leave it, the rest of the
/// file is as shipped, and the Language Map still diagnoses nothing.
///
#[test]
fn the_decay_control_is_drawn_as_the_guide_draws_it_and_diagnosed_nowhere() {
    let text = with_decay();
    let rows: Vec<&str> = text.lines().collect();
    assert_eq!(rows.len(), DEC_ROW + 5);
    assert_eq!(
        rows[..CUT_ROW + 10],
        PRESETS.lines().collect::<Vec<_>>()[..]
    );
    assert_eq!(rows[DEC_ROW - 1], "");
    for (row, line) in rows.iter().enumerate().skip(DEC_ROW) {
        assert_guide_draws(GUIDE, row, line);
    }
    let run = Run::open(&text);
    assert_eq!(mapped(&run.source), vec![]);
}

///
/// The guide's decay control, driven by the same selector: it starts from
/// the `20` the copy brings, sends preset `00`'s `10` on the first Tick and
/// holds it, then climbs `10` a Tick to preset `01`'s `70` from the Tick
/// after the selector changes. The bassline and the cutoff and resonance
/// Control Changes are Tick for Tick what the shipped file delivers, with the
/// decay's after them.
///
#[test]
fn the_decay_control_follows_the_selector_and_leaves_the_rest_as_shipped() {
    let mut run = Run::open(&with_decay());
    let cutoffs = [
        0x20, 0x20, 0x20, 0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x60, 0x60,
    ];
    let resonances = [
        0x20, 0x20, 0x20, 0x20, 0x25, 0x2A, 0x2F, 0x34, 0x39, 0x3E, 0x43, 0x48, 0x4D, 0x50,
    ];
    let decays = [
        0x10, 0x10, 0x10, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x70, 0x70, 0x70, 0x70,
    ];
    for tick in 0..cutoffs.len() {
        if tick == 4 {
            run.select("01");
        }
        run.expect(&[
            (CUTOFF, cutoffs[tick]),
            (RESONANCE, resonances[tick]),
            (DECAY, decays[tick]),
        ]);
    }
    assert_eq!(run.row(DEC_ROW + 2), "      ||70  7003  01||");
}

/// The shipped Tracker, which the guide moves the controls into.
const TRACKER: &str = include_str!("../../../../examples/tracker.orcvs");

/// The row the guide pastes the copied Preset block and controls onto in the
/// Tracker, and the rows their selector and Control Changes land on.
const TRACKER_PASTE_ROW: usize = 36;
const TRACKER_SELECTOR_ROW: usize = TRACKER_PASTE_ROW + SELECTOR_ROW - 14;
const TRACKER_CUT_ROW: usize = TRACKER_PASTE_ROW + CUT_ROW - 14;
const TRACKER_RES_ROW: usize = TRACKER_PASTE_ROW + RES_ROW - 14;

///
/// The Tracker with the guide's transfer: the Presets' rows 14 to 28, the
/// Preset block and both controls, pasted at (0, 36), and each control's
/// Absolute Read retyped to name the preset in force at (2, 38).
///
fn tracker_with_presets() -> String {
    let region: Vec<&str> = PRESETS.lines().skip(14).take(15).collect();
    let mut edits: Vec<(usize, usize, &str, &str)> = region
        .iter()
        .enumerate()
        .map(|(offset, &line)| (TRACKER_PASTE_ROW + offset, 0, "", line))
        .collect();
    edits.extend([
        (TRACKER_CUT_ROW, 22, "10", "26"),
        (TRACKER_RES_ROW, 22, "10", "26"),
    ]);
    edited(TRACKER, &edits)
}

///
/// The guide draws the Tracker's pasted rows as the transfer leaves them, and
/// the Language Map diagnoses exactly what it diagnoses in the shipped
/// Tracker: the transfer adds no diagnosis.
///
#[test]
fn the_controls_moved_into_the_tracker_are_drawn_and_add_no_diagnosis() {
    let text = tracker_with_presets();
    let rows: Vec<&str> = text.lines().collect();
    assert_eq!(rows[..31], TRACKER.lines().collect::<Vec<_>>()[..]);
    assert_eq!(rows.len(), TRACKER_PASTE_ROW + 15);
    for (row, line) in rows.iter().enumerate().skip(TRACKER_PASTE_ROW) {
        assert_guide_draws(GUIDE, row, line);
    }
    let tracker = Run::open(TRACKER);
    let shipped = mapped(&tracker.source);
    assert_eq!(shipped.len(), 8, "the shipped Tracker's diagnoses");
    assert_eq!(mapped(&Run::open(&text).source), shipped);
}

///
/// In the Tracker the controls send after every voice's Notes on each Tick,
/// and every voice plays what it plays in the shipped Tracker. The selector
/// at (5, 37) turns both controls toward preset `01` as it does in the
/// Presets, and nothing is diagnosed on any Tick.
///
#[test]
fn the_controls_moved_into_the_tracker_slew_its_bassline_and_leave_its_notes() {
    let mut transferred = Run::open(&tracker_with_presets());
    let mut shipped = Run::open(TRACKER);
    let cutoffs = [
        0x20, 0x20, 0x20, 0x20, 0x28, 0x30, 0x38, 0x40, 0x48, 0x50, 0x58, 0x60, 0x60, 0x60, 0x60,
        0x60, 0x60, 0x60,
    ];
    let resonances = [
        0x20, 0x20, 0x20, 0x20, 0x25, 0x2A, 0x2F, 0x34, 0x39, 0x3E, 0x43, 0x48, 0x4D, 0x50, 0x50,
        0x50, 0x50, 0x50,
    ];
    for (tick, (&cutoff, &resonance)) in cutoffs.iter().zip(&resonances).enumerate() {
        if tick == 4 {
            transferred.type_at(SELECTOR_COLUMN + 1, TRACKER_SELECTOR_ROW, "1");
        }
        let tick = tick as u64;
        let notes = shipped.run.run_tick(tick);
        assert_eq!(
            tick_diagnostics(&notes),
            vec![],
            "the shipped Tracker's Tick {tick}"
        );
        let plan = transferred.run.run_tick(tick);
        assert_eq!(tick_diagnostics(&plan), vec![], "Tick {tick}'s diagnostics");
        let mut expected = shipped
            .adapter
            .command_lists()
            .last()
            .cloned()
            .expect("the shipped Tracker delivers every Tick");
        assert!(
            expected
                .iter()
                .any(|command| matches!(command, OutputCommand::NoteOn { .. })),
            "Tick {tick} starts a Note"
        );
        expected.extend([change(CUTOFF, cutoff), change(RESONANCE, resonance)]);
        assert_eq!(
            transferred.adapter.command_lists().last(),
            Some(&expected),
            "Tick {tick}'s delivery"
        );
    }
    assert_eq!(transferred.adapter.command_lists().len(), cutoffs.len());
}
