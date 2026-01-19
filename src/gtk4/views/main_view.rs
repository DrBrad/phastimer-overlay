use std::rc::Rc;
use gtk4::{gdk, style_context_add_provider_for_display, Builder, CssProvider, GestureClick, GridView, Label, ListItem, MultiSelection, NoSelection, Orientation, SignalListItemFactory, SingleSelection, StringObject, Widget};
use gtk4::gio::ListStore;
use gtk4::prelude::{BoxExt, Cast, EventControllerExt, GestureSingleExt, ListItemExt, ListModelExt, SelectionModelExt, StaticType, WidgetExt};
use crate::gtk4::views::inter::stackable::Stackable;
use crate::gtk4::windows::main_window::MainWindow;

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::fmt::format;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use rdev::{listen, EventType, Key};
use crate::bus::event_bus::{pause_event, register_event, resume_event, unregister_event};
use crate::bus::event_bus::EventPropagation::Continue;
use crate::bus::events::button_event::ButtonEvent;
use crate::bus::events::timer_event::TimerEvent;
use crate::gtk4::windows::console_window::ConsoleWindow;
use crate::settings::{BLOOD_MOON, GHOST_SPEED, KEY_MS, KEY_OBAMBO_RESET, KEY_OBAMBO_START, KEY_RESET, KEY_TIMER_RESET, KEY_TIMER_START};
use crate::utils::bpm::TapState;

pub struct MainView {
    pub root: gtk4::Box,
    pub button_event_listener: Option<RefCell<u32>>,
    pub timer_event_listener: Option<RefCell<u32>>
}

impl MainView {

    pub fn new(window: &MainWindow) -> Self {
        let builder = Builder::from_resource("/smudgetimer/rust/res/ui/main_view.ui");

        let provider = CssProvider::new();
        provider.load_from_resource("/smudgetimer/rust/res/ui/main_view.css");
        style_context_add_provider_for_display(&gdk::Display::default().unwrap(), &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let root: gtk4::Box = builder
            .object("root")
            .expect("Couldn't find 'root' in main_view.ui");


        let smudge: Label = builder
            .object("smudge")
            .expect("Couldn't find 'smudge' in main_view.ui");

        let obombo: Label = builder
            .object("obombo")
            .expect("Couldn't find 'obombo' in main_view.ui");

        let bps: Label = builder
            .object("bps")
            .expect("Couldn't find 'bps' in main_view.ui");


        let smudge_timer_running = Rc::new(AtomicBool::new(false));
        let obombo_timer_running = Rc::new(AtomicBool::new(false));
        let smudge_now = Rc::new(RefCell::new(0u128));
        let obombo_now = Rc::new(RefCell::new(0u128));
        let mut obombo_state = Rc::new(RefCell::new(true));

        let timer_event_listener = Some(RefCell::new(register_event("timer_event", {
            let smudge = smudge.clone();
            let obombo = obombo.clone();
            let smudge_timer_running = Rc::clone(&smudge_timer_running);
            let smudge_now = Rc::clone(&smudge_now);
            let obombo_timer_running = Rc::clone(&obombo_timer_running);
            let obombo_now = Rc::clone(&obombo_now);
            let obombo_state = Rc::clone(&obombo_state);

            move |id, event| {
                let event = event.as_any().downcast_ref::<TimerEvent>().unwrap();

                if smudge_timer_running.load(Ordering::Relaxed) && event.time >= *smudge_now.borrow() {
                    smudge.set_label(&format!("{}", ms_to_msm(event.time - *smudge_now.borrow())));
                }

                if obombo_timer_running.load(Ordering::Relaxed) && event.time >= *obombo_now.borrow() {
                    let elapsed = (event.time - *obombo_now.borrow())/10;
                    if elapsed < 6000 {
                        if *obombo_state.borrow() {
                            *obombo_state.borrow_mut() = false;
                        }
                    } else if (elapsed - 6000) % 12000 == 0 {
                        let new_state = !*obombo_state.borrow();
                        *obombo_state.borrow_mut() = new_state;
                        obombo.set_label(if new_state { "AGGRO" } else { "CALM" });
                    }
                }

                Continue
            }
        }, false)));

        const SPEEDS: [f64; 5] = [
            0.5,
            0.75,
            1.0,
            1.25,
            1.50
        ];

        let button_event_listener = Some(RefCell::new(register_event("button_event", {
            let smudge = smudge.clone();
            let obombo = obombo.clone();
            let window = window.window.clone();

            let tap_state = RefCell::new(TapState::default());

            move |id, event| unsafe {
                let event = event.as_any().downcast_ref::<ButtonEvent>().unwrap();

                match event.button {
                    Key::ControlRight => {
                        ConsoleWindow::new(&window);
                    }
                    k if k == KEY_TIMER_START => {
                        *smudge_now.borrow_mut() = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                        smudge_timer_running.store(true, Ordering::Relaxed);
                    }
                    k if k == KEY_TIMER_RESET => {
                        smudge_timer_running.store(false, Ordering::Relaxed);
                        smudge.set_label("00:00.00");
                    }
                    k if k == KEY_OBAMBO_START => {
                        *obombo_now.borrow_mut() = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                        obombo_timer_running.store(true, Ordering::Relaxed);
                        obombo.set_label("CALM");
                    }
                    k if k == KEY_OBAMBO_RESET => {
                        obombo_timer_running.store(false, Ordering::Relaxed);
                        *obombo_state.borrow_mut() = true;
                        obombo.set_label("NONE");
                    }
                    k if k == KEY_RESET => {
                        smudge_timer_running.store(false, Ordering::Relaxed);
                        smudge.set_label("00:00.00");

                        obombo_timer_running.store(false, Ordering::Relaxed);
                        *obombo_state.borrow_mut() = true;
                        obombo.set_label("NONE");

                        tap_state.borrow_mut().reset();
                        bps.set_label("0.00 m/s");
                    }
                    k if k == KEY_MS => {
                        if let Some((bpm, mut ms)) = tap_state.borrow_mut().tap_and_compute() {
                            ms = ms/SPEEDS[GHOST_SPEED];

                            if BLOOD_MOON {
                                ms = ms*0.85;
                            }

                            bps.set_label(&format!("{:.2} m/s", ms));

                        } else {
                            bps.set_label("0.00 m/s");
                        }
                    }
                    _ => {}
                }

                Continue
            }
        }, false)));

        Self {
            root,
            button_event_listener,
            timer_event_listener
        }
    }
}

impl Stackable for MainView {

    fn get_name(&self) -> String {
        String::from("main_view")
    }

    fn get_root(&self) -> &Widget {
        self.root.upcast_ref()
    }

    fn on_create(&self) {
        //(self.show_title_bar)(true);
    }

    fn on_resume(&self) {
        //(self.show_title_bar)(true);
    }

    fn on_pause(&self) {
        if let Some(button_event_listener) = &self.button_event_listener {
            pause_event("button_event", *button_event_listener.borrow());
        }

        if let Some(timer_event_listener) = &self.timer_event_listener {
            pause_event("timer_event", *timer_event_listener.borrow());
        }
    }

    fn on_destroy(&self) {
        if let Some(button_event_listener) = &self.button_event_listener {
            unregister_event("button_event", *button_event_listener.borrow());
        }

        if let Some(timer_event_listener) = &self.timer_event_listener {
            unregister_event("timer_event", *timer_event_listener.borrow());
        }
    }
}

fn ms_to_msm(ms: u128) -> String {
    let mut total_seconds = ms / 1000;
    let mut centiseconds = ((ms % 1000) + 5) / 10; // rounded

    if centiseconds == 100 {
        centiseconds = 0;
        total_seconds += 1;
    }

    let seconds = total_seconds % 60;
    let minutes = (total_seconds / 60) % 60;

    format!("{:02}:{:02}.{:02}", minutes, seconds, centiseconds)
}
