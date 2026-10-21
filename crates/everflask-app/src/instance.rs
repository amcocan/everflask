//! One EverFlask at a time. Two copies would show two flasks with separate
//! state and could double the activity nudge rate.

use objc2_app_kit::NSRunningApplication;

/// True if another copy of this app was already running when we started.
/// Builds run outside an `.app` (e.g. `cargo run`) have no bundle
/// identifier and are never blocked.
pub fn another_is_running() -> bool {
    let me = NSRunningApplication::currentApplication();
    let Some(bundle_id) = me.bundleIdentifier() else {
        return false;
    };
    let my_pid = me.processIdentifier();
    let launched = |app: &NSRunningApplication| {
        app.launchDate()
            .map_or(f64::MAX, |d| d.timeIntervalSinceReferenceDate())
    };
    let my_launch = launched(&me);
    NSRunningApplication::runningApplicationsWithBundleIdentifier(&bundle_id)
        .iter()
        .filter(|other| other.processIdentifier() != my_pid)
        // The earlier launch wins; PID breaks an exact tie, so if two start
        // together exactly one survives.
        .any(|other| {
            let theirs = launched(&other);
            theirs < my_launch || (theirs == my_launch && other.processIdentifier() < my_pid)
        })
}
