//! Manual test: proves a nudge resets the system idle timer and doesn't
//! move the pointer. Needs the Accessibility permission for the process that
//! runs it, and no real input during the run:
//!
//!     cargo test -p everflask-platform --test macos_activity -- --ignored
#![cfg(target_os = "macos")]

use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use everflask_platform::macos::{has_activity_permission, nudge_activity};

fn hid_idle_secs() -> f64 {
    let out = Command::new("ioreg")
        .args(["-c", "IOHIDSystem"])
        .output()
        .expect("run ioreg");
    let text = String::from_utf8_lossy(&out.stdout);
    let ns: u64 = text
        .lines()
        .find_map(|l| l.split("\"HIDIdleTime\" = ").nth(1))
        .and_then(|v| v.trim().parse().ok())
        .expect("HIDIdleTime in ioreg output");
    ns as f64 / 1e9
}

#[test]
#[ignore = "needs Accessibility permission and an idle machine"]
fn nudge_resets_idle_time_without_moving_pointer() {
    assert!(
        has_activity_permission(),
        "grant Accessibility to the app running this test (e.g. Terminal)"
    );
    sleep(Duration::from_secs(4));
    let before = hid_idle_secs();
    assert!(
        before >= 3.0,
        "don't touch the mouse/keyboard during the test"
    );

    let pointer = || {
        use objc2_core_graphics::CGEvent;
        let p = CGEvent::location(CGEvent::new(None).as_deref());
        (p.x, p.y)
    };
    let start = pointer();
    nudge_activity().unwrap();
    sleep(Duration::from_millis(300));

    assert!(
        hid_idle_secs() < 1.0,
        "idle time was {before:.1}s and did not reset"
    );
    assert_eq!(pointer(), start, "pointer moved");
}
