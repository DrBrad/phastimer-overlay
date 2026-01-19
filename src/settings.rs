use std::{fs, io};
use std::path::PathBuf;
use rdev::Key;
use crate::utils::keys::str_to_key;

pub static mut GHOST_SPEED: usize = 2;
pub static mut BLOOD_MOON: bool = false;
pub static mut KEY_TIMER_START: Key = Key::Num1;
pub static mut KEY_TIMER_RESET: Key = Key::Num2;
pub static mut KEY_OBAMBO_START: Key = Key::Num3;
pub static mut KEY_OBAMBO_RESET: Key = Key::Num4;
pub static mut KEY_MS: Key = Key::Num5;
pub static mut KEY_RESET: Key = Key::Num0;
pub static mut LOCATION: usize = 0;

pub unsafe fn load_settings() {
    let path = config_path();
    let Ok(text) = fs::read_to_string(path) else {
        return;
    };

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        let Some((k, v)) = line.split_once('=') else { continue; };
        let k = k.trim();
        let v = v.trim();

        match k {
            "key_timer_start" => KEY_TIMER_START = str_to_key(v),
            "key_timer_reset" => KEY_TIMER_RESET = str_to_key(v),
            "key_obambo_start" => KEY_OBAMBO_START = str_to_key(v),
            "key_obambo_reset" => KEY_OBAMBO_RESET = str_to_key(v),
            "key_ms" => KEY_MS = str_to_key(v),
            "key_reset" => KEY_RESET = str_to_key(v),
            "location" => LOCATION = v.parse().unwrap(),
            _ => {}
        }
    }
}

pub unsafe fn save_settings() -> io::Result<()> {
    let path = config_path();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }

    let contents = format!(
        "key_timer_start={:?}\n\
             key_timer_reset={:?}\n\
             key_obambo_start={:?}\n\
             key_obambo_reset={:?}\n\
             key_ms={:?}\n\
             key_reset={:?}\n\
             location={}",
        *&raw const KEY_TIMER_START,
        *&raw const KEY_TIMER_RESET,
        *&raw const KEY_OBAMBO_START,
        *&raw const KEY_OBAMBO_RESET,
        *&raw const KEY_MS,
        *&raw const KEY_RESET,
        *&raw const LOCATION
    );

    fs::write(path, contents)
}

fn config_path() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        return PathBuf::from(appdata).join("SmudgeTimer").join("config.ini");
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config").join("smudgetimer").join("config.ini");
    }

    PathBuf::from("config.ini")
}

pub unsafe fn verify_key_bind(key: &Key) -> bool {
    if key.eq(&Key::Unknown(0)) {
        return false;
    }

    if KEY_TIMER_START == *key {
        return false;
    }

    if KEY_TIMER_RESET == *key {
        return false;
    }

    if KEY_OBAMBO_START == *key {
        return false;
    }

    if KEY_OBAMBO_RESET == *key {
        return false;
    }

    if KEY_MS == *key {
        return false;
    }

    if KEY_RESET == *key {
        return false;
    }

    true
}
