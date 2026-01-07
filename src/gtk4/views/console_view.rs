use std::process::exit;
use std::thread;
use glib::Propagation;
use gtk4::{gdk, style_context_add_provider_for_display, Builder, Button, ComboBoxText, CssProvider, Paned, ScrolledWindow};
use gtk4::prelude::{ButtonExt, RangeExt, ScaleExt};
use rdev::{listen, EventType, Key};
use crate::bus::event_bus::{register_event, send_event, unregister_event};
use crate::bus::event_bus::EventPropagation::Continue;
use crate::bus::events::button_event::ButtonEvent;
use crate::settings::GHOST_SPEED;

pub struct ConsoleView {
    pub root: gtk4::Box
}

impl ConsoleView {

    pub fn new() -> Self {
        let builder = Builder::from_resource("/smudgetimer/rust/res/ui/console_view.ui");

        let provider = CssProvider::new();
        provider.load_from_resource("/smudgetimer/rust/res/ui/console_view.css");
        style_context_add_provider_for_display(&gdk::Display::default().unwrap(), &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let root: gtk4::Box = builder
            .object("root")
            .expect("Couldn't find 'root' in console_view.ui");




        let ghost_speed: gtk4::Scale = builder
            .object("ghost_speed")
            .expect("Couldn't find 'ghost_speed' in console_view.ui");

        const SPEED_LABELS: [&str; 5] = [
            "50%",
            "75%",
            "100%",
            "125%",
            "150%"
        ];

        ghost_speed.set_format_value_func(|_, value| {
            let idx = value.round().clamp(0.0, 4.0) as usize;
            unsafe { GHOST_SPEED = idx; }
            SPEED_LABELS[idx].to_string()
        });


        let complete_reset_btn: Button = builder
            .object("complete_reset_btn")
            .expect("Couldn't find 'complete_reset_btn' in console_view.ui");

        complete_reset_btn.connect_clicked(move |_| {
            /*
            register_event("button_event", |id, event| {
                let event = event.as_any().downcast_ref::<ButtonEvent>().unwrap();

                //unregister_event("button_event", id);
                println!("Button event: {:?}", event.button);
                Continue
            }, false);*/
        });







        Self {
            root
        }
    }
}
