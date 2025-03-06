mod components;
mod utils;
mod io;

use io::cartridge_reader::read_cartridge;
use crate::components::cpu::Cpu;

fn main() {
    let gameboy_cpu = Cpu::new();
    let bytes = read_cartridge(String::from("resources/roms/cpu_instrs/individual/01-special.gb"));
    for (line, byte) in bytes.iter().enumerate() {
        if *byte != 0 {
            println!("{} at line {}", byte, line);
        }
    }
}