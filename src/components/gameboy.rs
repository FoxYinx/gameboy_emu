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
        let cartridge_data = io::cartridge_reader::read_cartridge(filename);
        self.memory.write_cartridge(&cartridge_data);
    }

    pub fn toggle_debug(&mut self) {
        self.cpu.toggle_debug()
    }

    pub fn start(&mut self) {
        for _i in 0..100_000_000 {
            if let Some(opcode) = self.memory.get(self.cpu.registers.pc as usize) {
                let pc_modified = self.cpu.process_opcode(*opcode, &mut self.memory);
                self.cpu.update_ime();
                if !pc_modified {
                    self.cpu.registers.pc = self.cpu.registers.pc.wrapping_add(1);
                }
            } else {
                panic!("Tried to access address outside of ROM")
            }
        }
    }
}
