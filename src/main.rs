//#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod gtk4;
mod bus;
mod utils;

use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use rdev::{listen, EventType, Key};
use crate::bus::event_bus::send_event;
use crate::bus::events::button_event::ButtonEvent;
use crate::bus::events::inter::event::Event;
use crate::bus::events::timer_event::TimerEvent;
use crate::gtk4::app::App;
use crate::gtk4::windows::console_window::ConsoleWindow;
//export GTK_DEBUG=interactive

//glib-compile-resources res/gtk4/linux.gresources.xml --target=res/resources.gresources

/*
rustup install nightly
rustup override set nightly
*/

//NOTES
//NORMAL BLINKS - NOT (Oni, Phantom, Deogen, Obake)
//FAST - (Twin, Moroi, Revenant, Thaye)
//SLOW - (Jinn, Deogen)
//LOS - NOT (Deogen, Hantu, Revenant, and Thaye)
//


/*
cargo build --release
powershell -ExecutionPolicy Bypass -File tools\package.ps1

$env:PATH="C:\Windows\System32;C:\Windows"
Start-Process -FilePath .\target\release\smudge-timer.exe
*/



fn main() {
    thread::spawn(|| {
        if let Err(err) = listen(|event| {
            match event.event_type {
                EventType::KeyPress(_) => {}
                EventType::KeyRelease(key) => {
                    match key {
                        _ => send_event(Box::new(ButtonEvent::new(key)))
                    }
                }
                EventType::ButtonPress(_) => {}
                EventType::ButtonRelease(_) => {}
                EventType::MouseMove { .. } => {}
                EventType::Wheel { .. } => {}
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
