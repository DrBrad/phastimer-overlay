use std::cell::Cell;
use std::rc::Rc;
use glib::Propagation;
use gtk4::{gdk, style_context_add_provider_for_display, Builder, Button, ComboBoxText, CssProvider, Paned, ScrolledWindow, Switch, Window};
use gtk4::prelude::{ButtonExt, RangeExt, ScaleExt, WidgetExt};
use crate::settings::{save_settings, verify_key_bind, BLOOD_MOON, GHOST_SPEED, KEY_MS, KEY_OBAMBO_RESET, KEY_OBAMBO_START, KEY_RESET, KEY_TIMER_RESET, KEY_TIMER_START};
use crate::utils::keys::gtk4_key_to_key;

pub struct ConsoleView {
    pub root: gtk4::Box
}

impl ConsoleView {

    pub fn new(window: &Window) -> Self {
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
        unsafe { ghost_speed.set_value(GHOST_SPEED as f64); }

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


        let blood_moon_swc: Switch = builder
            .object("blood_moon_swc")
            .expect("Couldn't find 'blood_moon_swc' in console_view.ui");
        unsafe { blood_moon_swc.set_active(BLOOD_MOON); }

        blood_moon_swc.connect_state_set(|_sw, state| {
            unsafe { BLOOD_MOON = state; }
            Propagation::Proceed
        });


        let timer_start_btn: Button = builder
            .object("timer_start_btn")
            .expect("Couldn't find 'timer_start_btn' in console_view.ui");
        let timer_reset_btn: Button = builder
            .object("timer_reset_btn")
            .expect("Couldn't find 'timer_reset_btn' in console_view.ui");
        let obambo_start_btn: Button = builder
            .object("obambo_start_btn")
            .expect("Couldn't find 'obambo_start_btn' in console_view.ui");
        let obambo_reset_btn: Button = builder
            .object("obambo_reset_btn")
            .expect("Couldn't find 'obambo_reset_btn' in console_view.ui");
        let ms_btn: Button = builder
            .object("ms_btn")
            .expect("Couldn't find 'ms_btn' in console_view.ui");
        let complete_reset_btn: Button = builder
            .object("complete_reset_btn")
            .expect("Couldn't find 'complete_reset_btn' in console_view.ui");

        unsafe {
            timer_start_btn.set_label(&format!("{:?}", *&raw const KEY_TIMER_START));
            timer_reset_btn.set_label(&format!("{:?}", *&raw const KEY_TIMER_RESET));
            obambo_start_btn.set_label(&format!("{:?}", *&raw const KEY_OBAMBO_START));
            obambo_reset_btn.set_label(&format!("{:?}", *&raw const KEY_OBAMBO_RESET));
            ms_btn.set_label(&format!("{:?}", *&raw const KEY_MS));
            complete_reset_btn.set_label(&format!("{:?}", *&raw const KEY_RESET));
        }

        let capture_next = Rc::new(Cell::new(false));
        let capture_target = Rc::new(Cell::new(0));

        timer_start_btn.connect_clicked({
            let capture_next = capture_next.clone();
            let mut capture_target = capture_target.clone();
            move |_| {
                capture_target.set(0);
                capture_next.set(true);
            }
        });

        timer_reset_btn.connect_clicked({
            let capture_next = capture_next.clone();
            let capture_target = capture_target.clone();
            move |_| {
                capture_target.set(1);
                capture_next.set(true);
            }
        });

        obambo_start_btn.connect_clicked({
            let capture_next = capture_next.clone();
            let capture_target = capture_target.clone();
            move |_| {
                capture_target.set(2);
                capture_next.set(true);
            }
        });

        obambo_reset_btn.connect_clicked({
            let capture_next = capture_next.clone();
            let capture_target = capture_target.clone();
            move |_| {
                capture_target.set(3);
                capture_next.set(true);
            }
        });

        ms_btn.connect_clicked({
            let capture_next = capture_next.clone();
            let capture_target = capture_target.clone();
            move |_| {
                capture_target.set(4);
                capture_next.set(true);
            }
        });

        complete_reset_btn.connect_clicked({
            let capture_next = capture_next.clone();
            let capture_target = capture_target.clone();
            move |_| {
                capture_target.set(5);
                capture_next.set(true);
            }
        });

        let controller = gtk4::EventControllerKey::new();

        {
            let capture_next = capture_next.clone();
            let capture_target = capture_target.clone();

            controller.connect_key_pressed(move |_, key, _keycode, _state| unsafe {
                if !capture_next.get() {
                    return Propagation::Proceed;
                }

                capture_next.set(false);

                let key = gtk4_key_to_key(&key);

                if verify_key_bind(&key) {
                    match capture_target.get() {
                        0 => {
                            KEY_TIMER_START = key;
                            timer_start_btn.set_label(&format!("{:?}", key));
                        }
                        1 => {
                            KEY_TIMER_RESET = key;
                            timer_reset_btn.set_label(&format!("{:?}", key));
                        }
                        2 => {
                            KEY_OBAMBO_START = key;
                            obambo_start_btn.set_label(&format!("{:?}", key));
                        }
                        3 => {
                            KEY_OBAMBO_RESET = key;
                            obambo_start_btn.set_label(&format!("{:?}", key));
                        }
                        4 => {
                            KEY_MS = key;
                            ms_btn.set_label(&format!("{:?}", key));
                        }
                        5 => {
                            KEY_RESET = key;
                            complete_reset_btn.set_label(&format!("{:?}", key));
                        }
                        _ => {}
                    }
                    println!("{:?}", save_settings());
                }

                Propagation::Stop
            });
        }

        window.add_controller(controller);

        Self {
            root
        }
    }
}
