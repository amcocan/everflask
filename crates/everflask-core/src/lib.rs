//! Platform-independent state and logic. No OS calls live here.

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
}
