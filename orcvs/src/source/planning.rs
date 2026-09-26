//! Planning a Tick away from the Source lock, and committing it only against
//! the revision it was planned from.
//!
//! A Tick Plan is a function of one Source revision and the Tick (ADR 0003),
//! so the inputs can be taken out under a short read and interpreted with no
//! lock held. What that gives up is the guarantee that the Source is still at
//! that revision when the plan is committed, so the revision's identity is
//! captured with its contents and checked again under the write lock the
//! commit takes. A plan refused there is dropped whole: its Cell writes, Play
//! Commands and diagnostics describe a revision nobody can observe any more.

use std::sync::Arc;

use super::buffer::SourceBuffer;
use super::language_map::LanguageMap;
use super::tick;
use super::{OPTIMISTIC_TICK_ATTEMPTS, RevisionId, Source, SourceCommander, Tick, TickPlan};
use crate::grid::Grid;

///
/// What a Tick is planned from: one Source revision's Grid, Cells and
/// Language Map, and that revision's identity.
///
/// All four are read from one `&Source`, so under one guard, which is what
/// makes the identity name the revision the contents came from. The Cells and
/// the Language Map are shared rather than copied — the Map with the schedule
/// every revision holding the same scheduling inputs shares, so planning reads
/// and fills that schedule without the Source. An edit landing while this is
/// held copies the Cells before writing, and this keeps the revision it took.
///
pub(super) struct PlanningSnapshot {
    grid: Grid,
    cells: SourceBuffer,
    language_map: Arc<LanguageMap>,
    revision: RevisionId,
}

impl PlanningSnapshot {
    pub(super) fn capture(source: &Source) -> Self {
        Self {
            grid: source.grid(),
            cells: source.shared_cells(),
            language_map: source.shared_language_map(),
            revision: source.revision(),
        }
    }

    /// Interprets this revision at `tick`. Touches no Source.
    pub(super) fn plan(&self, tick: Tick) -> PlannedTick {
        // Each computation's Turn is discarded, as `Source::execute` discards
        // it: a Playback Engine asks nothing about how a plan was reached.
        let (plan, _) = tick::plan(self.grid, self.cells.bytes(), &self.language_map, tick);
        PlannedTick {
            revision: self.revision,
            plan,
        }
    }
}

///
/// A Tick Plan that has not been committed, and the revision it was planned
/// against.
///
/// The plan is reachable only through [`Self::commit`], so nothing can publish
/// a plan the Source has not accepted.
///
pub(super) struct PlannedTick {
    revision: RevisionId,
    plan: TickPlan,
}

impl PlannedTick {
    ///
    /// Commits the plan when `source` is still at the revision it was planned
    /// against, and answers the plan to publish.
    ///
    /// Any write since the snapshot mints a new revision, so a mismatch means
    /// an edit — or another commit — landed while this plan was being made,
    /// and committing would overwrite it with Cells derived without it. The
    /// check and the commit take one `&mut Source`, so no write can land
    /// between them.
    ///
    pub(super) fn commit(self, source: &mut Source) -> Result<TickPlan, StalePlan> {
        if source.revision() != self.revision {
            return Err(StalePlan);
        }
        source.commit_tick(&self.plan);
        Ok(self.plan)
    }
}

///
/// A plan refused because the Source moved past the revision it was planned
/// against.
///
/// It carries nothing: the refused plan's writes, Play Commands and
/// diagnostics are dropped with it, so a caller holding this has nothing to
/// deliver.
///
#[derive(Debug, PartialEq, Eq)]
pub(super) struct StalePlan;

///
/// One Tick being committed to a shared Source, attempt by attempt.
///
/// The first [`OPTIMISTIC_TICK_ATTEMPTS`] attempts are planned from a
/// snapshot with no guard held and can be refused; the attempt after that
/// many refusals is planned and committed under one write guard, which
/// nothing can refuse. So a Tick ends in at most one more attempt than the
/// budget, however fast the Source is edited.
///
pub(super) struct TickCommit<'s> {
    source: &'s SourceCommander,
    tick: Tick,
    refused: usize,
}

impl<'s> TickCommit<'s> {
    pub(super) fn new(source: &'s SourceCommander, tick: Tick) -> Self {
        Self {
            source,
            tick,
            refused: 0,
        }
    }

    ///
    /// The next attempt. An optimistic attempt is planned here, from the
    /// current revision, and holds no guard once this returns; the locked
    /// attempt plans when it commits.
    ///
    fn plan(&mut self) -> Attempt<'_, 's> {
        let kind = if self.refused < OPTIMISTIC_TICK_ATTEMPTS {
            AttemptKind::Optimistic(self.source.planning_snapshot().plan(self.tick))
        } else {
            AttemptKind::Locked
        };
        Attempt {
            tick_commit: self,
            kind,
        }
    }

    /// Commits attempt after attempt until one is accepted.
    pub(super) fn run(mut self) -> TickPlan {
        loop {
            if let Ok(plan) = self.plan().commit() {
                return plan;
            }
        }
    }
}

///
/// One attempt at committing a [`TickCommit`]'s Tick. It borrows the commit
/// mutably, so only the attempt last planned can be committed, and a refusal
/// is counted against the commit that planned it.
///
struct Attempt<'c, 's> {
    tick_commit: &'c mut TickCommit<'s>,
    kind: AttemptKind,
}

enum AttemptKind {
    Optimistic(PlannedTick),
    Locked,
}

impl Attempt<'_, '_> {
    ///
    /// Commits this attempt and answers the plan to publish. Only an
    /// optimistic attempt is refused, when an edit landed after it was
    /// planned; the locked attempt plans and commits under one write guard.
    ///
    fn commit(self) -> Result<TickPlan, StalePlan> {
        let tick_commit = self.tick_commit;
        match self.kind {
            AttemptKind::Optimistic(planned) => {
                let committed = tick_commit.source.commit_planned(planned);
                if committed.is_err() {
                    tick_commit.refused += 1;
                }
                committed
            }
            AttemptKind::Locked => Ok(tick_commit.source.execute_locked(tick_commit.tick)),
        }
    }
}

#[cfg(test)]
impl Attempt<'_, '_> {
    fn is_locked(&self) -> bool {
        matches!(self.kind, AttemptKind::Locked)
    }
}

#[cfg(test)]
mod tests {
    use super::{StalePlan, TickCommit};
    use crate::grid::{CellIndex, Grid};
    use crate::source::{
        MidiChannel, Note, OPTIMISTIC_TICK_ATTEMPTS, PlayCommand, SourceCommander, Tick, Velocity,
    };

    fn cell(grid: Grid, idx: usize) -> CellIndex {
        grid.cell_index(idx).expect("inside the Grid")
    }

    fn type_at(source: &SourceCommander, start: usize, text: &str) {
        let grid = source.grid();
        for (offset, content) in text.chars().enumerate() {
            source
                .set(cell(grid, start + offset), &content.to_string())
                .expect("a Cell the Source accepts");
        }
    }

    fn row(source: &SourceCommander, y: usize) -> String {
        let columns = source.grid().columns();
        source.snapshot()[y * columns..(y + 1) * columns].to_owned()
    }

    fn c4() -> PlayCommand {
        PlayCommand::Raw {
            channel: MidiChannel::try_from(0).unwrap(),
            velocity: Velocity::try_from(0x7F).unwrap(),
            note: Note::try_from(60).unwrap(),
        }
    }

    #[test]
    fn a_captured_snapshot_holds_no_source_lock() {
        let source = SourceCommander::new(Grid::with_shape(8, 2));
        type_at(&source, 0, ".+0102");

        let snapshot = source.planning_snapshot();

        // An editor's write is admitted while the snapshot is alive, and so
        // while it is planned from.
        assert!(source.inner.try_write().is_ok());
        let planned = snapshot.plan(Tick::ZERO);
        assert!(source.inner.try_write().is_ok());
        assert!(source.commit_planned(planned).is_ok());
    }

    #[test]
    fn a_snapshot_names_the_revision_its_contents_came_from() {
        let source = SourceCommander::new(Grid::with_shape(8, 2));
        type_at(&source, 0, ".+0102");
        let before = source.revision();

        let snapshot = source.planning_snapshot();
        type_at(&source, 5, "5");

        assert_eq!(snapshot.revision, before);
        assert_eq!(&snapshot.cells.as_str()[..6], ".+0102");
        assert_ne!(source.revision(), before);
    }

    #[test]
    fn a_snapshot_shares_the_cells_rather_than_copying_them() {
        let source = SourceCommander::new(Grid::with_shape(8, 2));
        type_at(&source, 0, ".+0102");

        let snapshot = source.planning_snapshot();

        let held = source.inner.read().unwrap().shared_cells();
        assert_eq!(snapshot.cells.bytes().as_ptr(), held.bytes().as_ptr());
    }

    #[test]
    fn an_edit_between_plan_and_commit_is_never_overwritten() {
        let source = SourceCommander::new(Grid::with_shape(8, 2));
        type_at(&source, 0, ".+0102");

        let planned = source.planning_snapshot().plan(Tick::ZERO);
        // The edit lands while the plan that answers `03` is being made.
        type_at(&source, 5, "5");
        let edited = source.snapshot();

        assert_eq!(source.commit_planned(planned), Err(StalePlan));
        assert_eq!(source.snapshot(), edited, "a refused plan writes no Cell");

        // The same Tick is planned again, from the revision holding the edit.
        let plan = source.execute(Tick::ZERO);
        assert_eq!(row(&source, 0), ".+0105  ");
        assert_eq!(row(&source, 1), "06      ");
        assert_eq!(plan.writes.len(), 2);
    }

    #[test]
    fn a_refused_plan_carrying_play_commands_delivers_none_of_them() {
        let grid = Grid::with_shape(10, 3);
        let source = SourceCommander::new(grid);
        type_at(&source, 0, ".=0101");
        type_at(&source, 20, "!>007FC4");

        let planned = source.planning_snapshot().plan(Tick::ZERO);
        assert_eq!(planned.plan.play_commands, vec![c4()]);
        assert!(!planned.plan.writes.is_empty());
        // The Raw Play is deleted while the plan that sounds it is made.
        source.unset(cell(grid, 20));
        let edited = source.snapshot();

        // Refusal hands back no plan, so there is no Play Command, write,
        // diagnostic or lock to publish, and no note to take ownership of.
        assert_eq!(source.commit_planned(planned), Err(StalePlan));
        assert_eq!(source.snapshot(), edited);

        let accepted = source.execute(Tick::ZERO);
        assert!(accepted.play_commands.is_empty());
        assert_eq!(row(&source, 1), "**        ");
        assert_eq!(row(&source, 2), " >007FC4  ");
    }

    #[test]
    fn a_tick_planned_from_a_snapshot_is_the_tick_planned_under_the_lock() {
        // Two Sources holding the same revision, one planned from a copied
        // snapshot and one under the write guard, Tick after Tick: every plan
        // and every revision the commits leave agree.
        let fixtures: [(usize, &[(usize, &str)]); 3] = [
            (8, &[(0, ".+0102")]),
            (16, &[(0, "~+0110"), (8, "~.0110")]),
            (10, &[(0, ".=0101"), (20, "!>007FC4")]),
        ];
        for (columns, texts) in fixtures {
            let grid = Grid::with_shape(columns, 3);
            let planned = SourceCommander::new(grid);
            let locked = SourceCommander::new(grid);
            for (start, text) in texts {
                type_at(&planned, *start, text);
                type_at(&locked, *start, text);
            }

            let mut tick = Tick::ZERO;
            for _ in 0..4 {
                let from_snapshot = planned
                    .commit_planned(planned.planning_snapshot().plan(tick))
                    .expect("no edit lands between plan and commit");
                assert_eq!(
                    from_snapshot,
                    locked.execute_locked(tick),
                    "{texts:?} at {tick:?}"
                );
                assert_eq!(
                    planned.snapshot(),
                    locked.snapshot(),
                    "{texts:?} at {tick:?}"
                );
                tick = tick.next();
            }
        }
    }

    #[test]
    fn a_tick_nothing_edits_commits_on_its_first_optimistic_attempt() {
        let source = SourceCommander::new(Grid::with_shape(8, 2));
        type_at(&source, 0, ".+0102");
        let mut commit = TickCommit::new(&source, Tick::ZERO);

        let first = commit.plan();
        assert!(!first.is_locked());
        let plan = first.commit().expect("no edit landed while it was planned");

        assert_eq!(row(&source, 1), "03      ");
        assert_eq!(plan.writes.len(), 2);
    }

    #[test]
    fn a_tick_conflicting_on_every_optimistic_attempt_commits_once_under_the_lock() {
        // An edit lands inside each attempt's window, between the plan and
        // its commit: each optimistic attempt is refused, and the attempt
        // after the last refusal is the locked one, which commits from the
        // edit that landed before it.
        let grid = Grid::with_shape(8, 2);
        let source = SourceCommander::new(grid);
        type_at(&source, 0, ".+0100");
        let mut commit = TickCommit::new(&source, Tick::ZERO);

        for attempt in 1..=OPTIMISTIC_TICK_ATTEMPTS {
            let planned = commit.plan();
            assert!(!planned.is_locked(), "attempt {attempt} took the lock");
            assert!(
                source.inner.try_write().is_ok(),
                "attempt {attempt} holds a guard between its plan and its commit"
            );
            type_at(&source, 4, &format!("{attempt:02X}"));
            assert_eq!(planned.commit(), Err(StalePlan));
            assert_eq!(row(&source, 1), "        ", "attempt {attempt} wrote");
        }

        let locked = commit.plan();
        assert!(locked.is_locked());
        type_at(&source, 4, "0A");
        let plan = locked
            .commit()
            .expect("the locked attempt cannot be refused");

        assert_eq!(row(&source, 0), ".+010A  ");
        assert_eq!(row(&source, 1), "0B      ");
        assert_eq!(plan.writes.len(), 2);
    }

    #[test]
    fn consecutive_accepted_ticks_apply_in_order_across_refused_plans() {
        // `~+0110` counts the Ticks that committed: each reads the answer the
        // last accepted Tick left south of it. `~.0110` answers the absolute
        // Tick it was planned at. A refused plan that was committed would
        // count twice; a retry that consumed a Tick would skip a step.
        let grid = Grid::with_shape(16, 2);
        let source = SourceCommander::new(grid);
        type_at(&source, 0, "~+0110");
        type_at(&source, 8, "~.0110");

        let mut tick = Tick::ZERO;
        for step in 1..=6u8 {
            if step % 2 == 0 {
                let planned = source.planning_snapshot().plan(tick);
                // An edit in a Cell no Tick reads or writes.
                type_at(&source, 15, if step % 4 == 0 { "x" } else { "y" });
                assert_eq!(source.commit_planned(planned), Err(StalePlan));
            }
            source.execute(tick);
            let counted = format!("{step:02X}");
            let at = format!("{:02X}", step - 1);
            assert_eq!(&row(&source, 1)[..2], counted, "Ticks committed");
            assert_eq!(&row(&source, 1)[8..10], at, "the Tick planned at");
            tick = tick.next();
        }
    }

    #[test]
    fn ticks_under_continuous_editing_terminate_and_count_once() {
        // Timing-dependent in how often an attempt is refused, and in nothing
        // it asserts: every Tick returns and every one counts once. The edits
        // land in a Cell no Tick writes, so this cannot show an edit
        // surviving a stale plan; the deterministic tests above do.
        let grid = Grid::with_shape(16, 2);
        let source = SourceCommander::new(grid);
        type_at(&source, 0, "~+01FF");
        const TICKS: u8 = 200;

        let editor = source.clone();
        let editing = std::thread::spawn(move || {
            let grid = editor.grid();
            for round in 0..2_000usize {
                let content = if round % 2 == 0 { "a" } else { "b" };
                editor.set(cell(grid, 12), content).unwrap();
            }
            editor.set(cell(grid, 12), "z").unwrap();
        });

        let mut tick = Tick::ZERO;
        for _ in 0..TICKS {
            source.execute(tick);
            tick = tick.next();
        }
        editing.join().unwrap();

        assert_eq!(&row(&source, 1)[..2], format!("{TICKS:02X}"));
        assert_eq!(&row(&source, 0)[12..13], "z");
    }

    ///
    /// A timing profile, run on demand:
    ///
    /// ```sh
    /// cargo nextest run --package orcvs --release --no-capture --run-ignored only \
    ///     -E 'test(/source::planning::tests::profile/)'
    /// ```
    ///
    /// One thread executes a Tick every `PERIOD`, as a Playback run does,
    /// while the test thread reads revisions, then edits, against it. Each is
    /// measured twice in the same process: once with every Tick planned and
    /// committed under one write guard, which is what the fallback attempt
    /// does, and once through `execute`. The edits are measured paced, one
    /// every half millisecond, far faster than anyone types, and back to back,
    /// where every optimistic attempt is refused and every Tick pays its
    /// retries and then the locked attempt.
    ///
    /// Ignored because it asserts nothing: it prints nearest-rank percentiles
    /// of each wait and of the Tick's own latency, which is where retries cost.
    ///
    mod profile {
        use crate::grid::Grid;
        use crate::source::{SourceCommander, Tick};
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::time::{Duration, Instant};

        const SHAPE: (usize, usize) = (128, 128);
        const READS: usize = 20_000;
        const EDITS: usize = 2_000;
        /// Reads are paced so they sample the Tick period evenly: a read
        /// kept waiting would otherwise be one sample however long it waited.
        const READ_PACE: Duration = Duration::from_micros(50);
        const EDIT_PACE: Duration = Duration::from_micros(500);
        /// About twice what a Tick over `SHAPE` costs, so a Tick is in
        /// progress for roughly half of every period.
        const PERIOD: Duration = Duration::from_millis(2);

        #[derive(Clone, Copy)]
        enum Mode {
            Locked,
            Planned,
        }

        /// Independent adders on even rows, each answering into the blank
        /// row beneath it: every root delivers and none contests a Cell.
        fn playing(grid: Grid) -> SourceCommander {
            let source = SourceCommander::new(grid);
            let mut writes = Vec::new();
            for y in (0..grid.rows()).step_by(2) {
                for x in (0..grid.columns().saturating_sub(7)).step_by(8) {
                    for (offset, byte) in ".+0102".bytes().enumerate() {
                        let position = grid.position(x + offset, y).expect("inside the Grid");
                        writes.push(crate::source::CellWrite {
                            cell: grid.index(position),
                            content: crate::source::CellContent::new(byte).unwrap(),
                        });
                    }
                }
            }
            source.write_cells(&writes);
            source
        }

        fn percentiles(label: &str, mut samples: Vec<Duration>) {
            samples.sort_unstable();
            let at = |q: f64| {
                let rank = ((samples.len() as f64) * q).ceil() as usize;
                samples[rank.clamp(1, samples.len()) - 1]
            };
            println!(
                "{label:<38} n={:>6} p50={:>10.1?} p99={:>10.1?} p99.9={:>10.1?} max={:>10.1?}",
                samples.len(),
                at(0.5),
                at(0.99),
                at(0.999),
                at(1.0),
            );
        }

        /// Runs `measure` while another thread executes Ticks in `mode`, and
        /// answers what `measure` recorded and each Tick's latency.
        fn under_ticks(
            source: &SourceCommander,
            mode: Mode,
            measure: impl FnOnce() -> Vec<Duration>,
        ) -> (Vec<Duration>, Vec<Duration>) {
            let running = Arc::new(AtomicBool::new(true));
            let ticker = {
                let source = source.clone();
                let running = Arc::clone(&running);
                std::thread::spawn(move || {
                    let mut tick = Tick::ZERO;
                    let mut latencies = Vec::new();
                    let mut deadline = Instant::now();
                    while running.load(Ordering::Relaxed) {
                        spin_for(deadline.saturating_duration_since(Instant::now()));
                        deadline += PERIOD;
                        let began = Instant::now();
                        match mode {
                            Mode::Locked => source.execute_locked(tick),
                            Mode::Planned => source.execute(tick),
                        };
                        latencies.push(began.elapsed());
                        tick = tick.next();
                    }
                    latencies
                })
            };
            // Let the ticker reach its steady state before anything is timed.
            std::thread::sleep(Duration::from_millis(50));
            let measured = measure();
            running.store(false, Ordering::Relaxed);
            (measured, ticker.join().expect("the ticker"))
        }

        /// Spun rather than slept: an OS timer may wake late by more than
        /// the whole interval.
        fn spin_for(interval: Duration) {
            let until = Instant::now() + interval;
            while Instant::now() < until {
                std::hint::spin_loop();
            }
        }

        fn reads(source: &SourceCommander) -> Vec<Duration> {
            (0..READS)
                .map(|_| {
                    spin_for(READ_PACE);
                    let began = Instant::now();
                    std::hint::black_box(source.read_revision());
                    began.elapsed()
                })
                .collect()
        }

        fn edits(source: &SourceCommander, pace: Option<Duration>) -> Vec<Duration> {
            let cell = source.grid().cell_index(4).expect("an operand digit");
            (0..EDITS)
                .map(|edit| {
                    if let Some(pace) = pace {
                        spin_for(pace);
                    }
                    let content = if edit % 2 == 0 { "3" } else { "2" };
                    let began = Instant::now();
                    source.set(cell, content).expect("an operand digit");
                    began.elapsed()
                })
                .collect()
        }

        #[test]
        #[ignore = "a timing profile that asserts nothing; run on demand"]
        fn profile_revision_reads_and_edits_under_running_ticks() {
            let grid = Grid::with_shape(SHAPE.0, SHAPE.1);
            for (mode, name) in [(Mode::Locked, "locked"), (Mode::Planned, "planned")] {
                let source = playing(grid);
                source.execute(Tick::ZERO);

                let (waits, ticks) = under_ticks(&source, mode, || reads(&source));
                percentiles(&format!("{name}: revision read"), waits);
                percentiles(&format!("{name}: tick while reading"), ticks);

                let (waits, ticks) = under_ticks(&source, mode, || edits(&source, Some(EDIT_PACE)));
                percentiles(&format!("{name}: paced edit"), waits);
                percentiles(&format!("{name}: tick under paced edits"), ticks);

                let (waits, ticks) = under_ticks(&source, mode, || edits(&source, None));
                percentiles(&format!("{name}: back-to-back edit"), waits);
                percentiles(&format!("{name}: tick under back-to-back edits"), ticks);
            }
        }
    }
}
