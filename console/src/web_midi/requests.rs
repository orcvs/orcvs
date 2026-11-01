//! The page's requests for MIDI access and the answer that settles them.
//!
//! A browser can leave a request unanswered for good: Firefox does when its
//! permission prompt is dismissed. So the performer's Scan sends a fresh
//! request, and the page can hold several. Which answers count:
//!
//! - A grant from any request settles access as granted, even after a refusal,
//!   because the performer allowed a prompt the browser showed.
//! - A refusal settles access as unavailable only when it answers the latest
//!   request. An older request's refusal is stale: it says nothing about the
//!   prompt the latest one is showing, and an older request that never settles
//!   cannot keep the latest one's refusal from counting.
//!
//! The browser adapter keeps one of these and stores each request's answer
//! here; the tests drive the same rules without a browser.

use super::AccessStatus;

/// Which of the page's requests an answer belongs to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RequestId(u32);

pub(super) struct AccessRequests<G> {
    latest: RequestId,
    settled: Settled<G>,
}

enum Settled<G> {
    Pending,
    Granted(G),
    Unavailable,
}

impl<G> AccessRequests<G> {
    /// Access with the page's first request outstanding, and that request's id.
    pub(super) fn first() -> (Self, RequestId) {
        let first = RequestId(0);
        let requests = Self {
            latest: first,
            settled: Settled::Pending,
        };
        (requests, first)
    }

    pub(super) fn status(&self) -> AccessStatus {
        match self.settled {
            Settled::Pending => AccessStatus::Pending,
            Settled::Granted(_) => AccessStatus::Granted,
            Settled::Unavailable => AccessStatus::Unavailable,
        }
    }

    pub(super) fn granted(&self) -> Option<&G> {
        match &self.settled {
            Settled::Granted(granted) => Some(granted),
            Settled::Pending | Settled::Unavailable => None,
        }
    }

    ///
    /// The id of a fresh request while access is pending, which becomes the
    /// latest, or `None` once access is settled and there is nothing to ask.
    ///
    pub(super) fn ask_again(&mut self) -> Option<RequestId> {
        if !matches!(self.settled, Settled::Pending) {
            return None;
        }
        self.latest = RequestId(self.latest.0.wrapping_add(1));
        Some(self.latest)
    }

    ///
    /// Stores the browser's answer to request `id`: `Some` grant or `None`
    /// for a refusal. Answers whether it changed the access status.
    ///
    pub(super) fn answer(&mut self, id: RequestId, answer: Option<G>) -> bool {
        match (answer, &self.settled) {
            (_, Settled::Granted(_)) => false,
            (Some(granted), Settled::Pending | Settled::Unavailable) => {
                self.settled = Settled::Granted(granted);
                true
            }
            (None, Settled::Pending) if id == self.latest => {
                self.settled = Settled::Unavailable;
                true
            }
            (None, Settled::Pending | Settled::Unavailable) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AccessRequests, AccessStatus};

    #[test]
    fn the_first_answer_settles_the_only_request() {
        let (mut granted, first) = AccessRequests::first();
        assert_eq!(granted.status(), AccessStatus::Pending);
        assert!(granted.answer(first, Some("access")));
        assert_eq!(granted.status(), AccessStatus::Granted);
        assert_eq!(granted.granted(), Some(&"access"));

        let (mut refused, first) = AccessRequests::<&str>::first();
        assert!(refused.answer(first, None));
        assert_eq!(refused.status(), AccessStatus::Unavailable);
        assert_eq!(refused.granted(), None);
    }

    ///
    /// The dismissed Firefox prompt: the first request never settles, and the
    /// fresh one a Scan sends is answered either way.
    ///
    #[test]
    fn a_request_that_never_settles_does_not_hold_back_the_latest_answer() {
        let (mut refused, _dismissed) = AccessRequests::<&str>::first();
        let fresh = refused.ask_again().expect("access is pending");
        assert!(refused.answer(fresh, None));
        assert_eq!(refused.status(), AccessStatus::Unavailable);

        let (mut granted, _dismissed) = AccessRequests::first();
        let fresh = granted.ask_again().expect("access is pending");
        assert!(granted.answer(fresh, Some("access")));
        assert_eq!(granted.status(), AccessStatus::Granted);
    }

    #[test]
    fn a_stale_refusal_leaves_the_latest_request_pending() {
        let (mut requests, stale) = AccessRequests::<&str>::first();
        let fresh = requests.ask_again().expect("access is pending");
        assert!(!requests.answer(stale, None));
        assert_eq!(requests.status(), AccessStatus::Pending);
        assert!(requests.answer(fresh, Some("access")));
        assert_eq!(requests.status(), AccessStatus::Granted);
    }

    #[test]
    fn a_grant_from_any_request_wins_whenever_it_arrives() {
        let (mut requests, older) = AccessRequests::first();
        let latest = requests.ask_again().expect("access is pending");
        assert!(requests.answer(latest, None));
        assert!(requests.answer(older, Some("access")));
        assert_eq!(requests.status(), AccessStatus::Granted);

        assert!(!requests.answer(latest, None));
        assert!(!requests.answer(older, Some("another")));
        assert_eq!(requests.granted(), Some(&"access"));
    }

    #[test]
    fn a_settled_answer_is_not_asked_again() {
        let (mut granted, first) = AccessRequests::first();
        granted.answer(first, Some("access"));
        assert_eq!(granted.ask_again(), None);

        let (mut refused, first) = AccessRequests::<&str>::first();
        refused.answer(first, None);
        assert_eq!(refused.ask_again(), None);
    }
}
