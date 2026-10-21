//! Native alerts. EverFlask has no windows, so it brings itself forward
//! first; otherwise the alert could open behind other apps.

use objc2_app_kit::{NSAlert, NSAlertFirstButtonReturn, NSAlertStyle, NSApplication};
use objc2_foundation::{MainThreadMarker, NSString};

fn alert(
    mtm: MainThreadMarker,
    title: &str,
    body: &str,
    style: NSAlertStyle,
) -> objc2::rc::Retained<NSAlert> {
    // The replacement, `activate()`, needs MacOS 14; we support 11+.
    #[allow(deprecated)]
    NSApplication::sharedApplication(mtm).activateIgnoringOtherApps(true);
    let alert = NSAlert::new(mtm);
    alert.setAlertStyle(style);
    alert.setMessageText(&NSString::from_str(title));
    alert.setInformativeText(&NSString::from_str(body));
    alert
}

/// Shows a destructive-action confirmation. True if the user confirmed.
pub fn confirm(title: &str, body: &str, confirm_label: &str) -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    let alert = alert(mtm, title, body, NSAlertStyle::Critical);
    alert.addButtonWithTitle(&NSString::from_str(confirm_label));
    alert.addButtonWithTitle(&NSString::from_str("Cancel"));
    alert.runModal() == NSAlertFirstButtonReturn
}

pub fn inform(title: &str, body: &str) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    alert(mtm, title, body, NSAlertStyle::Informational).runModal();
}
