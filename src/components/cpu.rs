use crate::components::registers::Registers;

pub struct Cpu {
    pub(crate) registers: Registers,
    debug: bool
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: Registers::default(),
            debug: false
        }
    }
    
    pub fn toggle_debug(&mut self) {
        self.debug = !self.debug
    }
    
    pub(crate) fn process_opcode(&mut self, opcode: u8) -> bool {
        match opcode { 
            0x00 => {
                if self.debug {
                    println!("Opcode: {:#04X} NOP, at PC {:#06X}", opcode, self.registers.pc);
                }
                false
            }
            0xC3 => {
                if self.debug {
                    println!("Opcode: {:#04X} JP imm16, at PC {:#06X}", opcode, self.registers.pc);
                }
                self.registers.pc = self.
                true
            }
            _ => {
                eprintln!("Unimplemented opcode: {:#04X}, at PC {:#06X}", opcode, self.registers.pc);
                false
            }
        }
    }
}