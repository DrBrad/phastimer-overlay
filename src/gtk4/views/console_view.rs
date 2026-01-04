use gtk4::{gdk, style_context_add_provider_for_display, Builder, ComboBoxText, CssProvider, Paned, ScrolledWindow};
use gtk4::gdk::RGBA;
use gtk4::prelude::{BoxExt, ComboBoxExt, ComboBoxExtManual, WidgetExt};

pub struct ConsoleView {
    pub root: Paned
}

impl ConsoleView {

    pub fn new() -> Self {
        let builder = Builder::from_resource("/smudgetimer/rust/res/ui/console_view.ui");

        let provider = CssProvider::new();
        provider.load_from_resource("/smudgetimer/rust/res/ui/console_view.css");
        style_context_add_provider_for_display(&gdk::Display::default().unwrap(), &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let root: Paned = builder
            .object("root")
            .expect("Couldn't find 'root' in console_view.ui");

        Self {
            root
        }
    }
}
