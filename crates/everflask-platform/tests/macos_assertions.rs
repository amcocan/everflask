//! Checks the real system state via `pmset`, not just our own bookkeeping.
#![cfg(target_os = "macos")]

use std::process::Command;

use everflask_core::KeepAwake;
use everflask_platform::macos::{MacKeepAwake, ASSERTION_NAME};

/// Number of live assertions this test process holds under our name.
fn our_assertions() -> usize {
    let out = Command::new("pmset")
        .args(["-g", "assertions"])
        .output()
        .expect("run pmset");
    let pid = format!("pid {}(", std::process::id());
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| l.contains(&pid) && l.contains(ASSERTION_NAME))
        .count()
}

#[test]
fn enable_disable_and_drop_manage_real_assertions() {
    let mut awake = MacKeepAwake::new();
    assert_eq!(our_assertions(), 0);

    awake.enable().unwrap();
    assert_eq!(our_assertions(), 2);

    // Idempotent: a second enable must not stack assertions.
    awake.enable().unwrap();
    assert_eq!(our_assertions(), 2);

    awake.disable().unwrap();
    assert_eq!(our_assertions(), 0);

    awake.enable().unwrap();
    drop(awake);
    assert_eq!(our_assertions(), 0);
}
