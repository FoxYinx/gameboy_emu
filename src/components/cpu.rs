use crate::components::registers::Registers;

pub struct Cpu {
    pub(crate) registers: Registers,
    cycles: u64,
    debug: bool,
    ime: bool,
    ime_scheduled: bool
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: Registers::default(),
            cycles: 0,
            debug: false,
            ime: true,
            ime_scheduled: false
        }
    }
    
    pub fn toggle_debug(&mut self) {
        self.debug = !self.debug
    }

    pub(crate) fn update_ime(&mut self) {
        if self.ime_scheduled {
            self.ime = !self.ime;
            self.ime_scheduled = false;
        }
    }
    
    pub(crate) fn process_opcode(&mut self, opcode: u8, memory: &mut [u8; 0x10000]) -> bool {
        match opcode { 
            0x00 => {
                if self.debug {
                    println!("Opcode: {:#04X} NOP, at PC {:#06X}", opcode, self.registers.pc);
                }
                
                self.cycles += 4;
                false
            }
            0x0D => {
                self.cycles += 4;
                let original = self.registers.c;
                self.registers.c = self.registers.c.wrapping_sub(1);

                if self.debug {
                    println!("Opcode: {:#04X} DEC C, C now is {:#04X}, at PC {:#06X}", opcode, self.registers.d, self.registers.pc);
                }

                self.registers.set_z(self.registers.c == 0);
                self.registers.set_n(true);
                self.registers.set_h((original & 0x0F) == 0x00);
                false
            }
            0x0E => {
                self.cycles += 8;
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(imm8) = memory.get(self.registers.pc as usize) {
                    if self.debug {
                        println!("Opcode: {:#04X} LD C imm8, with imm8 = {:#04X}, at PC {:#06X}", opcode, imm8, self.registers.pc.wrapping_sub(1));
                    }
                    
                    self.registers.c = *imm8;
                } else {
                    eprintln!("Failed to get imm8 at PC {:#06X}", self.registers.pc);
                }
                false
            }
            0x11 => {
                self.cycles += 12;
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(low) = memory.get(self.registers.pc as usize) {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.pc as usize) {
                        let immediate = ((*high as u16) << 8) | *low as u16;
                        
                        if self.debug {
                            println!("Opcode: {:#04X} LD DE imm16, with imm16 = {:#06X}, at PC {:#06X}", opcode, immediate, self.registers.pc.wrapping_sub(2));
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
            0x12 => {
                if self.debug {
                    println!("Opcode: {:#04X} LD [DE] A, with DE = {:#06X} & A = {:#04X}, at PC {:#06X}", opcode, self.registers.get_de(), self.registers.a, self.registers.pc);
                }

                self.cycles += 8;
                memory[self.registers.get_de() as usize] = self.registers.a;
                false
            }
            0x14 => {
                self.cycles += 4;
                let original = self.registers.d;
                self.registers.d = self.registers.d.wrapping_add(1);

                if self.debug {
                    println!("Opcode: {:#04X} INC D, D now is {:#04X}, at PC {:#06X}", opcode, self.registers.d, self.registers.pc);
                }

                self.registers.set_z(self.registers.d == 0);
                self.registers.set_n(false);
                self.registers.set_h((original & 0x0F) == 0x0F);
                false
            }
            0x1C => {
                self.cycles += 4;
                let original = self.registers.e;
                self.registers.e = self.registers.e.wrapping_add(1);

                if self.debug {
                    println!("Opcode: {:#04X} INC E, E now is {:#04X}, at PC {:#06X}", opcode, self.registers.e, self.registers.pc);
                }

                self.registers.set_z(self.registers.e == 0);
                self.registers.set_n(false);
                self.registers.set_h((original & 0x0F) == 0x0F);
                false
            }
            0x20 => {
                self.cycles += 8;
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if !self.registers.get_z() {
                    if let Some(offset) = memory.get(self.registers.pc as usize) {
                        let original_pc = self.registers.pc;
                        self.registers.pc = self.registers.pc.wrapping_add_signed(*offset as i8 as i16);
                        self.cycles += 4;
                        
                        if self.debug {
                            println!("Opcode: {:#04X} JP NZ e8, with e8 = {:#04X}, at PC {:#06X}", opcode, *offset, original_pc);
                        }
                    } else {
                        eprintln!("Failed to get offset for jump at PC {:#06X}", self.registers.pc);
                    }
                } else if self.debug {
                    println!("Opcode: {:#04X} JP NZ but Z is true, at PC {:#06X}", opcode, self.registers.pc.wrapping_sub(1));
                }
                false
            }
            0x21 => {
                self.cycles += 12;
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(low) = memory.get(self.registers.pc as usize) {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.pc as usize) {
                        let immediate = ((*high as u16) << 8) | *low as u16;

                        if self.debug {
                            println!("Opcode: {:#04X} LD HL imm16, with imm16 = {:#06X}, at PC {:#06X}", opcode, immediate, self.registers.pc.wrapping_sub(2));
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
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
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
                if self.debug {
                    println!("Opcode: {:#04X} LD B A, with A = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }
                
                self.cycles += 4;
                self.registers.b = self.registers.a;
                false
            }
            0x78 => {
                if self.debug {
                    println!("Opcode: {:#04X} LD A B, with B = {:#04X}, at PC {:#06X}", opcode, self.registers.b, self.registers.pc);
                }
                
                self.cycles += 4;
                self.registers.a = self.registers.b;
                false
            }
            0xC3 => {
                self.cycles += 16;
                if let Some(low) = memory.get((self.registers.pc + 1) as usize) {
                    if let Some(high) = memory.get((self.registers.pc + 2) as usize) {
                        let address = ((*high as u16) << 8) | *low as u16;

                        if self.debug {
                            println!("Opcode: {:#04X} JP a16, with a16 = {:#06X}, at PC {:#06X}", opcode, address, self.registers.pc);
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
            0xF3 => {
                if self.debug {
                    println!("Opcode: {:#04X} DI, at PC {:#06X}", opcode, self.registers.pc);
                }
                
                self.ime_scheduled = true;
                self.cycles += 4;
                false
            }
            _ => {
                panic!("Unimplemented opcode: {:#04X}, at PC {:#06X}", opcode, self.registers.pc)
            }
        }
    }
}