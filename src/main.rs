mod components;
mod io;
mod utils;

use components::gameboy::Gameboy;
use std::time::Instant;

fn main() {
    let start = Instant::now();

    let mut gameboy = Gameboy::new();
    //gameboy.toggle_debug_instructions();
    //gameboy.toggle_debug_registers();
    gameboy.cartridge_to_rom(String::from(
        "resources/roms/cpu_instrs/individual/11-op a,(hl).gb",
    ));
    gameboy.start(None);

    let end = Instant::now();
    println!("Time: {:?}", end.duration_since(start));
}
