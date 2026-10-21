//! Puts the flask on the status item's button as an appearance-aware image.
//!
//! Colour artwork can't be a template image, so macOS can't recolour it for
//! us. Instead the image is backed by a drawing handler that AppKit calls on
//! every redraw, including when the menu bar flips between light and dark
//! (Dark Mode, or Light Mode over a dark wallpaper). The handler picks the
//! matching artwork from the appearance it's being drawn in, so the switch
//! is instant and needs no polling.

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2::AnyThread;
use objc2_app_kit::{NSAppearance, NSImage};
use objc2_foundation::{ns_string, MainThreadMarker, NSArray, NSData, NSRect, NSSize};
use tray_icon::TrayIcon;

use crate::icon::Artwork;

/// Menu-bar icons are capped at 22pt high; the PNGs are 44px (2x).
const SIZE_PT: f64 = 22.0;

fn decode(png: &'static [u8]) -> Retained<NSImage> {
    let data = NSData::with_bytes(png);
    NSImage::initWithData(NSImage::alloc(), &data).expect("embedded PNG decodes")
}

/// True if AppKit is currently drawing in a dark appearance.
fn drawing_dark() -> bool {
    // Same values as AppKit's NSAppearanceNameAqua / NSAppearanceNameDarkAqua.
    let dark = ns_string!("NSAppearanceNameDarkAqua");
    let names = NSArray::from_slice(&[ns_string!("NSAppearanceNameAqua"), dark]);
    NSAppearance::currentDrawingAppearance()
        .bestMatchFromAppearancesWithNames(&names)
        .is_some_and(|best| &*best == dark)
}

pub fn set_flask(tray: &TrayIcon, art: Artwork) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let Some(button) = tray.ns_status_item().and_then(|item| item.button(mtm)) else {
        return;
    };

    let light = decode(art.light);
    let dark = decode(art.dark);
    let handler = RcBlock::new(move |rect: NSRect| -> Bool {
        let art = if drawing_dark() { &dark } else { &light };
        art.drawInRect(rect);
        Bool::YES
    });
    let image = NSImage::imageWithSize_flipped_drawingHandler(
        NSSize::new(SIZE_PT, SIZE_PT),
        false,
        &handler,
    );
    button.setImage(Some(&image));
}
