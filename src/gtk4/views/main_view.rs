use std::rc::Rc;
use gtk4::{gdk, style_context_add_provider_for_display, Builder, CssProvider, GestureClick, GridView, Label, ListItem, MultiSelection, NoSelection, Orientation, SignalListItemFactory, SingleSelection, StringObject, Widget};
use gtk4::gio::ListStore;
use gtk4::prelude::{BoxExt, Cast, EventControllerExt, GestureSingleExt, ListItemExt, ListModelExt, SelectionModelExt, StaticType, WidgetExt};
use crate::gtk4::views::inter::stackable::Stackable;
use crate::gtk4::windows::main_window::MainWindow;

use std::cell::RefCell;
use std::collections::HashMap;
use std::process::exit;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use rdev::{listen, EventType, Key};
use crate::bus::event_bus::{pause_event, register_event, resume_event, unregister_event};
use crate::bus::event_bus::EventPropagation::Continue;
use crate::bus::events::button_event::ButtonEvent;
use crate::bus::events::timer_event::TimerEvent;

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


        let time: Label = builder
            .object("time")
            .expect("Couldn't find 'time' in main_view.ui");


        let timer_running = Rc::new(AtomicBool::new(false));
        let now = Rc::new(RefCell::new(0u128));

        let timer_event_listener = Some(RefCell::new(register_event("timer_event", {
            let timer_running = Rc::clone(&timer_running);
            let now = Rc::clone(&now);
            move |event| {
                let event = event.as_any().downcast_ref::<TimerEvent>().unwrap();

                if timer_running.load(Ordering::Relaxed) {
                    time.set_label(&format!("{}", ms_to_hms(event.time - *now.borrow())));
                }

                Continue
            }
        }, true)));
        resume_event("timer_event", timer_event_listener.as_ref().unwrap().borrow().clone());


        let button_event_listener = Some(RefCell::new(register_event("button_event", {
            move |event| {
                let event = event.as_any().downcast_ref::<ButtonEvent>().unwrap();

                match event.button {
                    Key::BackQuote => {
                        if timer_running.load(Ordering::Relaxed) {
                            timer_running.store(false, Ordering::Relaxed);

                            return Continue;
                        }

                        *now.borrow_mut() = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                        timer_running.store(true, Ordering::Relaxed);
                    }
                    Key::Equal => {
                        exit(0);
                    }
                    _ => {}
                }

                Continue
            }
        }, true)));
        resume_event("button_event", button_event_listener.as_ref().unwrap().borrow().clone());

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

fn ms_to_hms(ms: u128) -> String {
    let total_seconds = ms / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
