//! Menu-bar flask that keeps the machine awake while full. Double-click the
//! icon (or use the right-click menu) to toggle.

mod icon;
mod menubar;

use std::time::{Duration, Instant};

use everflask_core::{DoubleClickDetector, KeepAwake, State};
use everflask_platform::macos::MacKeepAwake;
use icon::{Liquid, OffStyle};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

/// Close to the macOS default; read the user's setting in a later pass.
const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(400);
const LIQUID: Liquid = Liquid::Red;
const OFF_STYLE: OffStyle = OffStyle::Grey;

enum UserEvent {
    Tray(TrayIconEvent),
    Menu(MenuEvent),
}

fn tooltip(state: State) -> &'static str {
    match state {
        State::Full => "Everflask: keeping awake",
        State::Empty => "Everflask: off",
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
    let quit_item = MenuItem::new("Quit Everflask", true, None);
    menu.append(&toggle_item).expect("append menu item");
    menu.append(&quit_item).expect("append menu item");

    let mut state = State::default();
    let mut awake = MacKeepAwake::new();
    let mut clicks = DoubleClickDetector::new(DOUBLE_CLICK_WINDOW);
    let mut tray: Option<TrayIcon> = None;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        let mut toggle = false;
        let mut redraw = false;
        match event {
            // macOS requires the status item to be created after the loop starts.
            Event::NewEvents(StartCause::Init) => {
                tray = Some(
                    TrayIconBuilder::new()
                        .with_tooltip(tooltip(state))
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
            Event::UserEvent(UserEvent::Menu(e)) if e.id == *quit_item.id() => {
                let _ = awake.disable();
                tray.take();
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }

        if toggle {
            let next = state.toggled();
            let result = match next {
                State::Full => awake.enable(),
                State::Empty => awake.disable(),
            };
            // The icon must never claim the machine is awake when it isn't.
            if let Err(e) = result {
                eprintln!("everflask: could not switch to {next:?}: {e}");
                return;
            }
            state = next;
            redraw = true;
            println!("everflask: {state:?}");
        }

        if redraw {
            if let Some(tray) = &tray {
                menubar::set_flask(tray, icon::flask(state, LIQUID, OFF_STYLE));
                let _ = tray.set_tooltip(Some(tooltip(state)));
            }
        }
    });
}
