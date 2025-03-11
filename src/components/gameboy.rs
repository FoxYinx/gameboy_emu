use crate::components::cpu::Cpu;
use crate::components::memory::Memory;
use crate::io;

pub struct Gameboy {
    cpu: Cpu,
    memory: Memory
}

impl Gameboy {
    pub fn new() -> Self {
        Gameboy {
            cpu: Cpu::new(),
            memory: Memory::new()
        }
    }

    pub fn cartridge_to_rom(&mut self, filename: String) {
        println!("Loading ROM: {filename}");
        let cartridge_data = io::cartridge_reader::read_cartridge(filename);
        self.memory.write_cartridge(&cartridge_data);
        if let Some(header_checksum) = self.memory.get(0x014D) {
            if *header_checksum != 0x00 {
                self.cpu.registers.set_h(true);
                self.cpu.registers.set_c(true);
            }
        } else {
            eprintln!("Unable to access header checksum at 0x014D");
        }
    }

    pub fn toggle_debug_instructions(&mut self) {
        self.cpu.toggle_debug_instructions();
    }

    pub fn toggle_debug_registers(&mut self) {
        self.cpu.toggle_debug_registers();
    }

    pub fn start(&mut self, test: Option<u64>) {
        if let Some(iterations) = test {
            for _i in 0..iterations {
                if let Some(opcode) = self.memory.get(self.cpu.registers.pc as usize) {
                    let jumped = self.cpu.process_opcode(*opcode, &mut self.memory);
                    self.cpu.update_ime();
                    if !jumped {
                        self.cpu.registers.pc = self.cpu.registers.pc.wrapping_add(1);
                    }
                } else {
                    panic!("Tried to access address outside of ROM")
                }
            }
        } else {
            loop {
                if let Some(opcode) = self.memory.get(self.cpu.registers.pc as usize) {
                    let jumped = self.cpu.process_opcode(*opcode, &mut self.memory);
                    self.cpu.update_ime();
                    if !jumped {
                        self.cpu.registers.pc = self.cpu.registers.pc.wrapping_add(1);
                    }
                } else {
                    panic!("Tried to access address outside of ROM")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rom_01_special() {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(String::from("resources/roms/cpu_instrs/individual/01-special.gb"));
        gameboy.start(Some(2_000_000));
        let output = gameboy.memory.get_serial_output().get_output();
        assert!(output.contains("Passed"), "Test failed. Output: {}", output);
    }
}