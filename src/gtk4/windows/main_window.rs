use std::cell::RefCell;
use std::collections::HashMap;
use std::process::exit;
use std::rc::Rc;
use gtk4::{gdk, style_context_add_provider_for_display, Application, ApplicationWindow, Builder, CssProvider, Stack, StackPage};
use gtk4::prelude::{Cast, GtkWindowExt, ListModelExt, WidgetExt};
use crate::gtk4::views::inter::stackable::Stackable;
use crate::gtk4::views::main_view::MainView;

#[derive(Clone)]
pub struct MainWindow {
    pub window: ApplicationWindow,
    pub stack: Stack,
    pub views: Rc<RefCell<HashMap<String, Box<dyn Stackable>>>>
}

impl MainWindow {

    pub fn new(app: &Application) -> Self {
        let builder = Builder::from_resource("/smudgetimer/rust/res/ui/window.ui");

        let provider = CssProvider::new();
        provider.load_from_resource("/smudgetimer/rust/res/ui/window.css");

        style_context_add_provider_for_display(&gdk::Display::default().unwrap(), &provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

        let window: ApplicationWindow = builder
            .object("main_window")
            .expect("Failed to get the 'main_window' from window.ui");

        window.set_application(Some(app));
        window.connect_destroy(|_| exit(0));
        window.set_decorated(false);
        //window.set_border_width(1);


        let root: gtk4::Box = builder
            .object("root")
            .expect("Failed to get the 'root' from window.ui");

        //window_content.add(&create_alertbar());

        let stack: Stack = builder
            .object("stack")
            .expect("Failed to get the 'stack' from window.ui");

        let views: Rc<RefCell<HashMap<String, Box<dyn Stackable>>>> = Rc::new(RefCell::new(HashMap::new()));

        stack.connect_visible_child_name_notify({
            let views = views.clone();
            let mut previous = RefCell::new(String::new());
            move |stack| {
                let current = stack.visible_child_name().unwrap_or_default().to_string();

                if previous.borrow().is_empty() {
                    *previous.borrow_mut() = current;
                    return;
                }

                views.borrow().get(&*previous.borrow()).unwrap().on_pause();

                if views.borrow().contains_key(&current) {
                    views.borrow().get(&current).unwrap().on_resume();
                }

                *previous.borrow_mut() = current;
            }
        });


        window.show();

        let _self = Self {
            window,
            stack,
            views
        };

        _self.add_view(Box::new(MainView::new(&_self)));

        _self
    }

    pub fn add_view(&self, view: Box<dyn Stackable>) {
        let name = view.get_name();

        match self.stack.child_by_name(&name) {
            Some(child) => {
                //self.title_bar.back.style_context().add_class("active");
                //self.title_bar.next.style_context().remove_class("active");

                let pages = self.stack.pages();
                for i in (0..pages.n_items()).rev() {
                    let page = pages.item(i).expect("Failed to get page")
                        .downcast::<StackPage>()
                        .expect("Item is not a StackPage");

                    let eq = child.eq(&page.child());
                    let name = page.name().unwrap().to_string();
                    self.views.borrow().get(&name).unwrap().on_destroy();
                    self.stack.remove(&page.child());
                    self.views.borrow_mut().remove(&name);

                    if eq {
                        break;
                    }
                }
            }
            None => {
                if let Some(current) = self.stack.visible_child() {
                    let pages = self.stack.pages();
                    for i in (0..pages.n_items()).rev() {
                        let page = pages.item(i).expect("Failed to get page")
                            .downcast::<StackPage>()
                            .expect("Item is not a StackPage");

                        if current.eq(&page.child()) {
                            //self.title_bar.back.style_context().add_class("active");
                            //self.title_bar.next.style_context().remove_class("active");
                            break;
                        }

                        let name = page.name().unwrap().to_string();
                        self.views.borrow().get(&name).unwrap().on_destroy();
                        self.stack.remove(&page.child());
                        self.views.borrow_mut().remove(&name);
                    }
                }
            }
        }

        self.stack.add_named(view.get_root(), Some(&name));
        self.stack.set_visible_child_name(&name);
        view.on_create();
        self.views.borrow_mut().insert(name, view);
    }
}
