use crate::components::memory::Memory;
use crate::components::registers::Registers;

pub struct Cpu {
    pub(crate) registers: Registers,
    debug_registers: bool,
    cycles: u64,
    debug_instructions: bool,
    ime: bool,
    ime_scheduled: bool
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: Registers::default(),
            debug_registers: false,
            cycles: 0,
            debug_instructions: false,
            ime: true,
            ime_scheduled: false
        }
    }
    
    pub fn toggle_debug_instructions(&mut self) {
        self.debug_instructions = !self.debug_instructions
    }

    pub fn toggle_debug_registers(&mut self) {
        self.debug_registers = !self.debug_registers
    }

    pub(crate) fn update_ime(&mut self) {
        if self.ime_scheduled {
            self.ime = !self.ime;
            self.ime_scheduled = false;
        }
    }
    
    pub(crate) fn process_opcode(&mut self, opcode: u8, memory: &mut Memory) -> bool {
        if self.debug_registers {
            println!("A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X})", self.registers.a, self.registers.f, self.registers.b, self.registers.c, self.registers.d, self.registers.e, self.registers.h, self.registers.l, self.registers.sp, self.registers.pc, *memory.get(self.registers.pc as usize).unwrap(), memory.get(self.registers.pc as usize).unwrap().wrapping_add(1), memory.get(self.registers.pc as usize).unwrap().wrapping_add(2), memory.get(self.registers.pc as usize).unwrap().wrapping_add(3));
        }
        
        match opcode { 
            0x00 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} NOP, at PC {:#06X}", opcode, self.registers.pc);
                }
                
                self.cycles = self.cycles.wrapping_add(4);
                false
            }
            0x0D => {
                self.cycles = self.cycles.wrapping_add(4);
                let original = self.registers.c;
                self.registers.c = self.registers.c.wrapping_sub(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} DEC C, C now is {:#04X}, at PC {:#06X}", opcode, self.registers.c, self.registers.pc);
                }

                self.registers.set_z(self.registers.c == 0);
                self.registers.set_n(true);
                self.registers.set_h((original & 0x0F) == 0x00);
                false
            }
            0x0E => {
                self.cycles = self.cycles.wrapping_add(8);
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(imm8) = memory.get(self.registers.pc as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD C imm8, with imm8 = {:#04X}, at PC {:#06X}", opcode, imm8, self.registers.pc.wrapping_sub(1));
                    }
                    
                    self.registers.c = *imm8;
                } else {
                    eprintln!("Failed to get imm8 at PC {:#06X}", self.registers.pc);
                }
                false
            }
            0x11 => {
                self.cycles = self.cycles.wrapping_add(12);
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(low) = memory.get(self.registers.pc as usize) {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.pc as usize) {
                        let immediate = ((*high as u16) << 8) | *low as u16;
                        
                        if self.debug_instructions {
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
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD [DE] A, with DE = {:#06X} & A = {:#04X}, at PC {:#06X}", opcode, self.registers.get_de(), self.registers.a, self.registers.pc);
                }

                self.cycles = self.cycles.wrapping_add(8);
                memory.write_memory(self.registers.get_de() as usize, self.registers.a);
                false
            }
            0x14 => {
                self.cycles = self.cycles.wrapping_add(4);
                let original = self.registers.d;
                self.registers.d = self.registers.d.wrapping_add(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} INC D, D now is {:#04X}, at PC {:#06X}", opcode, self.registers.d, self.registers.pc);
                }

                self.registers.set_z(self.registers.d == 0);
                self.registers.set_n(false);
                self.registers.set_h((original & 0x0F) == 0x0F);
                false
            }
            0x18 => {
                self.cycles = self.cycles.wrapping_add(12);
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(offset) = memory.get(self.registers.pc as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} JR e8, with e8 = {:#04X}, at PC {:#06X}", opcode, *offset, self.registers.pc.wrapping_sub(1));
                    }

                    self.registers.pc = self.registers.pc.wrapping_add_signed(*offset as i8 as i16);
                } else {
                    eprintln!("Failed to get offset for jump at PC {:#06X}", self.registers.pc)
                }
                false
            }
            0x1C => {
                self.cycles = self.cycles.wrapping_add(4);
                let original = self.registers.e;
                self.registers.e = self.registers.e.wrapping_add(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} INC E, E now is {:#04X}, at PC {:#06X}", opcode, self.registers.e, self.registers.pc);
                }

                self.registers.set_z(self.registers.e == 0);
                self.registers.set_n(false);
                self.registers.set_h((original & 0x0F) == 0x0F);
                false
            }
            0x20 => {
                self.cycles = self.cycles.wrapping_add(8);
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if !self.registers.get_z() {
                    if let Some(offset) = memory.get(self.registers.pc as usize) {
                        if self.debug_instructions {
                            println!("Opcode: {:#04X} JR NZ e8, with e8 = {:#04X}, at PC {:#06X}", opcode, *offset, self.registers.pc.wrapping_sub(1));
                        }

                        self.registers.pc = self.registers.pc.wrapping_add_signed(*offset as i8 as i16);
                        self.cycles = self.cycles.wrapping_add(4);
                    } else {
                        eprintln!("Failed to get offset for jump at PC {:#06X}", self.registers.pc);
                    }
                } else if self.debug_instructions {
                    println!("Opcode: {:#04X} JR NZ but Z is true, at PC {:#06X}", opcode, self.registers.pc.wrapping_sub(1));
                }
                false
            }
            0x21 => {
                self.cycles = self.cycles.wrapping_add(12);
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(low) = memory.get(self.registers.pc as usize) {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.pc as usize) {
                        let immediate = ((*high as u16) << 8) | *low as u16;
                        self.registers.set_hl(immediate);

                        if self.debug_instructions {
                            println!("Opcode: {:#04X} LD HL imm16, with imm16 = {:#06X}, at PC {:#06X}", opcode, immediate, self.registers.pc.wrapping_sub(2));
                        }
                    } else {
                        eprintln!("Failed to get high value of immediate at PC {:#06X}", self.registers.pc);
                    }
                } else {
                    eprintln!("Failed to get low value of immediate at PC {:#06X}", self.registers.pc);
                }
                false
            }
            0x23 => {
                self.cycles = self.cycles.wrapping_add(8);
                self.registers.set_hl(self.registers.get_hl().wrapping_add(1));

                if self.debug_instructions {
                    println!("Opcode: {:#04X} INC HL, HL now is {:#06X}, at PC {:#06X}", opcode, self.registers.get_hl(), self.registers.pc);
                }
                
                false
            }
            0x2A => {
                self.cycles = self.cycles.wrapping_add(8);
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD A [HL+], with [HL] = {:#04X} & HL = {:#06X}, at PC {:#06X}", opcode, *value, self.registers.get_hl().wrapping_add(1), self.registers.pc);
                    }

                    self.registers.a = *value;
                    self.registers.set_hl(self.registers.get_hl().wrapping_add(1));
                } else {
                    eprintln!("Failed to get value at [HL] {:#06X}", self.registers.get_hl());
                }
                false
            }
            0x31 => {
                self.cycles = self.cycles.wrapping_add(12);
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(low) = memory.get(self.registers.pc as usize) {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.pc as usize) {
                        let immediate = ((*high as u16) << 8) | *low as u16;

                        if self.debug_instructions {
                            println!("Opcode: {:#04X} LD SP imm16, with imm16 = {:#06X}, at PC {:#06X}", opcode, immediate, self.registers.pc.wrapping_sub(2));
                        }

                        self.registers.sp = immediate;
                    } else {
                        eprintln!("Failed to get high value of immediate at PC {:#06X}", self.registers.pc);
                    }
                } else {
                    eprintln!("Failed to get low value of immediate at PC {:#06X}", self.registers.pc);
                }
                false
            }
            0x3E => {
                self.cycles = self.cycles.wrapping_add(8);
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(imm8) = memory.get(self.registers.pc as usize) {
                    self.registers.a = *imm8;
                    
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD A imm8, with imm8 = {:#04X}, at PC {:#06X}", opcode, imm8, self.registers.pc.wrapping_sub(1));
                    }
                } else {
                    eprintln!("Failed to read immediate value at PC {:#06X}", self.registers.pc);
                }
                false
            }
            0x47 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD B A, with A = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }

                self.cycles = self.cycles.wrapping_add(4);
                self.registers.b = self.registers.a;
                false
            }
            0x6B => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD L E, with E = {:#04X}, at PC {:#06X}", opcode, self.registers.e, self.registers.pc);
                }

                self.cycles = self.cycles.wrapping_add(4);
                self.registers.l = self.registers.e;
                false
            }
            0x78 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD A B, with B = {:#04X}, at PC {:#06X}", opcode, self.registers.b, self.registers.pc);
                }

                self.cycles = self.cycles.wrapping_add(4);
                self.registers.a = self.registers.b;
                false
            }
            0x7C => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD A H, with H = {:#04X}, at PC {:#06X}", opcode, self.registers.h, self.registers.pc);
                }

                self.cycles = self.cycles.wrapping_add(4);
                self.registers.a = self.registers.h;
                false
            }
            0x7D => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD A L, with L = {:#04X}, at PC {:#06X}", opcode, self.registers.l, self.registers.pc);
                }

                self.cycles = self.cycles.wrapping_add(4);
                self.registers.a = self.registers.l;
                false
            }
            0xC1 => {
                self.cycles = self.cycles.wrapping_add(12);
                if let Some(low) = memory.get(self.registers.sp as usize) {
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        self.registers.set_bc(((*high as u16) << 8) | *low as u16);

                        if self.debug_instructions {
                            println!("Opcode: {:#04X} POP BC, with BC = {:#06X}, SP = {:#06X}) at PC {:#06X}", opcode, self.registers.get_bc(), self.registers.sp, self.registers.pc);
                        }
                    } else {
                        eprintln!("Failed to get high value of jump address at PC {:#06X}", self.registers.pc);
                    }
                } else {
                    eprintln!("Failed to get low value of jump address at PC {:#06X}", self.registers.pc);
                }
                false
            }
            0xC3 => {
                self.cycles = self.cycles.wrapping_add(16);
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(low) = memory.get(self.registers.pc as usize) {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.pc as usize) {
                        let address = ((*high as u16) << 8) | *low as u16;

                        if self.debug_instructions {
                            println!("Opcode: {:#04X} JP a16, with a16 = {:#06X}, at PC {:#06X}", opcode, address, self.registers.pc.wrapping_sub(2));
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
            0xC9 => {
                self.cycles = self.cycles.wrapping_add(16);
                if let Some(low) = memory.get(self.registers.sp as usize) {
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        let return_address = ((*high as u16) << 8) | *low as u16;
                        
                        if self.debug_instructions {
                            println!("Opcode: {:#04X} RET to {:#06X}, PC was {:#06X}", opcode, return_address, self.registers.pc);
                        }
                        
                        self.registers.pc = return_address;
                        true
                    } else {
                        eprintln!("Failed to get high value of return address at PC {:#06X}", self.registers.pc);
                        false
                    }
                } else {
                    eprintln!("Failed to get low value of return address at PC {:#06X}", self.registers.pc);
                    false
                }
            }
            0xCD => {
                self.cycles = self.cycles.wrapping_add(24);
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(low) = memory.get(self.registers.pc as usize) {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.pc as usize) {
                        let address = ((*high as u16) << 8) | *low as u16;
                        let return_address = self.registers.pc.wrapping_add(1);
                        self.registers.sp = self.registers.sp.wrapping_sub(1);
                        memory.write_memory(self.registers.sp as usize, (return_address >> 8) as u8);
                        self.registers.sp = self.registers.sp.wrapping_sub(1);
                        memory.write_memory(self.registers.sp as usize, return_address as u8);

                        if self.debug_instructions {
                            println!("Opcode: {:#04X} CALL a16, with a16 = {:#06X}, at PC {:#06X}", opcode, address, self.registers.pc.wrapping_sub(2));
                        }

                        self.registers.pc = address;
                        true
                    } else {
                        eprintln!("Failed to get high value of call address at PC {:#06X}", self.registers.pc);
                        false
                    }
                } else {
                    eprintln!("Failed to get low value of call address at PC {:#06X}", self.registers.pc);
                    false
                }
            }
            0xE0 => {
                self.cycles = self.cycles.wrapping_add(12);
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(value) = memory.get(self.registers.pc as usize) {
                    let address = 0xFF00 | *value as u16;
                    
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LDH [a8] A, with a8 = {:#04X} & A = {:#04X} at PC {:#06X}", opcode, *value, self.registers.a, self.registers.pc.wrapping_sub(1));
                    }

                    memory.write_memory(address as usize, self.registers.a);
                } else {
                    eprintln!("Failed to get value at PC {:#06X}", self.registers.pc)
                }
                false
            }
            0xE1 => {
                self.cycles = self.cycles.wrapping_add(12);
                if let Some(low) = memory.get(self.registers.sp as usize) {
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        self.registers.set_hl(((*high as u16) << 8) | *low as u16);

                        if self.debug_instructions {
                            println!("Opcode: {:#04X} POP HL, with HL = {:#06X}, SP = {:#06X}) at PC {:#06X}", opcode, self.registers.get_hl(), self.registers.sp, self.registers.pc);
                        }
                    } else {
                        eprintln!("Failed to get high value of pop at PC {:#06X}", self.registers.pc);
                    }
                } else {
                    eprintln!("Failed to get low value of pop at PC {:#06X}", self.registers.pc);
                }
                false
            }
            0xE5 => {
                self.cycles = self.cycles.wrapping_add(16);
                let hl = self.registers.get_hl();
                let low = hl as u8;
                let high = (hl >> 8) as u8;
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, high);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, low);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} PUSH HL, with HL = {:#06X}, SP now {:#06X}, at PC {:#06X}", opcode, hl, self.registers.sp, self.registers.pc);
                }

                false
            }
            0xEA => {
                self.cycles = self.cycles.wrapping_add(16);
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(low) = memory.get(self.registers.pc as usize) {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.pc as usize) {
                        let address = ((*high as u16) << 8) | *low as u16;
                        memory.write_memory(address as usize, self.registers.a);
                        
                        if self.debug_instructions {
                            println!("Opcode: {:#04X} LD [a16] A, a16 = {:#06X} & A = {:#04X} at PC {:#06X}", opcode, address, self.registers.a, self.registers.pc.wrapping_sub(2));
                        }
                    } else {
                        println!("Failed to get high value of a16 at PC {:#06X}", self.registers.pc)
                    }
                } else {
                    println!("Failed to get low value of a16 at PC {:#06X}", self.registers.pc)
                }
                false
            }
            0xF1 => {
                self.cycles = self.cycles.wrapping_add(12);
                if let Some(low) = memory.get(self.registers.sp as usize) {
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        self.registers.set_af(((*high as u16) << 8) | *low as u16);

                        if self.debug_instructions {
                            println!("Opcode: {:#04X} POP AF, with AF = {:#06X}, SP = {:#06X}) at PC {:#06X}", opcode, self.registers.get_af(), self.registers.sp, self.registers.pc);
                        }
                    } else {
                        eprintln!("Failed to get high value of pop at PC {:#06X}", self.registers.pc);
                    }
                } else {
                    eprintln!("Failed to get low value of pop at PC {:#06X}", self.registers.pc);
                }
                false
            }
            0xF3 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} DI, at PC {:#06X}", opcode, self.registers.pc);
                }
                
                self.ime_scheduled = true;
                self.cycles = self.cycles.wrapping_add(4);
                false
            }
            0xF5 => {
                self.cycles = self.cycles.wrapping_add(16);
                let af = self.registers.get_af();
                let low = af as u8;
                let high = (af >> 8) as u8;
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, high);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, low);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} PUSH AF, with AF = {:#06X}, SP now {:#06X}, at PC {:#06X}", opcode, af, self.registers.sp, self.registers.pc);
                }

                false
            }
            _ => {
                panic!("Unimplemented opcode: {:#04X}, at PC {:#06X}", opcode, self.registers.pc)
            }
        }
    }
}