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
            0x0E => {
                self.cycles += 8;
                self.registers.pc += 1;
                if let Some(imm8) = cartridge.get(self.registers.pc as usize) {
                    if self.debug {
                        println!("Opcode: {:#04X} LD C imm8, with imm8 = {:#04X}, at PC {:#06X}", opcode, imm8, self.registers.pc - 1);
                    }
                    
                    self.registers.c = *imm8;
                } else {
                    eprintln!("Failed to get imm8 at PC {:#06X}", self.registers.pc);
                }
                false
            }
            0x11 => {
                self.cycles += 12;
                self.registers.pc += 1;
                if let Some(low) = cartridge.get(self.registers.pc as usize) {
                    self.registers.pc += 1;
                    if let Some(high) = cartridge.get(self.registers.pc as usize) {
                        let immediate = ((*high as u16) << 8) | *low as u16;
                        
                        if self.debug {
                            println!("Opcode: {:#04X} LD DE imm16, with imm16 = {:#06X}, at PC {:#06X}", opcode, immediate, self.registers.pc - 2);
                        }
                        
                        self.registers.set_de(immediate);
                    } else {
                        eprintln!("Failed to get high value of immediate at PC {:#06X}", self.registers.pc);
                    }
                } else {
                    eprintln!("Failed to get low value of jump immediate at PC {:#06X}", self.registers.pc);
                }
                false
            }
            0x21 => {
                self.cycles += 12;
                self.registers.pc += 1;
                if let Some(low) = cartridge.get(self.registers.pc as usize) {
                    self.registers.pc += 1;
                    if let Some(high) = cartridge.get(self.registers.pc as usize) {
                        let immediate = ((*high as u16) << 8) | *low as u16;

                        if self.debug {
                            println!("Opcode: {:#04X} LD HL imm16, with imm16 = {:#06X}, at PC {:#06X}", opcode, immediate, self.registers.pc - 2);
                        }
                        
                        self.registers.set_hl(immediate);
                    } else {
                        eprintln!("Failed to get high value of immediate at PC {:#06X}", self.registers.pc);
                    }
                } else {
                    eprintln!("Failed to get low value of jump immediate at PC {:#06X}", self.registers.pc);
                }
                false
            }
            0x2A => {
                self.cycles += 8;
                if let Some(value) = cartridge.get(self.registers.get_hl() as usize) {
                    if self.debug {
                        println!("Opcode: {:#04X} LD A [HL+], with [HL] = {:#04X}, at PC {:#06X}", opcode, value, self.registers.pc);
                    }
                    
                    self.registers.a = *value;
                    self.registers.set_hl(self.registers.get_hl().wrapping_add(1));
                } else {
                    eprintln!("Failed to get value at [HL] {:#06X}", self.registers.get_hl());
                }
                false
            }
            0x47 => {
                self.cycles += 4;
                
                if self.debug {
                    println!("Opcode: {:#04X} LD B A, with A = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }
                
                self.registers.b = self.registers.a;
                false
            }
            0xC3 => {
                self.cycles += 16;
                if let Some(low) = cartridge.get((self.registers.pc + 1) as usize) {
                    if let Some(high) = cartridge.get((self.registers.pc + 2) as usize) {
                        let address = ((*high as u16) << 8) | *low as u16;

                        if self.debug {
                            println!("Opcode: {:#04X} JP a16, with a16 = {:#06X}, at PC {:#06X}", opcode, address, self.registers.pc - 2);
                        }

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
                panic!("Unimplemented opcode: {:#04X}, at PC {:#06X}", opcode, self.registers.pc)
            }
        }
    }
}