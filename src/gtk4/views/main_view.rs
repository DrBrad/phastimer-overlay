use std::rc::Rc;
use gtk4::{gdk, style_context_add_provider_for_display, Builder, CssProvider, GestureClick, GridView, Label, ListItem, MultiSelection, NoSelection, Orientation, SignalListItemFactory, SingleSelection, StringObject, Widget};
use gtk4::gio::ListStore;
use gtk4::prelude::{BoxExt, Cast, EventControllerExt, GestureSingleExt, ListItemExt, ListModelExt, SelectionModelExt, StaticType, WidgetExt};
use crate::{GhostState, GHOSTS};
use crate::gtk4::views::inter::stackable::Stackable;
use crate::gtk4::windows::main_window::MainWindow;

use std::cell::RefCell;
use std::collections::HashMap;
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

        let ghosts_grid: GridView = builder
            .object("ghosts_grid")
            .expect("Couldn't find 'ghosts_grid' in main_view.ui");

        let status: Label = builder
            .object("status")
            .expect("Couldn't find 'status' in main_view.ui");

        let states: Rc<RefCell<HashMap<String, GhostState>>> = Rc::new(RefCell::new(HashMap::new()));

        {
            let mut m = states.borrow_mut();
            for name in GHOSTS {
                m.insert(name.to_string(), GhostState::Default);
            }
        }

        let store: ListStore = ListStore::new::<StringObject>();

        for name in GHOSTS {
            let obj = StringObject::new(name);
            store.append(&obj);
        }

        let factory = SignalListItemFactory::new();

        let states_for_setup = Rc::clone(&states);
        let states_for_bind = Rc::clone(&states);

        factory.connect_setup(move |_, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();

            let label = Label::new(None);
            label.set_margin_top(8);
            label.set_margin_bottom(8);
            label.set_margin_start(8);
            label.set_margin_end(8);

            let container = gtk4::Box::new(Orientation::Vertical, 0);
            container.append(&label);
            container.add_css_class("ghost-item");

            let states_for_gesture = Rc::clone(&states_for_setup);
            let gesture = GestureClick::new();

            use gtk4::gdk;
            gesture.set_button(gdk::BUTTON_PRIMARY);

            gesture.connect_pressed(move |gesture, _n_press, _x, _y| {
                let widget = gesture.widget().unwrap();
                let container = widget.downcast::<gtk4::Box>().unwrap();
                let label = container
                    .first_child()
                    .unwrap()
                    .downcast::<Label>()
                    .unwrap();

                let name = label.text().to_string();

                let mut map = states_for_gesture.borrow_mut();
                let entry = map.entry(name.clone()).or_insert(GhostState::Default);

                *entry = match *entry {
                    GhostState::Default => GhostState::Off,
                    GhostState::Off => GhostState::On,
                    GhostState::On => GhostState::Default
                };

                container.remove_css_class("ghost-default");
                container.remove_css_class("ghost-on");
                container.remove_css_class("ghost-off");

                match *entry {
                    GhostState::Default => container.add_css_class("ghost-default"),
                    GhostState::On => container.add_css_class("ghost-on"),
                    GhostState::Off => container.add_css_class("ghost-off"),
                };
            });

            container.add_controller(gesture);
            list_item.set_child(Some(&container));
        });

        factory.connect_bind(move |_, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().unwrap();
            let container = list_item
                .child()
                .unwrap()
                .downcast::<gtk4::Box>()
                .unwrap();
            let label = container
                .first_child()
                .unwrap()
                .downcast::<Label>()
                .unwrap();

            let item = list_item
                .item()
                .unwrap()
                .downcast::<StringObject>()
                .unwrap();

            let name = item.string().to_string();
            label.set_label(&name);

            let map = states_for_bind.borrow();
            let state = map.get(&name).cloned().unwrap_or(GhostState::Default);

            container.remove_css_class("ghost-default");
            container.remove_css_class("ghost-on");
            container.remove_css_class("ghost-off");

            match state {
                GhostState::Default => container.add_css_class("ghost-default"),
                GhostState::On => container.add_css_class("ghost-on"),
                GhostState::Off => container.add_css_class("ghost-off"),
            }
        });

        let selection = NoSelection::new(Some(store.clone()));
        ghosts_grid.set_factory(Some(&factory));
        ghosts_grid.set_model(Some(&selection));

        ghosts_grid.set_min_columns(3);
        ghosts_grid.set_max_columns(3);



        let timer_running = Rc::new(AtomicBool::new(false));
        let now = Rc::new(RefCell::new(0u128));

        let timer_event_listener = Some(RefCell::new(register_event("timer_event", {
            let states = states.clone();
            let store = store.clone();
            let timer_running = Rc::clone(&timer_running);
            let now = Rc::clone(&now);
            let status = status.clone();
            move |event| {
                let event = event.as_any().downcast_ref::<TimerEvent>().unwrap();

                if timer_running.load(Ordering::Relaxed) {
                    let time = event.time - *now.borrow();
                    if time > 120_000 {
                        states.borrow_mut().insert(GHOSTS[9].to_string(), GhostState::Off);
                        store.items_changed(9 as u32, 1, 1);
                        store.splice(9 as u32, 1, &[StringObject::new(GHOSTS[9])]);
                    }

                    if time > 240_000 {
                        for (index, name) in GHOSTS.iter().enumerate() {
                            match index {
                                0 | 8 => {
                                    states.borrow_mut().insert(name.to_string(), GhostState::Default);
                                    store.items_changed(index as u32, 1, 1);
                                    store.splice(index as u32, 1, &[StringObject::new(name)]);
                                }
                                _ => {
                                    states.borrow_mut().insert(name.to_string(), GhostState::Off);
                                    store.items_changed(index as u32, 1, 1);
                                    store.splice(index as u32, 1, &[StringObject::new(name)]);
                                }
                            }
                        }
                    }

                    status.set_label(&format!("{} • {}", status.label().as_str().split(" • ").next().unwrap(), ms_to_hms(time)));
                }

                Continue
            }
        }, true)));
        resume_event("timer_event", timer_event_listener.as_ref().unwrap().borrow().clone());


        let button_event_listener = Some(RefCell::new(register_event("button_event", {
            let states = states.clone();
            let store = store.clone();
            move |event| {
                let event = event.as_any().downcast_ref::<ButtonEvent>().unwrap();

                match event.button {
                    Key::Space => {
                        if timer_running.load(Ordering::Relaxed) {
                            timer_running.store(false, Ordering::Relaxed);

                            let n = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis();

                            if n - *now.borrow() < 120_000 {
                                for (index, name) in GHOSTS.iter().enumerate() {
                                    match index {
                                        9 => {
                                            states.borrow_mut().insert(name.to_string(), GhostState::On);
                                            store.items_changed(index as u32, 1, 1);
                                            store.splice(index as u32, 1, &[StringObject::new(name)]);
                                        }
                                        _ => {
                                            states.borrow_mut().insert(name.to_string(), GhostState::Off);
                                            store.items_changed(index as u32, 1, 1);
                                            store.splice(index as u32, 1, &[StringObject::new(name)]);
                                        }
                                    }
                                }
                            }
                            status.set_label(&format!("{} • {}", status.label().as_str().split(" • ").next().unwrap(), "No Timer"));

                            return Continue;
                        }

                        *now.borrow_mut() = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                        timer_running.store(true, Ordering::Relaxed);
                        status.set_label(&format!("{} • {}", status.label().as_str().split(" • ").next().unwrap(), "No Timer"));
                    }
                    Key::BackQuote => {
                        for (index, name) in GHOSTS.iter().enumerate() {
                            states.borrow_mut().insert(name.to_string(), GhostState::Default);
                            store.items_changed(index as u32, 1, 1);
                            store.splice(index as u32, 1, &[StringObject::new(name)]);
                        }
                        status.set_label(&format!("{} • {}", "Clean", status.label().as_str().split(" • ").last().unwrap()));
                    }
                    Key::Num1 => {
                        for (index, name) in GHOSTS.iter().enumerate() {
                            match index {
                                2 | 7 | 11 | 13 | 15 | 19 | 21 | 22 | 23 => {
                                    states.borrow_mut().insert(name.to_string(), GhostState::Off);
                                    store.items_changed(index as u32, 1, 1);
                                    store.splice(index as u32, 1, &[StringObject::new(name)]);
                                }
                                _ => {
                                    states.borrow_mut().insert(name.to_string(), GhostState::Default);
                                    store.items_changed(index as u32, 1, 1);
                                    store.splice(index as u32, 1, &[StringObject::new(name)]);
                                }
                            }
                        }
                        status.set_label(&format!("{} • {}", "Normal", status.label().as_str().split(" • ").last().unwrap()));
                    }
                    Key::Num2 => {
                        for (index, name) in GHOSTS.iter().enumerate() {
                            match index {
                                7 | 17 | 21 | 23 => {
                                    states.borrow_mut().insert(name.to_string(), GhostState::Default);
                                    store.items_changed(index as u32, 1, 1);
                                    store.splice(index as u32, 1, &[StringObject::new(name)]);
                                }
                                _ => {
                                    states.borrow_mut().insert(name.to_string(), GhostState::Off);
                                    store.items_changed(index as u32, 1, 1);
                                    store.splice(index as u32, 1, &[StringObject::new(name)]);
                                }
                            }
                        }
                        status.set_label(&format!("{} • {}", "Fast", status.label().as_str().split(" • ").last().unwrap()));
                    }
                    Key::Num3 => {
                        for (index, name) in GHOSTS.iter().enumerate() {
                            match index {
                                5 | 13 | 22 => {
                                    states.borrow_mut().insert(name.to_string(), GhostState::Default);
                                    store.items_changed(index as u32, 1, 1);
                                    store.splice(index as u32, 1, &[StringObject::new(name)]);
                                }
                                _ => {
                                    states.borrow_mut().insert(name.to_string(), GhostState::Off);
                                    store.items_changed(index as u32, 1, 1);
                                    store.splice(index as u32, 1, &[StringObject::new(name)]);
                                }
                            }
                        }
                        status.set_label(&format!("{} • {}", "Slow", status.label().as_str().split(" • ").last().unwrap()));
                    }
                    Key::Num4 => {
                        for (index, name) in GHOSTS.iter().enumerate() {
                            match index {
                                7 | 13 | 21 | 23 => {
                                    states.borrow_mut().insert(name.to_string(), GhostState::Default);
                                    store.items_changed(index as u32, 1, 1);
                                    store.splice(index as u32, 1, &[StringObject::new(name)]);
                                }
                                _ => {
                                    states.borrow_mut().insert(name.to_string(), GhostState::Off);
                                    store.items_changed(index as u32, 1, 1);
                                    store.splice(index as u32, 1, &[StringObject::new(name)]);
                                }
                            }
                        }
                        status.set_label(&format!("{} • {}", "LOS", status.label().as_str().split(" • ").last().unwrap()));
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
