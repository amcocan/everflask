//! Menu-bar flask that keeps the machine awake while full. Double-click the
//! icon (or use the right-click menu) to toggle.

mod icon;
mod menubar;

use std::time::{Duration, Instant};

use everflask_core::{DoubleClickDetector, KeepAwake, Nudger, State};
use everflask_platform::macos::{
    nudge_activity, request_activity_permission, ActivityError, MacKeepAwake,
};
use icon::{Liquid, OffStyle};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

/// Close to the macOS default; read the user's setting in a later pass.
const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(400);
/// Well inside the ~5 minute idle timeout chat apps typically use.
const NUDGE_INTERVAL: Duration = Duration::from_secs(60);
const LIQUID: Liquid = Liquid::Red;
const OFF_STYLE: OffStyle = OffStyle::Grey;

enum UserEvent {
    Tray(TrayIconEvent),
    Menu(MenuEvent),
}

fn tooltip(state: State, simulating: bool) -> &'static str {
    match (state, simulating) {
        (State::Full, true) => "Everflask: keeping awake + simulating activity",
        (State::Full, false) => "Everflask: keeping awake",
        (State::Empty, _) => "Everflask: off",
    }
}

fn main() {
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
    // Opt-in and off by default: see "Simulate activity" in the README.
    let simulate_item = CheckMenuItem::new(
        "Simulate activity (keeps chat apps Active)",
        true,
        false,
        None,
    );
    let quit_item = MenuItem::new("Quit Everflask", true, None);
    menu.append(&toggle_item).expect("append menu item");
    menu.append(&simulate_item).expect("append menu item");
    menu.append(&PredefinedMenuItem::separator())
        .expect("append menu item");
    menu.append(&quit_item).expect("append menu item");

    let mut state = State::default();
    let mut awake = MacKeepAwake::new();
    let mut clicks = DoubleClickDetector::new(DOUBLE_CLICK_WINDOW);
    // Nudges run only while the flask is full AND the user opted in.
    let mut nudger = Nudger::new(NUDGE_INTERVAL);
    let mut simulate = false;
    let mut tray: Option<TrayIcon> = None;

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
            Event::UserEvent(UserEvent::Menu(e)) if e.id == *simulate_item.id() => {
                // The menu has already flipped the checkmark.
                simulate = simulate_item.is_checked();
                if simulate && !request_activity_permission() {
                    eprintln!(
                        "everflask: grant Accessibility in System Settings > Privacy & \
                         Security, relaunch Everflask, then turn Simulate activity on again"
                    );
                    simulate = false;
                }
                simulate_item.set_checked(simulate);
                redraw = true;
            }
            Event::UserEvent(UserEvent::Menu(e)) if e.id == *quit_item.id() => quit = true,
            Event::NewEvents(StartCause::ResumeTimeReached { .. })
                if nudger.poll(Instant::now()) =>
            {
                if let Err(e) = nudge_activity() {
                    eprintln!("everflask: activity nudge failed: {e}");
                    if e == ActivityError::NoPermission {
                        // Permission revoked: turn the mode off visibly.
                        simulate = false;
                        simulate_item.set_checked(false);
                        redraw = true;
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
                }
                Err(e) => eprintln!("everflask: could not switch to {next:?}: {e}"),
            }
        }

        if state == State::Full && simulate {
            nudger.start(Instant::now());
        } else {
            nudger.stop();
        }

        if redraw {
            if let Some(tray) = &tray {
                menubar::set_flask(tray, icon::flask(state, LIQUID, OFF_STYLE));
                let _ = tray.set_tooltip(Some(tooltip(state, nudger.is_running())));
            }
        }

        // Sleep until the next nudge, or indefinitely when none are scheduled.
        *control_flow = match nudger.deadline() {
            Some(at) => ControlFlow::WaitUntil(at),
            None => ControlFlow::Wait,
        };
    });
}
