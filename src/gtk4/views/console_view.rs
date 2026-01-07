use glib::Propagation;
use gtk4::{gdk, style_context_add_provider_for_display, Builder, ComboBoxText, CssProvider, Paned, ScrolledWindow};
use gtk4::prelude::{RangeExt, ScaleExt};

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
            "150%",
        ];

        ghost_speed.set_format_value_func(|_, value| {
            let idx = value.round().clamp(0.0, 4.0) as usize;
            SPEED_LABELS[idx].to_string()
        });










        Self {
            root
        }
    }
}
