//! macOS backend: IOKit power assertions, the same mechanism `caffeinate` uses.
//! Assertions are released on `disable`, on drop, and by the kernel if the
//! process dies, so a crash can never leave the machine stuck awake.

use std::fmt;

use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use everflask_core::KeepAwake;

type IOPMAssertionID = u32;
type IOReturn = i32;

const K_IOPM_ASSERTION_LEVEL_ON: u32 = 255;
const K_IO_RETURN_SUCCESS: IOReturn = 0;

/// Name shown in `pmset -g assertions`.
pub const ASSERTION_NAME: &str = "Everflask: keeping awake";

/// Display sleep also blocks idle system sleep; we hold both so intent is
/// explicit and survives any future change in that coupling.
const ASSERTION_TYPES: [&str; 2] = ["PreventUserIdleDisplaySleep", "PreventUserIdleSystemSleep"];

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOPMAssertionCreateWithName(
        assertion_type: CFStringRef,
        level: u32,
        name: CFStringRef,
        id: *mut IOPMAssertionID,
    ) -> IOReturn;
    fn IOPMAssertionRelease(id: IOPMAssertionID) -> IOReturn;
}

#[derive(Debug)]
pub struct Error {
    call: &'static str,
    code: IOReturn,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} failed with IOReturn {:#x}", self.call, self.code)
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Default)]
pub struct MacKeepAwake {
    held: Vec<IOPMAssertionID>,
}

impl MacKeepAwake {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_enabled(&self) -> bool {
        !self.held.is_empty()
    }

    fn release_all(&mut self) -> Result<(), Error> {
        let mut result = Ok(());
        for id in self.held.drain(..) {
            // SAFETY: `id` came from a successful IOPMAssertionCreateWithName
            // and is released exactly once because `drain` removes it.
            let code = unsafe { IOPMAssertionRelease(id) };
            if code != K_IO_RETURN_SUCCESS && result.is_ok() {
                result = Err(Error {
                    call: "IOPMAssertionRelease",
                    code,
                });
            }
        }
        result
    }
}

impl KeepAwake for MacKeepAwake {
    type Error = Error;

    fn enable(&mut self) -> Result<(), Error> {
        if self.is_enabled() {
            return Ok(());
        }
        let name = CFString::new(ASSERTION_NAME);
        for kind in ASSERTION_TYPES {
            let kind = CFString::new(kind);
            let mut id: IOPMAssertionID = 0;
            // SAFETY: both CFStrings outlive the call, and `id` is a valid
            // out-pointer to a local.
            let code = unsafe {
                IOPMAssertionCreateWithName(
                    kind.as_concrete_TypeRef(),
                    K_IOPM_ASSERTION_LEVEL_ON,
                    name.as_concrete_TypeRef(),
                    &mut id,
                )
            };
            if code != K_IO_RETURN_SUCCESS {
                // Don't leave a half-enabled state behind.
                let _ = self.release_all();
                return Err(Error {
                    call: "IOPMAssertionCreateWithName",
                    code,
                });
            }
            self.held.push(id);
        }
        Ok(())
    }

    fn disable(&mut self) -> Result<(), Error> {
        self.release_all()
    }
}

impl Drop for MacKeepAwake {
    fn drop(&mut self) {
        let _ = self.release_all();
    }
}

// --- Activity simulation -------------------------------------------------
//
// The only input Everflask ever sends: a mouse-move event to the pointer's
// current position. Nothing moves on screen, but macOS counts it as user
// input, which resets the idle timers chat apps use for "Away".
// Requires the Accessibility permission (System Settings > Privacy &
// Security > Accessibility).

use objc2_core_graphics::{
    CGEvent, CGEventTapLocation, CGEventType, CGMouseButton, CGPreflightPostEventAccess,
    CGRequestPostEventAccess,
};

#[derive(Debug, PartialEq, Eq)]
pub enum ActivityError {
    /// The Accessibility permission has not been granted (or was revoked).
    NoPermission,
    /// CoreGraphics could not create the event.
    EventCreation,
}

impl fmt::Display for ActivityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActivityError::NoPermission => write!(f, "Accessibility permission not granted"),
            ActivityError::EventCreation => write!(f, "could not create mouse event"),
        }
    }
}

impl std::error::Error for ActivityError {}

/// Whether Everflask may post input events. Never prompts.
pub fn has_activity_permission() -> bool {
    CGPreflightPostEventAccess()
}

/// Asks macOS for the permission, showing the system prompt if it hasn't
/// been decided yet. Returns whether it is granted right now; the user may
/// still grant it later in System Settings.
pub fn request_activity_permission() -> bool {
    CGRequestPostEventAccess()
}

/// Posts one zero-distance mouse move at the pointer's current position.
pub fn nudge_activity() -> Result<(), ActivityError> {
    if !has_activity_permission() {
        return Err(ActivityError::NoPermission);
    }
    // An empty event carries the pointer's current location.
    let here = CGEvent::location(CGEvent::new(None).as_deref());
    let event = CGEvent::new_mouse_event(None, CGEventType::MouseMoved, here, CGMouseButton::Left)
        .ok_or(ActivityError::EventCreation)?;
    CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&event));
    Ok(())
}
