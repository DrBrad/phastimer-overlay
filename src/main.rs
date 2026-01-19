#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod gtk4;
mod bus;
mod utils;
mod settings;

use std::process::exit;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rdev::{listen, EventType, Key};
use crate::bus::event_bus::send_event;
use crate::bus::events::button_event::ButtonEvent;
use crate::bus::events::inter::event::Event;
use crate::bus::events::timer_event::TimerEvent;
use crate::gtk4::app::App;
use crate::settings::load_settings;

//export GTK_DEBUG=interactive
//$env:GTK_DEBUG="interactive"

//glib-compile-resources res/gtk4/windows.gresources.xml --target=res/resources.gresources

/*
rustup install nightly
rustup override set nightly
*/

/*
cargo build --release
powershell -ExecutionPolicy Bypass -File tools\package.ps1

$env:PATH="C:\Windows\System32;C:\Windows"
Start-Process -FilePath .\target\release\smudge-timer.exe
*/

fn main() {
    unsafe { load_settings(); }

    thread::spawn(|| {
        if let Err(err) = listen(|event| {
            match event.event_type {
                EventType::KeyRelease(key) => {
                    if key == Key::BackSlash {
                        exit(0);
                    }
                    send_event(Box::new(ButtonEvent::new(key)))
                }
                _ => {}
            }
        }) {
            eprintln!("Error: {:?}", err);
        }
    });

    thread::spawn(|| {
        loop {
            thread::sleep(Duration::from_millis(10));
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            send_event(Box::new(TimerEvent::new(now)));
        }
    });

    let app = App::new();
    app.run();
}
