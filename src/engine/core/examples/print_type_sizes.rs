use kime_engine_core::*;
use std::mem::size_of;

fn main() {
    println!("Engine: {}", size_of::<InputEngine>());
    println!("Config: {}", size_of::<Config>());
    println!("Hotkey: {}", size_of::<Hotkey>());
    println!("Option<Hotkey>: {}", size_of::<Option<Hotkey>>());
    println!("KeyMap<Hotkey>: {}", size_of::<KeyMap<Hotkey>>());
}
