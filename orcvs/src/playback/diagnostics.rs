use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use std::time::Duration;

use super::{OutputAdapterError, PlaybackDiagnostic, lock_recover};

///
/// The most diagnostics a Playback Engine retains between two drains.
///
/// A console drains every frame, so an ordinary run never comes near it. The
/// bound is for the embedder that stops draining — a hidden window, a test
/// harness, a host that never asked — whose engine would otherwise keep one
/// entry for every declined Tick for as long as the run lasts.
///
pub const MAX_RETAINED_DIAGNOSTICS: usize = 32;

///
/// The most bytes of message a retained diagnostic keeps.
///
/// An output adapter's error text comes from whatever backend it wraps, so its
/// length is not this engine's to choose. A longer message keeps its first
/// bytes up to a character boundary and ends in `…`, all within this bound.
///
pub const MAX_DIAGNOSTIC_MESSAGE_BYTES: usize = 256;

const TRUNCATION_MARK: char = '…';

///
/// How many diagnostics of one class a Playback Engine recorded and did not
/// retain.
///
/// The count saturates rather than wrapping: once it reaches `u32::MAX` it
/// stays there, and [`Self::is_saturated`] says the value is a lower bound
/// rather than an exact count.
///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OmissionCount(u32);

impl OmissionCount {
    /// The number of omitted diagnostics, exact unless saturated.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Whether the count stopped at `u32::MAX`, so at least that many were
    /// omitted.
    pub const fn is_saturated(self) -> bool {
        self.0 == u32::MAX
    }

    fn increment(&mut self) {
        self.0 = self.0.saturating_add(1);
    }
}

///
/// The diagnostics a Playback Engine recorded since the last drain and did
/// not retain, counted per class.
///
/// A drain that omitted nothing carries no summary, so a summary always has
/// at least one non-zero count.
///
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OmittedDiagnostics {
    pub overruns: OmissionCount,
    pub start_failures: OmissionCount,
    pub retune_failures: OmissionCount,
    pub output_failures: OmissionCount,
    pub clock_failures: OmissionCount,
}

impl OmittedDiagnostics {
    fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    fn count(&mut self, class: Class) {
        self.counter(class).increment();
    }

    fn counter(&mut self, class: Class) -> &mut OmissionCount {
        match class {
            Class::Overrun => &mut self.overruns,
            Class::StartFailure => &mut self.start_failures,
            Class::RetuneFailure => &mut self.retune_failures,
            Class::OutputFailure => &mut self.output_failures,
            Class::ClockFailure => &mut self.clock_failures,
        }
    }
}

///
/// The class of a retained diagnostic, ordered from the first to be evicted to
/// the last.
///
/// An Overrun carries no user-facing message and only says a Tick was late.
/// A start or retune failure is also handed back to the caller that asked, as
/// an `Err`, so the diagnostic is the second copy of it. An output failure is
/// the only word a user gets that their device refused a Tick. A clock failure
/// says the run has ended by itself, and `Orcvs` withdraws its request to play
/// only on finding one, so it is the last to go.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    Overrun,
    StartFailure,
    RetuneFailure,
    OutputFailure,
    ClockFailure,
}

impl Class {
    ///
    /// Start and retune failures are one tier: both are refusals the caller
    /// already received, and neither is worth more than the other.
    ///
    fn tier(self) -> u8 {
        match self {
            Class::Overrun => 0,
            Class::StartFailure | Class::RetuneFailure => 1,
            Class::OutputFailure => 2,
            Class::ClockFailure => 3,
        }
    }
}

///
/// One diagnostic a Playback Engine records, as distinct from the summary a
/// drain appends.
///
/// Every variant is one of [`PlaybackDiagnostic`]'s except `Omitted`, which
/// only [`DiagnosticLog::drain`] builds, so the log never has a summary to
/// retain or merge.
///
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Report {
    Overrun {
        scheduled_at: Duration,
        observed_at: Duration,
    },
    OutputFailure(OutputAdapterError),
    ClockFailure {
        message: String,
    },
    StartFailure {
        message: String,
    },
    RetuneFailure {
        message: String,
    },
}

impl Report {
    fn class(&self) -> Class {
        match self {
            Report::Overrun { .. } => Class::Overrun,
            Report::StartFailure { .. } => Class::StartFailure,
            Report::RetuneFailure { .. } => Class::RetuneFailure,
            Report::OutputFailure(_) => Class::OutputFailure,
            Report::ClockFailure { .. } => Class::ClockFailure,
        }
    }

    fn message_mut(&mut self) -> Option<&mut String> {
        match self {
            Report::Overrun { .. } => None,
            Report::OutputFailure(error) => Some(&mut error.message),
            Report::ClockFailure { message }
            | Report::StartFailure { message }
            | Report::RetuneFailure { message } => Some(message),
        }
    }
}

impl From<Report> for PlaybackDiagnostic {
    fn from(report: Report) -> Self {
        match report {
            Report::Overrun {
                scheduled_at,
                observed_at,
            } => PlaybackDiagnostic::Overrun {
                scheduled_at,
                observed_at,
            },
            Report::OutputFailure(error) => PlaybackDiagnostic::OutputFailure(error),
            Report::ClockFailure { message } => PlaybackDiagnostic::ClockFailure { message },
            Report::StartFailure { message } => PlaybackDiagnostic::StartFailure { message },
            Report::RetuneFailure { message } => PlaybackDiagnostic::RetuneFailure { message },
        }
    }
}

struct Retained {
    /// In recording order, and never longer than
    /// [`MAX_RETAINED_DIAGNOSTICS`], so its allocation never grows past the
    /// one made up front.
    entries: VecDeque<Report>,
    omitted: OmittedDiagnostics,
}

///
/// The ordered, bounded record of what a Playback Engine has to report.
///
/// Every writer — the engine's task and each handle reporting its own refusal
/// — and every drain share one log, so a drain answers in the order things
/// were recorded whichever part of the engine recorded them, and takes each
/// entry away so no two drains see it. The lock is held only to append one
/// entry or to take them all, never across a Tick or a delivery, so a drain
/// waits on at most one append and a Tick on at most one drain.
///
/// Retention is bounded in entries and in bytes:
///
/// - At most [`MAX_RETAINED_DIAGNOSTICS`] entries, each with a message of at
///   most [`MAX_DIAGNOSTIC_MESSAGE_BYTES`], in an allocation made once. What
///   the log holds between drains is therefore at most that many entries of
///   `PlaybackDiagnostic` plus that many bounded messages, whatever the
///   engine is asked to record.
/// - A diagnostic recorded into a full log evicts the oldest entry of the
///   lowest class present, provided that class is no higher than its own; a
///   diagnostic of a lower class than everything retained is itself omitted.
///   Classes are ordered Overrun, then start and retune failures, then output
///   failures, then clock failures, so a flood of one class never displaces a
///   more serious one, and the newest diagnostic of the highest class
///   recorded since the last drain is always retained.
/// - Every eviction and every omission is counted against its class in an
///   [`OmittedDiagnostics`] whose counts saturate. A drain that omitted
///   anything ends with one [`PlaybackDiagnostic::Omitted`] carrying them, so
///   the number of Overruns since the last drain is the Overruns retained plus
///   the Overruns omitted, exact unless the count saturated.
///
#[derive(Clone)]
pub(super) struct DiagnosticLog(Arc<Mutex<Retained>>);

impl DiagnosticLog {
    pub(super) fn new() -> Self {
        Self(Arc::new(Mutex::new(Retained {
            entries: VecDeque::with_capacity(MAX_RETAINED_DIAGNOSTICS),
            omitted: OmittedDiagnostics::default(),
        })))
    }

    /// Records one diagnostic under the retention policy on [`DiagnosticLog`].
    pub(super) fn report(&self, mut report: Report) {
        if let Some(message) = report.message_mut() {
            bound_message(message);
        }
        let class = report.class();
        let mut retained = lock_recover(&self.0);
        if retained.entries.len() == MAX_RETAINED_DIAGNOSTICS {
            let evicted = retained
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(index, retained)| (retained.class().tier(), *index))
                .filter(|(_, retained)| retained.class().tier() <= class.tier())
                .map(|(index, _)| index);
            match evicted.and_then(|index| retained.entries.remove(index)) {
                Some(evicted) => retained.omitted.count(evicted.class()),
                None => {
                    retained.omitted.count(class);
                    return;
                }
            }
        }
        retained.entries.push_back(report);
    }

    ///
    /// Takes every retained diagnostic in recording order, followed by one
    /// [`PlaybackDiagnostic::Omitted`] when anything since the last drain was
    /// not retained, and resets the log to empty.
    ///
    pub(super) fn drain(&self) -> Vec<PlaybackDiagnostic> {
        let mut retained = lock_recover(&self.0);
        let mut drained: Vec<_> = retained
            .entries
            .drain(..)
            .map(PlaybackDiagnostic::from)
            .collect();
        let omitted = std::mem::take(&mut retained.omitted);
        if !omitted.is_empty() {
            drained.push(PlaybackDiagnostic::Omitted(omitted));
        }
        drained
    }
}

///
/// Cuts `message` to at most [`MAX_DIAGNOSTIC_MESSAGE_BYTES`] on a character
/// boundary, marking the cut, and releases any capacity past its length so the
/// bound holds for the allocation and not just the text.
///
fn bound_message(message: &mut String) {
    if message.len() > MAX_DIAGNOSTIC_MESSAGE_BYTES {
        let mut end = MAX_DIAGNOSTIC_MESSAGE_BYTES - TRUNCATION_MARK.len_utf8();
        while !message.is_char_boundary(end) {
            end -= 1;
        }
        message.truncate(end);
        message.push(TRUNCATION_MARK);
    }
    message.shrink_to_fit();
}

#[cfg(test)]
impl DiagnosticLog {
    ///
    /// The heap the log holds: its entry allocation and every retained
    /// message's allocation.
    ///
    pub(super) fn retained_bytes(&self) -> usize {
        let retained = lock_recover(&self.0);
        let messages: usize = retained
            .entries
            .iter()
            .map(|report| match report {
                Report::Overrun { .. } => 0,
                Report::OutputFailure(error) => error.message.capacity(),
                Report::ClockFailure { message }
                | Report::StartFailure { message }
                | Report::RetuneFailure { message } => message.capacity(),
            })
            .sum();
        retained.entries.capacity() * std::mem::size_of::<Report>() + messages
    }

    ///
    /// The most [`Self::retained_bytes`] may ever answer: a full entry
    /// allocation, each entry holding a message of the largest size kept.
    ///
    pub(super) fn retention_bound() -> usize {
        MAX_RETAINED_DIAGNOSTICS * (std::mem::size_of::<Report>() + MAX_DIAGNOSTIC_MESSAGE_BYTES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overrun(index: u64) -> Report {
        Report::Overrun {
            scheduled_at: Duration::from_secs(index),
            observed_at: Duration::from_secs(index + 2),
        }
    }

    fn clock_failure(message: &str) -> Report {
        Report::ClockFailure {
            message: message.to_string(),
        }
    }

    fn omitted(drained: &[PlaybackDiagnostic]) -> OmittedDiagnostics {
        match drained.last() {
            Some(PlaybackDiagnostic::Omitted(omitted)) => *omitted,
            _ => OmittedDiagnostics::default(),
        }
    }

    #[test]
    fn a_log_under_its_bound_drains_everything_in_order_and_no_summary() {
        let log = DiagnosticLog::new();
        log.report(overrun(1));
        log.report(clock_failure("ended"));
        log.report(overrun(2));

        assert_eq!(
            log.drain(),
            vec![
                overrun(1).into(),
                clock_failure("ended").into(),
                overrun(2).into()
            ]
        );
        assert!(log.drain().is_empty(), "a drain takes what it answers");
    }

    #[test]
    fn a_full_log_omits_an_overrun_and_counts_it() {
        let log = DiagnosticLog::new();
        for index in 0..MAX_RETAINED_DIAGNOSTICS as u64 + 5 {
            log.report(overrun(index));
        }

        let drained = log.drain();
        assert_eq!(drained.len(), MAX_RETAINED_DIAGNOSTICS + 1);
        assert_eq!(
            drained[0],
            overrun(5).into(),
            "Overruns evict the oldest of their own class"
        );
        assert_eq!(omitted(&drained).overruns.get(), 5);
        assert!(
            log.drain().is_empty(),
            "the counts are taken with the entries"
        );
    }

    #[test]
    fn a_higher_class_evicts_the_oldest_of_the_lowest_class_present() {
        let log = DiagnosticLog::new();
        log.report(clock_failure("first"));
        for index in 0..MAX_RETAINED_DIAGNOSTICS as u64 - 1 {
            log.report(Report::StartFailure {
                message: format!("start {index}"),
            });
        }
        log.report(overrun(0));
        log.report(Report::OutputFailure(OutputAdapterError::new(
            "device lost",
        )));

        let drained = log.drain();
        assert_eq!(drained[0], clock_failure("first").into());
        assert_eq!(
            drained[1],
            PlaybackDiagnostic::StartFailure {
                message: "start 1".to_string(),
            },
            "the output failure evicted the oldest start failure"
        );
        assert_eq!(
            drained[MAX_RETAINED_DIAGNOSTICS - 1],
            PlaybackDiagnostic::OutputFailure(OutputAdapterError::new("device lost"))
        );
        let omitted = omitted(&drained);
        assert_eq!(
            omitted.overruns.get(),
            1,
            "a lower class than everything held is omitted"
        );
        assert_eq!(omitted.start_failures.get(), 1);
    }

    #[test]
    fn a_log_full_of_clock_failures_keeps_the_newest() {
        let log = DiagnosticLog::new();
        for index in 0..MAX_RETAINED_DIAGNOSTICS + 3 {
            log.report(clock_failure(&format!("failure {index}")));
        }

        let drained = log.drain();
        assert_eq!(
            drained[MAX_RETAINED_DIAGNOSTICS - 1],
            clock_failure(&format!("failure {}", MAX_RETAINED_DIAGNOSTICS + 2)).into()
        );
        assert_eq!(omitted(&drained).clock_failures.get(), 3);
    }

    #[test]
    fn an_omission_count_saturates_and_says_so() {
        let mut count = OmissionCount(u32::MAX - 1);
        assert!(!count.is_saturated());

        count.increment();
        count.increment();

        assert_eq!(count.get(), u32::MAX);
        assert!(count.is_saturated(), "a saturated count is a lower bound");
    }

    #[test]
    fn a_long_message_is_cut_on_a_character_boundary_within_the_bound() {
        let log = DiagnosticLog::new();
        // Three-byte characters, so the bound falls inside one of them.
        let message = "é€".repeat(MAX_DIAGNOSTIC_MESSAGE_BYTES);
        log.report(Report::OutputFailure(OutputAdapterError::new(
            message.clone(),
        )));

        assert!(log.retained_bytes() <= DiagnosticLog::retention_bound());
        let drained = log.drain();
        let [PlaybackDiagnostic::OutputFailure(error)] = drained.as_slice() else {
            panic!("one retained output failure: {drained:?}");
        };
        assert!(error.message.len() <= MAX_DIAGNOSTIC_MESSAGE_BYTES);
        assert!(error.message.ends_with(TRUNCATION_MARK));
        let kept = error.message.trim_end_matches(TRUNCATION_MARK);
        assert!(
            message.starts_with(kept),
            "the cut keeps the message's start"
        );
    }

    #[test]
    fn a_short_message_is_kept_whole() {
        let log = DiagnosticLog::new();
        log.report(clock_failure("ended"));

        assert_eq!(log.drain(), vec![clock_failure("ended").into()]);
    }
}
