//! Platform-independent state and logic. No OS calls live here.

use std::time::{Duration, Instant};

/// Whether the flask is full (keeping the machine awake) or empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum State {
    #[default]
    Empty,
    Full,
}

impl State {
    pub fn toggled(self) -> Self {
        match self {
            State::Empty => State::Full,
            State::Full => State::Empty,
        }
    }
}

/// Turns a stream of single clicks into double-clicks. macOS status items
/// only report single clicks, so we time them ourselves.
#[derive(Debug)]
pub struct DoubleClickDetector {
    window: Duration,
    last: Option<Instant>,
}

impl DoubleClickDetector {
    pub fn new(window: Duration) -> Self {
        Self { window, last: None }
    }

    /// Record a click at `now`. Returns true if it completes a double-click.
    /// A third quick click starts a new pair rather than toggling again.
    pub fn click(&mut self, now: Instant) -> bool {
        match self.last.take() {
            Some(prev) if now.saturating_duration_since(prev) <= self.window => true,
            _ => {
                self.last = Some(now);
                false
            }
        }
    }
}

/// Schedules activity nudges at a fixed rate while running. Pure timing
/// logic; the app decides when it runs and what a nudge does.
#[derive(Debug)]
pub struct Nudger {
    interval: Duration,
    next: Option<Instant>,
}

impl Nudger {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            next: None,
        }
    }

    /// Start (or keep) running. The first nudge is due immediately, so the
    /// idle timer resets as soon as the mode turns on.
    pub fn start(&mut self, now: Instant) {
        self.next.get_or_insert(now);
    }

    pub fn stop(&mut self) {
        self.next = None;
    }

    pub fn is_running(&self) -> bool {
        self.next.is_some()
    }

    /// When the event loop should next wake, if running.
    pub fn deadline(&self) -> Option<Instant> {
        self.next
    }

    /// True if a nudge is due at `now`; schedules the following one.
    /// Never reports more than one nudge per call, however late it is.
    pub fn poll(&mut self, now: Instant) -> bool {
        match self.next {
            Some(due) if now >= due => {
                self.next = Some(now + self.interval);
                true
            }
            _ => false,
        }
    }
}

/// A nudge is only needed once the user has been idle for `threshold`.
/// While they're actively using the machine, posting input is pointless and
/// could interfere with what they're doing (e.g. mid-drag).
pub fn needs_nudge(idle: Duration, threshold: Duration) -> bool {
    idle >= threshold
}

/// Implemented per OS in `everflask-platform`; mocked in tests.
pub trait KeepAwake {
    type Error: std::error::Error;
    fn enable(&mut self) -> Result<(), Self::Error>;
    fn disable(&mut self) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_flips_state() {
        assert_eq!(State::Empty.toggled(), State::Full);
        assert_eq!(State::Full.toggled(), State::Empty);
    }

    const WINDOW: Duration = Duration::from_millis(400);

    #[test]
    fn two_quick_clicks_is_double() {
        let mut d = DoubleClickDetector::new(WINDOW);
        let t = Instant::now();
        assert!(!d.click(t));
        assert!(d.click(t + Duration::from_millis(200)));
    }

    #[test]
    fn slow_clicks_are_not_double() {
        let mut d = DoubleClickDetector::new(WINDOW);
        let t = Instant::now();
        assert!(!d.click(t));
        assert!(!d.click(t + Duration::from_millis(600)));
        // The slow second click becomes the start of a new pair.
        assert!(d.click(t + Duration::from_millis(700)));
    }

    const EVERY: Duration = Duration::from_secs(60);

    #[test]
    fn nudger_is_idle_until_started() {
        let mut n = Nudger::new(EVERY);
        assert!(!n.is_running());
        assert_eq!(n.deadline(), None);
        assert!(!n.poll(Instant::now()));
    }

    #[test]
    fn nudger_fires_immediately_then_at_interval() {
        let mut n = Nudger::new(EVERY);
        let t = Instant::now();
        n.start(t);
        assert!(n.poll(t));
        assert!(!n.poll(t + Duration::from_secs(59)));
        assert!(n.poll(t + EVERY));
        assert_eq!(n.deadline(), Some(t + EVERY + EVERY));
    }

    #[test]
    fn nudger_never_bursts_after_a_long_gap() {
        // e.g. the Mac was asleep: one nudge on wake, not a backlog.
        let mut n = Nudger::new(EVERY);
        let t = Instant::now();
        n.start(t);
        let late = t + Duration::from_secs(3600);
        assert!(n.poll(late));
        assert!(!n.poll(late));
        assert_eq!(n.deadline(), Some(late + EVERY));
    }

    #[test]
    fn restarting_does_not_reset_schedule_and_stop_clears_it() {
        let mut n = Nudger::new(EVERY);
        let t = Instant::now();
        n.start(t);
        assert!(n.poll(t));
        n.start(t + Duration::from_secs(10));
        assert_eq!(n.deadline(), Some(t + EVERY));
        n.stop();
        assert!(!n.is_running());
        assert!(!n.poll(t + EVERY));
    }

    #[test]
    fn nudges_only_when_idle_long_enough() {
        let threshold = Duration::from_secs(45);
        assert!(!needs_nudge(Duration::from_secs(5), threshold));
        assert!(!needs_nudge(Duration::from_secs(44), threshold));
        assert!(needs_nudge(threshold, threshold));
        assert!(needs_nudge(Duration::from_secs(300), threshold));
    }

    #[test]
    fn triple_click_toggles_once() {
        let mut d = DoubleClickDetector::new(WINDOW);
        let t = Instant::now();
        assert!(!d.click(t));
        assert!(d.click(t + Duration::from_millis(100)));
        assert!(!d.click(t + Duration::from_millis(200)));
    }
}
