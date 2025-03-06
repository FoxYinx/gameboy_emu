mod components;
mod utils;
mod io;

use crate::components::cpu::Cpu;
use std::time::Instant;

fn main() {
    let start = Instant::now();
    
    let mut gameboy_cpu = Cpu::new();
    gameboy_cpu.toggle_debug();
    gameboy_cpu.cartridge_to_rom(String::from("resources/roms/cpu_instrs/individual/01-special.gb"));
    gameboy_cpu.start();

    let end = Instant::now();
    println!("Time: {:?}", end.duration_since(start));
}