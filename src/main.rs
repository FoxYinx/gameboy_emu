mod components;
mod io;
mod utils;
mod window;

use crate::window::emulator_app::EmulatorApp;

fn main() {
    let options = eframe::NativeOptions::default();
    let result = eframe::run_native(
        "Gameboy Emulator",
        options,
        Box::new(|cc| Ok(Box::new(EmulatorApp::new(
            cc,
            String::from("resources/roms/blargg/cpu_instrs/individual/01-special.gb")
        ))),
        ));
    //gameboy.toggle_debug_registers();
}
