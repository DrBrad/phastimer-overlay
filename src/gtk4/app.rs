use gtk4::{gdk, gio, style_context_add_provider_for_display, Application, Builder, CssProvider, Settings, StyleContext};
use gtk4::gio::{resources_register, ApplicationFlags, Resource};
use gtk4::glib::Bytes;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, FileExt, GtkApplicationExt, ObjectExt, StaticType, StyleContextExt};
use crate::gtk4::windows::main_window::MainWindow;

pub struct App {
    app: Application
}

impl App {

    pub fn new() -> Self {
        let app = Application::new(Some("smudgetimer.rust"), ApplicationFlags::HANDLES_OPEN);

        Self {
            app
        }
    }

    pub fn run(&self) {
        self.app.connect_activate(move |app| {
            if let Some(settings) = Settings::default() {
                settings.set_property("gtk-application-prefer-dark-theme", &true);
            }

            let resource_data = include_bytes!("../../res/resources.gresources");

            let resource = Resource::from_data(&Bytes::from(resource_data)).unwrap();
            resources_register(&resource);


            let provider = CssProvider::new();
            provider.load_from_resource("/smudgetimer/rust/res/ui/theme.css");

            style_context_add_provider_for_display(&gdk::Display::default().unwrap(), &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

            //provider.load_from_resource("/smudgetimer/rust/res/ui/theme.css");

            MainWindow::new(&app);

            //register_app_actions(&app);
        });

        self.app.run();
    }
}
