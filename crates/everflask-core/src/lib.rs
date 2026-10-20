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

    #[test]
    fn triple_click_toggles_once() {
        let mut d = DoubleClickDetector::new(WINDOW);
        let t = Instant::now();
        assert!(!d.click(t));
        assert!(d.click(t + Duration::from_millis(100)));
        assert!(!d.click(t + Duration::from_millis(200)));
    }
}
