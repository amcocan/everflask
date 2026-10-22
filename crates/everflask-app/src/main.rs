//! Menu-bar flask. While full it keeps the machine awake and, if the user
//! has granted Accessibility, keeps chat apps from showing them as Away.
//! Double-click the icon (or use the right-click menu) to toggle.

mod dialog;
mod icon;
mod instance;
mod menubar;

use std::time::{Duration, Instant};

use everflask_core::{needs_nudge, DoubleClickDetector, KeepAwake, Nudger, State};
use everflask_platform::macos::uninstall::{self, BUNDLE_ID};
use everflask_platform::macos::{
    activity_permission, nudge_activity, open_accessibility_settings, request_activity_permission,
    seconds_since_input, ActivityPermission, MacKeepAwake,
};
use icon::{Liquid, OffStyle};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

/// Close to the MacOS default; read the user's setting in a later pass.
const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(400);
/// How often to check whether a nudge is needed.
const NUDGE_INTERVAL: Duration = Duration::from_secs(60);
/// Only nudge after this much real idle time. Worst case the idle timer
/// reaches IDLE_THRESHOLD + NUDGE_INTERVAL (105 s), well inside the ~5 minute
/// timeout chat apps typically use.
const IDLE_THRESHOLD: Duration = Duration::from_secs(45);
const LIQUID: Liquid = Liquid::Red;
const OFF_STYLE: OffStyle = OffStyle::Grey;

/// Diagnostic log to stderr, only when `EVERFLASK_DEBUG` is set. Launch with
/// `open --env EVERFLASK_DEBUG=1 --stderr <file> EverFlask.app`.
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if std::env::var_os("EVERFLASK_DEBUG").is_some() {
            eprintln!("everflask[debug]: {}", format!($($arg)*));
        }
    };
}

enum UserEvent {
    Tray(TrayIconEvent),
    Menu(MenuEvent),
}

fn tooltip(state: State, permission: ActivityPermission) -> &'static str {
    match (state, permission) {
        (State::Empty, _) => "EverFlask: Off.",
        (State::Full, ActivityPermission::Active) => "EverFlask: Keeping awake and active.",
        (State::Full, ActivityPermission::NotGranted) => {
            "EverFlask: Keeping awake. Allow Accessibility to stay Active in chat apps."
        }
    }
}

/// Removes EverFlask's settings, permissions and the app itself, then exits.
/// Never returns once the user has confirmed.
fn uninstall(awake: &mut MacKeepAwake, nudger: &mut Nudger) {
    if !dialog::confirm(
        "Uninstall EverFlask?",
        "This removes EverFlask's settings and its Accessibility permission, \
         then moves EverFlask to the Trash.",
        "Uninstall",
    ) {
        return;
    }
    nudger.stop();
    let _ = awake.disable();

    // Permissions must be reset while MacOS can still find the app.
    let mut report = uninstall::remove_artifacts(BUNDLE_ID);
    match uninstall::trash_running_bundle(BUNDLE_ID) {
        Ok(path) => report
            .removed
            .push(format!("{} (moved to Trash)", path.display())),
        Err(e) => report.failed.push(format!("app bundle: {e}")),
    }

    let body = if report.failed.is_empty() {
        "EverFlask and its settings have been removed.".to_string()
    } else {
        format!(
            "Some items couldn't be removed:\n\n• {}\n\nYou can remove them by hand; see \
             \"Uninstalling\" in the README.",
            report.failed.join("\n• ")
        )
    };
    dialog::inform("EverFlask uninstalled", &body);

    // Catch anything AppKit wrote while the dialogs were open, then exit
    // without AppKit's normal shutdown, which could save preferences again.
    uninstall::remove_preferences(BUNDLE_ID, &mut uninstall::Report::default());
    std::process::exit(0);
}

fn main() {
    if instance::another_is_running() {
        return;
    }

    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
        // Menu-bar only: no Dock icon, no app switcher entry.
        event_loop.set_activation_policy(ActivationPolicy::Accessory);
    }

    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |e| {
        let _ = proxy.send_event(UserEvent::Tray(e));
    }));
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |e| {
        let _ = proxy.send_event(UserEvent::Menu(e));
    }));

    let menu = Menu::new();
    let toggle_item = MenuItem::new("Toggle", true, None);
    // Shown only while the permission isn't active (see `redraw` below).
    let allow_item = MenuItem::new("Allow Accessibility", true, None);
    let uninstall_item = MenuItem::new("Uninstall EverFlask…", true, None);
    let quit_item = MenuItem::new("Quit EverFlask", true, None);
    menu.append(&toggle_item).expect("append menu item");
    menu.append(&allow_item).expect("append menu item");
    menu.append(&PredefinedMenuItem::separator())
        .expect("append menu item");
    menu.append(&uninstall_item).expect("append menu item");
    menu.append(&quit_item).expect("append menu item");

    let mut state = State::default();
    let mut awake = MacKeepAwake::new();
    let mut clicks = DoubleClickDetector::new(DOUBLE_CLICK_WINDOW);
    // Nudges run only while the flask is full, and only post input if the
    // user granted Accessibility. Without it, full means keep-awake only.
    let mut nudger = Nudger::new(NUDGE_INTERVAL);
    let mut permission = activity_permission();
    debug_log!("started; Accessibility: {permission:?}");
    let mut tray: Option<TrayIcon> = None;
    let mut allow_shown = true;

    event_loop.run(move |event, _, control_flow| {
        let mut toggle = false;
        let mut redraw = false;
        let mut quit = false;
        match event {
            // macOS requires the status item to be created after the loop starts.
            Event::NewEvents(StartCause::Init) => {
                tray = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(menu.clone()))
                        .with_menu_on_left_click(false)
                        .build()
                        .expect("create tray icon"),
                );
                redraw = true;
            }
            Event::UserEvent(UserEvent::Tray(TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            })) => toggle = cfg!(target_os = "macos") && clicks.click(Instant::now()),
            // Windows reports double-clicks natively (alongside the clicks above,
            // which is why the detector is macOS-only).
            Event::UserEvent(UserEvent::Tray(TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            })) => toggle = true,
            Event::UserEvent(UserEvent::Menu(e)) if e.id == *toggle_item.id() => toggle = true,
            Event::UserEvent(UserEvent::Menu(e)) if e.id == *allow_item.id() => {
                open_accessibility_settings();
                permission = activity_permission();
                redraw = true;
            }
            Event::UserEvent(UserEvent::Menu(e)) if e.id == *uninstall_item.id() => {
                uninstall(&mut awake, &mut nudger);
            }
            Event::UserEvent(UserEvent::Menu(e)) if e.id == *quit_item.id() => quit = true,
            Event::NewEvents(StartCause::ResumeTimeReached { .. })
                if nudger.poll(Instant::now()) =>
            {
                let now_permission = activity_permission();
                if now_permission != permission {
                    debug_log!("Accessibility: {permission:?} -> {now_permission:?}");
                    permission = now_permission;
                    redraw = true;
                }
                let idle = Duration::from_secs_f64(seconds_since_input().max(0.0));
                let needed = needs_nudge(idle, IDLE_THRESHOLD);
                debug_log!(
                    "tick: idle {:.1}s, nudge needed: {needed}",
                    idle.as_secs_f64()
                );
                if needed {
                    match nudge_activity() {
                        Ok(()) => debug_log!("nudged"),
                        Err(e) => debug_log!("nudge skipped: {e}"),
                    }
                }
            }
            _ => {}
        }

        if quit {
            nudger.stop();
            let _ = awake.disable();
            tray.take();
            *control_flow = ControlFlow::Exit;
            return;
        }

        if toggle {
            let next = state.toggled();
            let result = match next {
                State::Full => awake.enable(),
                State::Empty => awake.disable(),
            };
            // The icon must never claim the machine is awake when it isn't.
            match result {
                Ok(()) => {
                    state = next;
                    redraw = true;
                    println!("everflask: {state:?}");
                    if state == State::Full && permission == ActivityPermission::NotGranted {
                        // Shows the MacOS prompt the first time only; later
                        // calls just report the current answer.
                        request_activity_permission();
                        permission = activity_permission();
                        debug_log!("requested permission; now {permission:?}");
                    }
                }
                Err(e) => eprintln!("everflask: could not switch to {next:?}: {e}"),
            }
        }

        if state == State::Full {
            nudger.start(Instant::now());
        } else {
            nudger.stop();
        }

        if redraw {
            // Right after Toggle, the item it relates to.
            let want_allow = permission != ActivityPermission::Active;
            if want_allow != allow_shown {
                let result = if want_allow {
                    menu.insert(&allow_item, 1)
                } else {
                    menu.remove(&allow_item)
                };
                match result {
                    Ok(()) => allow_shown = want_allow,
                    Err(e) => eprintln!("everflask: could not update menu: {e}"),
                }
            }
            if let Some(tray) = &tray {
                menubar::set_flask(tray, icon::flask(state, LIQUID, OFF_STYLE));
                let _ = tray.set_tooltip(Some(tooltip(state, permission)));
            }
        }

        // Sleep until the next nudge, or indefinitely when none are scheduled.
        *control_flow = match nudger.deadline() {
            Some(at) => ControlFlow::WaitUntil(at),
            None => ControlFlow::Wait,
        };
    });
}
