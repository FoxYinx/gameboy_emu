use crate::components::registers::Registers;

pub struct Cpu {
    pub(crate) registers: Registers,
    cycles: u64,
    debug: bool
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: Registers::default(),
            cycles: 0,
            debug: false
        }
    }
    
    pub fn toggle_debug(&mut self) {
        self.debug = !self.debug
    }
    
    pub(crate) fn process_opcode(&mut self, opcode: u8, cartridge: &Vec<u8>) -> bool {
        match opcode { 
            0x00 => {
                if self.debug {
                    println!("Opcode: {:#04X} NOP, at PC {:#06X}", opcode, self.registers.pc);
                }
                
                self.cycles += 4;
                false
            }
            0xC3 => {
                if self.debug {
                    println!("Opcode: {:#04X} JP imm16, at PC {:#06X}", opcode, self.registers.pc);
                }

                self.cycles += 16;
                if let Some(low) = cartridge.get((self.registers.pc + 1) as usize) {
                    if let Some(high) = cartridge.get((self.registers.pc + 2) as usize) {
                        let address = ((*high as u16) << 8) | *low as u16;
                        self.registers.pc = address;
                        true
                    } else {
                        eprintln!("Failed to get high value of jump address at PC {:#06X}", self.registers.pc);
                        false
                    }
                } else { 
                    eprintln!("Failed to get low value of jump address at PC {:#06X}", self.registers.pc);
                    false
                }
            }
            _ => {
                eprintln!("Unimplemented opcode: {:#04X}, at PC {:#06X}", opcode, self.registers.pc);
                false
            }
        }
    }
}