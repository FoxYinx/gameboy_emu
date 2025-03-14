mod components;
mod io;
mod utils;

use components::gameboy::Gameboy;

fn main() {
    let mut gameboy = Gameboy::new();
    //gameboy.toggle_debug_instructions();
    //gameboy.toggle_debug_registers();
    gameboy.cartridge_to_rom(String::from(
        "resources/roms/cpu_instrs/cpu_instrs.gb",
    ));
    gameboy.start(None);
}
