mod components;
mod io;
mod utils;

use components::gameboy::Gameboy;

fn main() {
    let mut gameboy = Gameboy::new();
    //gameboy.toggle_debug_instructions();
    //gameboy.toggle_debug_registers();
    gameboy.cartridge_to_rom(String::from(
        "resources/roms/mem_timing/individual/01-read_timing.gb",
    ));
    gameboy.start(None);
}
