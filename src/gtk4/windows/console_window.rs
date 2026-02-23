use gtk4::{style_context_add_provider_for_display, Application, ApplicationWindow, Builder, CssProvider, Stack, StackPage, Window};
use gtk4::prelude::{GtkWindowExt, WidgetExt};
use crate::gtk4::views::console_view::ConsoleView;

#[derive(Clone)]
pub struct ConsoleWindow {
    pub window: Window
}

impl ConsoleWindow {

    pub fn new(app_window: &ApplicationWindow) -> Self {
        let window = Window::new();
        window.set_title(Some("PhasTimer"));
        window.set_default_size(1200, 700);

        let view = ConsoleView::new(app_window, &window);

        window.set_child(Some(&view.root));
        window.show();
        window.present();

        Self {
            window
        }
    }
}
