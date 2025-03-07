use crate::components::cpu::Cpu;
use crate::io;

pub struct Gameboy {
    cpu: Cpu,
    memory: [u8; 0x10000]
}

impl Gameboy {
    pub fn new() -> Self {
        Gameboy {
            cpu: Cpu::new(),
            memory: [0; 0x10000]
        }
    }

    pub fn cartridge_to_rom(&mut self, filename: String) {
        let cartridge_data = io::cartridge_reader::read_cartridge(filename);
        self.memory[0x0000..cartridge_data.len()].copy_from_slice(&cartridge_data);
    }

    pub fn toggle_debug(&mut self) {
        self.cpu.toggle_debug()
    }

    pub fn start(&mut self) {
        for _i in 0..20000 {
            if let Some(opcode) = self.memory.get(self.cpu.registers.pc as usize) {
                let pc_modified = self.cpu.process_opcode(*opcode, &mut self.memory);
                if !pc_modified {
                    self.cpu.registers.pc = self.cpu.registers.pc.wrapping_add(1);
                }
            } else {
                panic!("Tried to access address outside of ROM")
            }
        }
    }
}
