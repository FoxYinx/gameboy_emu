mod components;
mod utils;
mod io;

use std::time::Instant;
use components::gameboy::Gameboy;

fn main() {
    let start = Instant::now();
    
    let mut gameboy = Gameboy::new();
    //gameboy.toggle_debug_instructions();
    //gameboy.toggle_debug_registers();
    gameboy.cartridge_to_rom(String::from("resources/roms/cpu_instrs/individual/02-interrupts.gb"));
    gameboy.start();

    let end = Instant::now();
    println!("Time: {:?}", end.duration_since(start));
}