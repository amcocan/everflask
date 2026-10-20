//! Placeholder flask icons, rasterised at runtime. Replaced by designed
//! artwork in `assets/icons` once the design phase lands.

use everflask_core::State;
use tray_icon::Icon;

const SIZE: u32 = 36;
const STROKE: f32 = 2.5;
/// Liquid fills the bulb below this row when the flask is full.
const LIQUID_TOP: f32 = 19.0;

/// Signed distance to the flask outline: negative inside, positive outside.
fn flask_sdf(x: f32, y: f32) -> f32 {
    let bulb = ((x - 18.0).powi(2) + (y - 23.5).powi(2)).sqrt() - 11.5;
    let neck = rect_sdf(x, y, 18.0, 9.0, 4.0, 6.0);
    let rim = rect_sdf(x, y, 18.0, 3.5, 6.0, 1.5);
    bulb.min(neck).min(rim)
}

fn rect_sdf(x: f32, y: f32, cx: f32, cy: f32, hw: f32, hh: f32) -> f32 {
    let dx = (x - cx).abs() - hw;
    let dy = (y - cy).abs() - hh;
    let outside = (dx.max(0.0).powi(2) + dy.max(0.0).powi(2)).sqrt();
    outside + dx.max(dy).min(0.0)
}

/// Coverage in 0..=1 for a band `d0 <= d <= d1`, with a 1px soft edge.
fn band(d: f32, d0: f32, d1: f32) -> f32 {
    let inner = (d - d0 + 0.5).clamp(0.0, 1.0);
    let outer = (d1 - d + 0.5).clamp(0.0, 1.0);
    inner.min(outer)
}

pub fn flask(state: State) -> Icon {
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for py in 0..SIZE {
        for px in 0..SIZE {
            let (x, y) = (px as f32 + 0.5, py as f32 + 0.5);
            let d = flask_sdf(x, y);
            let mut alpha = band(d, -STROKE, 0.0);
            if state == State::Full && y >= LIQUID_TOP {
                alpha = alpha.max(band(d, f32::MIN, 0.0));
            }
            // Template image: black + alpha; macOS recolours for light/dark.
            rgba.extend_from_slice(&[0, 0, 0, (alpha * 255.0).round() as u8]);
        }
    }
    Icon::from_rgba(rgba, SIZE, SIZE).expect("icon buffer matches dimensions")
}
