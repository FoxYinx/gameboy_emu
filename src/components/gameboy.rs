use crate::components::cpu::Cpu;
use crate::io;

pub struct Gameboy {
    cpu: Cpu,
    cartridge: Vec<u8>
}

impl Gameboy {
    pub fn new() -> Self {
        Gameboy {
            cpu: Cpu::new(),
            cartridge: Vec::new()
        }
    }

    pub fn cartridge_to_rom(&mut self, filename: String) {
        self.cartridge = io::cartridge_reader::read_cartridge(filename)
    }

    pub fn toggle_debug(&mut self) {
        self.cpu.toggle_debug()
    }

    pub fn start(&mut self) {
        for _i in 0..1000 {
            if let Some(opcode) = self.cartridge.get(self.cpu.registers.pc as usize) {
                let pc_modified = self.cpu.process_opcode(*opcode, &mut self.cartridge);
                if !pc_modified {
                    self.cpu.registers.pc += 1;
                }
            } else {
                panic!("Tried to access address outside of ROM")
            }
        }
    }
}
