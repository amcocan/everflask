//! Flask artwork, pre-rendered from `assets/icons/*.svg` by `build.rs`.

use everflask_core::State;

/// Liquid colour for the full flask. Artwork exists for red, green and blue.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Green and Blue are selectable but not yet exposed.
pub enum Liquid {
    Red,
    Green,
    Blue,
}

/// How the flask looks when off.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Both styles are under evaluation.
pub enum OffStyle {
    Empty,
    Grey,
}

/// A PNG for each menu-bar appearance.
pub struct Artwork {
    pub light: &'static [u8],
    pub dark: &'static [u8],
}

macro_rules! pair {
    ($suffix:literal) => {
        Artwork {
            light: include_bytes!(concat!(env!("OUT_DIR"), "/Light-", $suffix, ".png")),
            dark: include_bytes!(concat!(env!("OUT_DIR"), "/Dark-", $suffix, ".png")),
        }
    };
}

pub fn flask(state: State, liquid: Liquid, off: OffStyle) -> Artwork {
    match (state, liquid, off) {
        (State::Empty, _, OffStyle::Empty) => pair!("Off"),
        (State::Empty, _, OffStyle::Grey) => pair!("Off-Grey"),
        (State::Full, Liquid::Red, _) => pair!("On-R"),
        (State::Full, Liquid::Green, _) => pair!("On-G"),
        (State::Full, Liquid::Blue, _) => pair!("On-B"),
    }
}
