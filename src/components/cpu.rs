use crate::components::memory::Memory;
use crate::components::registers::Registers;

pub struct Cpu {
    pub(crate) registers: Registers,
    debug_registers: bool,
    debug_instructions: bool,
    pub(crate) ime: bool,
    ime_pending: u8,
    pub(crate) halted: bool
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: Registers::default(),
            debug_registers: false,
            debug_instructions: false,
            ime: false,
            ime_pending: 0,
            halted: false
        }
    }
    
    pub fn toggle_debug_instructions(&mut self) {
        self.debug_instructions = !self.debug_instructions
    }

    pub fn toggle_debug_registers(&mut self) {
        self.debug_registers = !self.debug_registers
    }

    pub(crate) fn update_ime(&mut self) {
        if self.ime_pending > 0 {
            self.ime_pending -= 1;
            if self.ime_pending == 0 {
                self.ime = true;
            }
        }
    }

    pub fn check_interrupts(&mut self,  memory: &mut Memory) -> Option<u64> {
        if self.ime {
            if let Some(ie) = memory.get(0xFFFF) {
                if let Some(if_) = memory.get(0xFF0F) {

                    let ie = *ie;
                    let if_ = *if_;
                    let pending = ie & if_;

                    if pending != 0 {
                        let vector = match pending.trailing_zeros() {
                            0 => 0x40, // VBlank
                            1 => 0x48, // LCD STAT
                            2 => 0x50, // Timer
                            3 => 0x58, // Serial
                            4 => 0x60, // Joypad
                            _ => unreachable!(),
                        };

                        let high = (self.registers.pc >> 8) as u8;
                        let low = self.registers.pc as u8;
                        self.registers.sp = self.registers.sp.wrapping_sub(1);
                        memory.write_memory(self.registers.sp as usize, high);
                        self.registers.sp = self.registers.sp.wrapping_sub(1);
                        memory.write_memory(self.registers.sp as usize, low);
                        self.registers.pc = vector;
                        memory.write_memory(0xFF0F, if_ & !(1 << pending.trailing_zeros()));
                        self.ime = false;
                        self.halted = false;
                    }
                    
                    return Some(20);
                }
            }
        }
        None
    }
    
    pub(crate) fn process_opcode(&mut self, opcode: u8, memory: &mut Memory) -> (bool, u64) {
        if self.debug_registers {
            println!("A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X})", self.registers.a, self.registers.f, self.registers.b, self.registers.c, self.registers.d, self.registers.e, self.registers.h, self.registers.l, self.registers.sp, self.registers.pc, *memory.get(self.registers.pc as usize).unwrap(), memory.get(self.registers.pc.wrapping_add(1) as usize).unwrap(), memory.get(self.registers.pc.wrapping_add(2) as usize).unwrap(), memory.get(self.registers.pc.wrapping_add(3) as usize).unwrap());
        }
        
        match opcode { 
            0x00 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} NOP, at PC {:#06X}", opcode, self.registers.pc);
                }

                (false, 4)
            }
            0x01 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(low) = memory.get(self.registers.pc as usize) {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.pc as usize) {
                        let immediate = ((*high as u16) << 8) | *low as u16;

                        if self.debug_instructions {
                            println!("Opcode: {:#04X} LD BC imm16, with imm16 = {:#06X}, at PC {:#06X}", opcode, immediate, self.registers.pc.wrapping_sub(2));
                        }

                        self.registers.set_bc(immediate);
                    } else {
                        eprintln!("Failed to get high value of immediate at PC {:#06X}", self.registers.pc);
                    }
                } else {
                    eprintln!("Failed to get low value of immediate at PC {:#06X}", self.registers.pc);
                }
                (false, 12)
            }
            0x03 => {
                self.registers.set_bc(self.registers.get_bc().wrapping_add(1));

                if self.debug_instructions {
                    println!("Opcode: {:#04X} INC BC, BC now is {:#06X}, at PC {:#06X}", opcode, self.registers.get_bc(), self.registers.pc);
                }

                (false, 8)
            }
            0x04 => {
                let original = self.registers.b;
                self.registers.b = self.registers.b.wrapping_add(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} INC B, B now is {:#04X}, at PC {:#06X}", opcode, self.registers.b, self.registers.pc);
                }

                self.registers.set_z(self.registers.b == 0);
                self.registers.set_n(false);
                self.registers.set_h((original & 0x0F) == 0x0F);
                (false, 4)
            }
            0x05 => {
                let original = self.registers.b;
                self.registers.b = self.registers.b.wrapping_sub(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} DEC B, B now is {:#04X}, at PC {:#06X}", opcode, self.registers.b, self.registers.pc);
                }

                self.registers.set_z(self.registers.b == 0);
                self.registers.set_n(true);
                self.registers.set_h((original & 0x0F) == 0x00);
                (false, 4)
            }
            0x06 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(imm8) = memory.get(self.registers.pc as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD B imm8, with imm8 = {:#04X}, at PC {:#06X}", opcode, imm8, self.registers.pc.wrapping_sub(1));
                    }

                    self.registers.b = *imm8;
                } else {
                    eprintln!("Failed to get imm8 at PC {:#06X}", self.registers.pc);
                }
                (false, 8)
            }
            0x0C => {
                let original = self.registers.c;
                self.registers.c = self.registers.c.wrapping_add(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} INC C, C now is {:#04X}, at PC {:#06X}", opcode, self.registers.c, self.registers.pc);
                }

                self.registers.set_z(self.registers.c == 0);
                self.registers.set_n(false);
                self.registers.set_h((original & 0x0F) == 0x0F);
                (false, 4)
            }
            0x0D => {
                let original = self.registers.c;
                self.registers.c = self.registers.c.wrapping_sub(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} DEC C, C now is {:#04X}, at PC {:#06X}", opcode, self.registers.c, self.registers.pc);
                }

                self.registers.set_z(self.registers.c == 0);
                self.registers.set_n(true);
                self.registers.set_h((original & 0x0F) == 0x00);
                (false, 4)
            }
            0x0E => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(imm8) = memory.get(self.registers.pc as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD C imm8, with imm8 = {:#04X}, at PC {:#06X}", opcode, imm8, self.registers.pc.wrapping_sub(1));
                    }
                    
                    self.registers.c = *imm8;
                } else {
                    eprintln!("Failed to get imm8 at PC {:#06X}", self.registers.pc);
                }
                (false, 8)
            }
            0x0F => {
                let new_carry = self.registers.a & 0x01;
                self.registers.a = self.registers.a.rotate_right(1);
                self.registers.set_z(false);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(new_carry != 0);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} RRCA, A = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }

                (false, 4)
            }
            0x11 => {
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
                    eprintln!("Failed to get low value of immediate at PC {:#06X}", self.registers.pc);
                }
                (false, 12)
            }
            0x12 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD [DE] A, with DE = {:#06X} & A = {:#04X}, at PC {:#06X}", opcode, self.registers.get_de(), self.registers.a, self.registers.pc);
                }

                memory.write_memory(self.registers.get_de() as usize, self.registers.a);
                (false, 8)
            }
            0x13 => {
                self.registers.set_de(self.registers.get_de().wrapping_add(1));

                if self.debug_instructions {
                    println!("Opcode: {:#04X} INC DE, DE now is {:#06X}, at PC {:#06X}", opcode, self.registers.get_de(), self.registers.pc);
                }

                (false, 8)
            }
            0x14 => {
                let original = self.registers.d;
                self.registers.d = self.registers.d.wrapping_add(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} INC D, D now is {:#04X}, at PC {:#06X}", opcode, self.registers.d, self.registers.pc);
                }

                self.registers.set_z(self.registers.d == 0);
                self.registers.set_n(false);
                self.registers.set_h((original & 0x0F) == 0x0F);
                (false, 4)
            }
            0x18 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(offset) = memory.get(self.registers.pc as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} JR e8, with e8 = {:#04X}, at PC {:#06X}", opcode, *offset, self.registers.pc.wrapping_sub(1));
                    }

                    self.registers.pc = self.registers.pc.wrapping_add_signed(*offset as i8 as i16);
                } else {
                    eprintln!("Failed to get offset for jump at PC {:#06X}", self.registers.pc)
                }
                (false, 12)
            }
            0x1A => {
                if let Some(value) = memory.get(self.registers.get_de() as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD A [DE], with [DE] = {:#04X} & DE = {:#06X}, at PC {:#06X}", opcode, *value, self.registers.get_de().wrapping_add(1), self.registers.pc);
                    }

                    self.registers.a = *value;
                } else {
                    eprintln!("Failed to get value at [DE] {:#06X}", self.registers.get_de());
                }
                (false, 8)
            }
            0x1C => {
                let original = self.registers.e;
                self.registers.e = self.registers.e.wrapping_add(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} INC E, E now is {:#04X}, at PC {:#06X}", opcode, self.registers.e, self.registers.pc);
                }

                self.registers.set_z(self.registers.e == 0);
                self.registers.set_n(false);
                self.registers.set_h((original & 0x0F) == 0x0F);
                (false, 4)
            }
            0x1D => {
                let original = self.registers.e;
                self.registers.e = self.registers.e.wrapping_sub(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} DEC E, E now is {:#04X}, at PC {:#06X}", opcode, self.registers.e, self.registers.pc);
                }

                self.registers.set_z(self.registers.e == 0);
                self.registers.set_n(true);
                self.registers.set_h((original & 0x0F) == 0x00);
                (false, 4)
            }
            0x1E => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(imm8) = memory.get(self.registers.pc as usize) {
                    self.registers.e = *imm8;

                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD E imm8, with imm8 = {:#04X}, at PC {:#06X}", opcode, imm8, self.registers.pc.wrapping_sub(1));
                    }
                } else {
                    eprintln!("Failed to read immediate value at PC {:#06X}", self.registers.pc);
                }
                (false, 8)
            }
            0x1F => {
                let old_carry = self.registers.get_c() as u8;
                let new_carry = self.registers.a & 0x01;
                self.registers.a  = (self.registers.a >> 1) | (old_carry << 7);
                self.registers.set_z(false);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(new_carry != 0);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} RRA, A = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }

                (false, 4)
            }
            0x20 => {
                let mut cycles = 8;
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if !self.registers.get_z() {
                    if let Some(offset) = memory.get(self.registers.pc as usize) {
                        if self.debug_instructions {
                            println!("Opcode: {:#04X} JR NZ e8, with e8 = {:#04X}, at PC {:#06X}", opcode, *offset, self.registers.pc.wrapping_sub(1));
                        }

                        self.registers.pc = self.registers.pc.wrapping_add_signed(*offset as i8 as i16);
                        cycles += 4;
                    } else {
                        eprintln!("Failed to get offset for jump at PC {:#06X}", self.registers.pc);
                    }
                } else if self.debug_instructions {
                    println!("Opcode: {:#04X} JR NZ but Z is true, at PC {:#06X}", opcode, self.registers.pc.wrapping_sub(1));
                }
                (false, cycles)
            }
            0x21 => {
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
                (false, 12)
            }
            0x22 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD [HL+] A, with HL = {:#06X} & A = {:#04X}, at PC {:#06X}", opcode, self.registers.get_hl(), self.registers.a, self.registers.pc);
                }

                memory.write_memory(self.registers.get_hl() as usize, self.registers.a);
                self.registers.set_hl(self.registers.get_hl().wrapping_add(1));
                (false, 8)
            }
            0x23 => {
                self.registers.set_hl(self.registers.get_hl().wrapping_add(1));

                if self.debug_instructions {
                    println!("Opcode: {:#04X} INC HL, HL now is {:#06X}, at PC {:#06X}", opcode, self.registers.get_hl(), self.registers.pc);
                }

                (false, 8)
            }
            0x24 => {
                let original = self.registers.h;
                self.registers.h = self.registers.h.wrapping_add(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} INC H, H now is {:#04X}, at PC {:#06X}", opcode, self.registers.h, self.registers.pc);
                }

                self.registers.set_z(self.registers.h == 0);
                self.registers.set_n(false);
                self.registers.set_h((original & 0x0F) == 0x0F);
                (false, 4)
            }
            0x25 => {
                let original = self.registers.h;
                self.registers.h = self.registers.h.wrapping_sub(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} DEC H, H now is {:#04X}, at PC {:#06X}", opcode, self.registers.h, self.registers.pc);
                }

                self.registers.set_z(self.registers.h == 0);
                self.registers.set_n(true);
                self.registers.set_h((original & 0x0F) == 0x00);
                (false, 4)
            }
            0x26 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(imm8) = memory.get(self.registers.pc as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD H imm8, with imm8 = {:#04X}, at PC {:#06X}", opcode, imm8, self.registers.pc.wrapping_sub(1));
                    }

                    self.registers.h = *imm8;
                } else {
                    eprintln!("Failed to get imm8 at PC {:#06X}", self.registers.pc);
                }
                (false, 8)
            }
            0x27 => {
                let mut a = self.registers.a;
                let mut correction: u8 = 0;

                if self.registers.get_h() || (!self.registers.get_n() && (a & 0x0F) > 0x09) {
                    correction |= 0x06;
                }
                if self.registers.get_c() || (!self.registers.get_n() && a > 0x99) {
                    correction |= 0x60;
                    self.registers.set_c(true);
                }

                if self.registers.get_n() {
                    a = a.wrapping_sub(correction);
                } else {
                    a = a.wrapping_add(correction);
                }

                self.registers.set_z(a == 0);
                self.registers.set_h(false);
                self.registers.a = a;

                if self.debug_instructions {
                    println!("Opcode: {:#04X} DAA, A adjusted to {:#04X}, at PC {:#06X}", opcode, a, self.registers.pc);
                }

                (false, 4)
            }
            0x28 => {
                let mut cycles = 8;
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if self.registers.get_z() {
                    if let Some(offset) = memory.get(self.registers.pc as usize) {
                        if self.debug_instructions {
                            println!("Opcode: {:#04X} JR Z e8, with e8 = {:#04X}, at PC {:#06X}", opcode, *offset, self.registers.pc.wrapping_sub(1));
                        }

                        self.registers.pc = self.registers.pc.wrapping_add_signed(*offset as i8 as i16);
                        cycles += 4;
                    } else {
                        eprintln!("Failed to get offset for jump at PC {:#06X}", self.registers.pc);
                    }
                } else if self.debug_instructions {
                    println!("Opcode: {:#04X} JR Z but Z is false, at PC {:#06X}", opcode, self.registers.pc.wrapping_sub(1));
                }
                (false, cycles)
            }
            0x29 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                let hl = self.registers.get_hl();
                let sum = hl as u32 + hl as u32;
                let new_hl = sum as u16;
                self.registers.set_hl(new_hl);
                self.registers.set_n(false);
                self.registers.set_h((hl & 0x0FFF) * 2 > 0x0FFF);
                self.registers.set_c(sum > 0xFFFF);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} ADD HL HL, with HL = {:#04X}, at PC {:#06X}", opcode, new_hl, self.registers.pc.wrapping_sub(1));
                }

                (false, 8)
            }
            0x2A => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD A [HL+], with [HL] = {:#04X} & HL = {:#06X}, at PC {:#06X}", opcode, *value, self.registers.get_hl(), self.registers.pc);
                    }

                    self.registers.a = *value;
                    self.registers.set_hl(self.registers.get_hl().wrapping_add(1));
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x2C => {
                let original = self.registers.l;
                self.registers.l = self.registers.l.wrapping_add(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} INC L, L now is {:#04X}, at PC {:#06X}", opcode, self.registers.l, self.registers.pc);
                }

                self.registers.set_z(self.registers.l == 0);
                self.registers.set_n(false);
                self.registers.set_h((original & 0x0F) == 0x0F);
                (false, 4)
            }
            0x2D => {
                let original = self.registers.l;
                self.registers.l = self.registers.l.wrapping_sub(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} DEC L, L now is {:#04X}, at PC {:#06X}", opcode, self.registers.l, self.registers.pc);
                }

                self.registers.set_z(self.registers.l == 0);
                self.registers.set_n(true);
                self.registers.set_h((original & 0x0F) == 0x00);
                (false, 4)
            }
            0x2E => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(imm8) = memory.get(self.registers.pc as usize) {
                    self.registers.l = *imm8;

                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD L imm8, with imm8 = {:#04X}, at PC {:#06X}", opcode, imm8, self.registers.pc.wrapping_sub(1));
                    }
                } else {
                    eprintln!("Failed to read immediate value at PC {:#06X}", self.registers.pc);
                }
                (false, 8)
            }
            0x2F => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} CPL, at PC {:#06X}", opcode, self.registers.pc);
                }

                self.registers.a = !self.registers.a;
                self.registers.set_n(true);
                self.registers.set_h(true);
                (false, 4)
            }
            0x30 => {
                let mut cycles = 8;
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if !self.registers.get_c() {
                    if let Some(offset) = memory.get(self.registers.pc as usize) {
                        if self.debug_instructions {
                            println!("Opcode: {:#04X} JR NC e8, with e8 = {:#04X}, at PC {:#06X}", opcode, *offset, self.registers.pc.wrapping_sub(1));
                        }

                        self.registers.pc = self.registers.pc.wrapping_add_signed(*offset as i8 as i16);
                        cycles += 4;
                    } else {
                        eprintln!("Failed to get offset for jump at PC {:#06X}", self.registers.pc);
                    }
                } else if self.debug_instructions {
                    println!("Opcode: {:#04X} JR NC but C is true, at PC {:#06X}", opcode, self.registers.pc.wrapping_sub(1));
                }
                (false, cycles)
            }
            0x31 => {
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
                (false, 12)
            }
            0x32 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD [HL-] A, with HL = {:#06X} & A = {:#04X}, at PC {:#06X}", opcode, self.registers.get_hl(), self.registers.a, self.registers.pc);
                }

                memory.write_memory(self.registers.get_hl() as usize, self.registers.a);
                self.registers.set_hl(self.registers.get_hl().wrapping_sub(1));
                (false, 8)
            }
            0x35 => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    let original = *value;
                    let result = value.wrapping_sub(1);
                    memory.write_memory(self.registers.get_hl() as usize, result);

                    if self.debug_instructions {
                        println!("Opcode: {:#04X} DEC [HL], [HL] now is {:#04X}, at PC {:#06X}", opcode, result, self.registers.pc);
                    }

                    self.registers.set_z(result== 0);
                    self.registers.set_n(true);
                    self.registers.set_h((original & 0x0F) == 0x00);
                } else {
                    eprintln!("Failed to access [HL] at HL {:#06X}", self.registers.get_hl());
                }

                (false, 12)
            }
            0x38 => {
                let mut cycles = 8;
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if self.registers.get_c() {
                    if let Some(offset) = memory.get(self.registers.pc as usize) {
                        if self.debug_instructions {
                            println!("Opcode: {:#04X} JR C e8, with e8 = {:#04X}, at PC {:#06X}", opcode, *offset, self.registers.pc.wrapping_sub(1));
                        }

                        self.registers.pc = self.registers.pc.wrapping_add_signed(*offset as i8 as i16);
                        cycles += 4;
                    } else {
                        eprintln!("Failed to get offset for jump at PC {:#06X}", self.registers.pc);
                    }
                } else if self.debug_instructions {
                    println!("Opcode: {:#04X} JR C but C is false, at PC {:#06X}", opcode, self.registers.pc.wrapping_sub(1));
                }
                (false, cycles)
            }
            0x3C => {
                let original = self.registers.a;
                self.registers.a = self.registers.a.wrapping_add(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} INC A, A now is {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }

                self.registers.set_z(self.registers.a == 0);
                self.registers.set_n(false);
                self.registers.set_h((original & 0x0F) == 0x0F);
                (false, 4)
            }
            0x3D => {
                let original = self.registers.a;
                self.registers.a = self.registers.a.wrapping_sub(1);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} DEC A, A now is {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }

                self.registers.set_z(self.registers.a == 0);
                self.registers.set_n(true);
                self.registers.set_h((original & 0x0F) == 0x00);
                (false, 4)
            }
            0x3E => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(imm8) = memory.get(self.registers.pc as usize) {
                    self.registers.a = *imm8;
                    
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD A imm8, with imm8 = {:#04X}, at PC {:#06X}", opcode, imm8, self.registers.pc.wrapping_sub(1));
                    }
                } else {
                    eprintln!("Failed to read immediate value at PC {:#06X}", self.registers.pc);
                }
                (false, 8)
            }
            0x40 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD B B, with B = {:#04X}, at PC {:#06X}", opcode, self.registers.b, self.registers.pc);
                }
                
                (false, 4)
            }
            0x41 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD B C, with C = {:#04X}, at PC {:#06X}", opcode, self.registers.c, self.registers.pc);
                }

                self.registers.b = self.registers.c;
                (false, 4)
            }
            0x42 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD B D, with D = {:#04X}, at PC {:#06X}", opcode, self.registers.d, self.registers.pc);
                }

                self.registers.b = self.registers.d;
                (false, 4)
            }
            0x43 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD B E, with E = {:#04X}, at PC {:#06X}", opcode, self.registers.e, self.registers.pc);
                }

                self.registers.b = self.registers.e;
                (false, 4)
            }
            0x44 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD B H, with H = {:#04X}, at PC {:#06X}", opcode, self.registers.h, self.registers.pc);
                }

                self.registers.b = self.registers.h;
                (false, 4)
            }
            0x45 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD B L, with L = {:#04X}, at PC {:#06X}", opcode, self.registers.l, self.registers.pc);
                }

                self.registers.b = self.registers.l;
                (false, 4)
            }
            0x46 => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD B [HL], with [HL] = {:#04X} & HL = {:#06X}, at PC {:#06X}", opcode, *value, self.registers.get_hl(), self.registers.pc);
                    }

                    self.registers.b = *value;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x47 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD B A, with A = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }

                self.registers.b = self.registers.a;
                (false, 4)
            }
            0x48 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD C B, with B = {:#04X}, at PC {:#06X}", opcode, self.registers.b, self.registers.pc);
                }

                self.registers.c = self.registers.b;
                (false, 4)
            }
            0x49 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD C C, with C = {:#04X}, at PC {:#06X}", opcode, self.registers.c, self.registers.pc);
                }
                
                (false, 4)
            }
            0x4A => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD C D, with D = {:#04X}, at PC {:#06X}", opcode, self.registers.d, self.registers.pc);
                }

                self.registers.c = self.registers.d;
                (false, 4)
            }
            0x4B => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD C E, with E = {:#04X}, at PC {:#06X}", opcode, self.registers.e, self.registers.pc);
                }

                self.registers.c = self.registers.e;
                (false, 4)
            }
            0x4C => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD C H, with H = {:#04X}, at PC {:#06X}", opcode, self.registers.h, self.registers.pc);
                }

                self.registers.c = self.registers.h;
                (false, 4)
            }
            0x4D => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD C L, with L = {:#04X}, at PC {:#06X}", opcode, self.registers.l, self.registers.pc);
                }

                self.registers.c = self.registers.l;
                (false, 4)
            }
            0x4E => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD C [HL], with [HL] = {:#04X} & HL = {:#06X}, at PC {:#06X}", opcode, *value, self.registers.get_hl(), self.registers.pc);
                    }

                    self.registers.c = *value;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x4F => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD C A, with A = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }

                self.registers.c = self.registers.a;
                (false, 4)
            }
            0x50 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD D B, with B = {:#04X}, at PC {:#06X}", opcode, self.registers.b, self.registers.pc);
                }

                self.registers.d = self.registers.b;
                (false, 4)
            }
            0x51 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD D C, with C = {:#04X}, at PC {:#06X}", opcode, self.registers.c, self.registers.pc);
                }

                self.registers.d = self.registers.c;
                (false, 4)
            }
            0x52 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD D D, with D = {:#04X}, at PC {:#06X}", opcode, self.registers.d, self.registers.pc);
                }
                
                (false, 4)
            }
            0x53 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD D E, with E = {:#04X}, at PC {:#06X}", opcode, self.registers.e, self.registers.pc);
                }

                self.registers.d = self.registers.e;
                (false, 4)
            }
            0x54 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD D H, with H = {:#04X}, at PC {:#06X}", opcode, self.registers.h, self.registers.pc);
                }

                self.registers.d = self.registers.h;
                (false, 4)
            }
            0x55 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD D L, with L = {:#04X}, at PC {:#06X}", opcode, self.registers.l, self.registers.pc);
                }

                self.registers.d = self.registers.l;
                (false, 4)
            }
            0x56 => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD D [HL], with [HL] = {:#04X} & HL = {:#06X}, at PC {:#06X}", opcode, *value, self.registers.get_hl(), self.registers.pc);
                    }

                    self.registers.d = *value;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x57 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD D A, with A = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }

                self.registers.d = self.registers.a;
                (false, 4)
            }
            0x58 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD E B, with B = {:#04X}, at PC {:#06X}", opcode, self.registers.b, self.registers.pc);
                }

                self.registers.e = self.registers.b;
                (false, 4)
            }
            0x59 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD E C, with C = {:#04X}, at PC {:#06X}", opcode, self.registers.c, self.registers.pc);
                }

                self.registers.e = self.registers.c;
                (false, 4)
            }
            0x5A => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD E D, with D = {:#04X}, at PC {:#06X}", opcode, self.registers.d, self.registers.pc);
                }

                self.registers.e = self.registers.d;
                (false, 4)
            }
            0x5B => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD E E, with E = {:#04X}, at PC {:#06X}", opcode, self.registers.e, self.registers.pc);
                }
                
                (false, 4)
            }
            0x5C => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD E H, with H = {:#04X}, at PC {:#06X}", opcode, self.registers.h, self.registers.pc);
                }

                self.registers.e = self.registers.h;
                (false, 4)
            }
            0x5D => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD E L, with L = {:#04X}, at PC {:#06X}", opcode, self.registers.l, self.registers.pc);
                }

                self.registers.e = self.registers.l;
                (false, 4)
            }
            0x5E => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD E [HL], with [HL] = {:#04X} & HL = {:#06X}, at PC {:#06X}", opcode, *value, self.registers.get_hl(), self.registers.pc);
                    }

                    self.registers.e = *value;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x5F => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD E A, with A = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }

                self.registers.e = self.registers.a;
                (false, 4)
            }
            0x60 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD H B, with B = {:#04X}, at PC {:#06X}", opcode, self.registers.b, self.registers.pc);
                }

                self.registers.h = self.registers.b;
                (false, 4)
            }
            0x61 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD H C, with C = {:#04X}, at PC {:#06X}", opcode, self.registers.c, self.registers.pc);
                }

                self.registers.h = self.registers.c;
                (false, 4)
            }
            0x62 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD H D, with D = {:#04X}, at PC {:#06X}", opcode, self.registers.d, self.registers.pc);
                }

                self.registers.h = self.registers.d;
                (false, 4)
            }
            0x63 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD H E, with E = {:#04X}, at PC {:#06X}", opcode, self.registers.e, self.registers.pc);
                }

                self.registers.h = self.registers.e;
                (false, 4)
            }
            0x64 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD H H, with H = {:#04X}, at PC {:#06X}", opcode, self.registers.h, self.registers.pc);
                }
                
                (false, 4)
            }
            0x65 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD H L, with L = {:#04X}, at PC {:#06X}", opcode, self.registers.l, self.registers.pc);
                }

                self.registers.h = self.registers.l;
                (false, 4)
            }
            0x66 => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD H [HL], with [HL] = {:#04X} & HL = {:#06X}, at PC {:#06X}", opcode, *value, self.registers.get_hl(), self.registers.pc);
                    }

                    self.registers.h = *value;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x67 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD H A, with A = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }

                self.registers.h = self.registers.a;
                (false, 4)
            }
            0x68 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD L B, with B = {:#04X}, at PC {:#06X}", opcode, self.registers.b, self.registers.pc);
                }

                self.registers.l = self.registers.b;
                (false, 4)
            }
            0x69 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD L C, with C = {:#04X}, at PC {:#06X}", opcode, self.registers.c, self.registers.pc);
                }

                self.registers.l = self.registers.c;
                (false, 4)
            }
            0x6A => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD L D, with D = {:#04X}, at PC {:#06X}", opcode, self.registers.d, self.registers.pc);
                }

                self.registers.l = self.registers.d;
                (false, 4)
            }
            0x6B => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD L E, with E = {:#04X}, at PC {:#06X}", opcode, self.registers.e, self.registers.pc);
                }

                self.registers.l = self.registers.e;
                (false, 4)
            }
            0x6C => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD L H, with H = {:#04X}, at PC {:#06X}", opcode, self.registers.h, self.registers.pc);
                }

                self.registers.l = self.registers.h;
                (false, 4)
            }
            0x6D => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD L L, with L = {:#04X}, at PC {:#06X}", opcode, self.registers.l, self.registers.pc);
                }
                
                (false, 4)
            }
            0x6E => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD L [HL], with [HL] = {:#04X} & HL = {:#06X}, at PC {:#06X}", opcode, *value, self.registers.get_hl(), self.registers.pc);
                    }

                    self.registers.l = *value;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x6F => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD L A, with A = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }

                self.registers.l = self.registers.a;
                (false, 4)
            }
            0x70 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD [HL] B, with HL = {:#06X} & D = {:#04X}, at PC {:#06X}", opcode, self.registers.get_hl(), self.registers.b, self.registers.pc);
                }

                memory.write_memory(self.registers.get_hl() as usize, self.registers.b);
                (false, 8)
            }
            0x71 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD [HL] C, with HL = {:#06X} & D = {:#04X}, at PC {:#06X}", opcode, self.registers.get_hl(), self.registers.c, self.registers.pc);
                }

                memory.write_memory(self.registers.get_hl() as usize, self.registers.c);
                (false, 8)
            }
            0x72 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD [HL] D, with HL = {:#06X} & D = {:#04X}, at PC {:#06X}", opcode, self.registers.get_hl(), self.registers.d, self.registers.pc);
                }

                memory.write_memory(self.registers.get_hl() as usize, self.registers.d);
                (false, 8)
            }
            0x73 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD [HL] E, with HL = {:#06X} & E = {:#04X}, at PC {:#06X}", opcode, self.registers.get_hl(), self.registers.e, self.registers.pc);
                }

                memory.write_memory(self.registers.get_hl() as usize, self.registers.e);
                (false, 8)
            }
            0x74 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD [HL] H, with HL = {:#06X} & H = {:#04X}, at PC {:#06X}", opcode, self.registers.get_hl(), self.registers.h, self.registers.pc);
                }

                memory.write_memory(self.registers.get_hl() as usize, self.registers.h);
                (false, 8)
            }
            0x75 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD [HL] L, with HL = {:#06X} & L = {:#04X}, at PC {:#06X}", opcode, self.registers.get_hl(), self.registers.l, self.registers.pc);
                }

                memory.write_memory(self.registers.get_hl() as usize, self.registers.l);
                (false, 8)
            }
            0x76 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} HALT, at PC {:#06X}", opcode, self.registers.pc);
                }

                self.halted = true;
                (false, 4)
            }
            0x77 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD [HL] A, with HL = {:#06X} & A = {:#04X}, at PC {:#06X}", opcode, self.registers.get_hl(), self.registers.a, self.registers.pc);
                }

                memory.write_memory(self.registers.get_hl() as usize, self.registers.a);
                (false, 8)
            }
            0x78 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD A B, with B = {:#04X}, at PC {:#06X}", opcode, self.registers.b, self.registers.pc);
                }

                self.registers.a = self.registers.b;
                (false, 4)
            }
            0x79 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD A C, with C = {:#04X}, at PC {:#06X}", opcode, self.registers.c, self.registers.pc);
                }

                self.registers.a = self.registers.c;
                (false, 4)
            }
            0x7A => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD A D, with D = {:#04X}, at PC {:#06X}", opcode, self.registers.d, self.registers.pc);
                }

                self.registers.a = self.registers.d;
                (false, 4)
            }
            0x7B => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD A E, with E = {:#04X}, at PC {:#06X}", opcode, self.registers.e, self.registers.pc);
                }

                self.registers.a = self.registers.e;
                (false, 4)
            }
            0x7C => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD A H, with H = {:#04X}, at PC {:#06X}", opcode, self.registers.h, self.registers.pc);
                }

                self.registers.a = self.registers.h;
                (false, 4)
            }
            0x7D => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD A L, with L = {:#04X}, at PC {:#06X}", opcode, self.registers.l, self.registers.pc);
                }

                self.registers.a = self.registers.l;
                (false, 4)
            }
            0x7E => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD A [HL], with [HL] = {:#04X} & HL = {:#06X}, at PC {:#06X}", opcode, *value, self.registers.get_hl(), self.registers.pc);
                    }

                    self.registers.a = *value;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x7F => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} LD A A, with A = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }
                
                (false, 4)
            }
            0xA9 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} XOR A C, A = {:#04X}, C = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.c, self.registers.pc);
                }

                self.registers.a ^= self.registers.c;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xAE => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} XOR A [HL], A = {:#04X}, [HL] = {:#04X}, at PC {:#06X}", opcode, self.registers.a, *value, self.registers.pc);
                    }

                    self.registers.a ^= value;
                    self.registers.set_z(self.registers.a == 0x00);
                    self.registers.set_n(false);
                    self.registers.set_h(false);
                    self.registers.set_c(false);
                }
                (false, 8)
            }
            0xAF => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} XOR A A, A = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }

                self.registers.a ^= self.registers.a;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xB1 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} OR A C, A = {:#04X}, C = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.c, self.registers.pc);
                }

                self.registers.a |= self.registers.c;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xB6 => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} OR A [HL], HL = {:#06X}, at PC {:#06X}", opcode, self.registers.get_hl(), self.registers.pc);
                    }

                    self.registers.a |= *value;
                    self.registers.set_z(self.registers.a == 0x00);
                    self.registers.set_n(false);
                    self.registers.set_h(false);
                    self.registers.set_c(false);
                } else {
                    eprintln!("Failed to access [HL] at HL {:#06X}", self.registers.get_hl());
                }

                (false, 8)
            }
            0xB7 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} OR A A, A = {:#04X}, at PC {:#06X}", opcode, self.registers.a, self.registers.pc);
                }

                self.registers.a |= self.registers.a;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xB8 => {
                self.registers.set_z(self.registers.a == self.registers.b);
                self.registers.set_n(true);
                self.registers.set_h((self.registers.a & 0x0F) < (self.registers.b & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.b);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} CP A B, with A = {:#04X} & B = {:#04X}, at PC {:#06X}", opcode, self.registers.a , self.registers.b, self.registers.pc);
                }

                (false, 4)
            }
            0xB9 => {
                self.registers.set_z(self.registers.a == self.registers.c);
                self.registers.set_n(true);
                self.registers.set_h((self.registers.a & 0x0F) < (self.registers.c & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.c);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} CP A C, with A = {:#04X} & C = {:#04X}, at PC {:#06X}", opcode, self.registers.a , self.registers.c, self.registers.pc);
                }

                (false, 4)
            }
            0xBA => {
                self.registers.set_z(self.registers.a == self.registers.d);
                self.registers.set_n(true);
                self.registers.set_h((self.registers.a & 0x0F) < (self.registers.d & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.d);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} CP A D, with A = {:#04X} & D = {:#04X}, at PC {:#06X}", opcode, self.registers.a , self.registers.d, self.registers.pc);
                }

                (false, 4)
            }
            0xBB => {
                self.registers.set_z(self.registers.a == self.registers.e);
                self.registers.set_n(true);
                self.registers.set_h((self.registers.a & 0x0F) < (self.registers.e & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.e);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} CP A E, with A = {:#04X} & E = {:#04X}, at PC {:#06X}", opcode, self.registers.a , self.registers.e, self.registers.pc);
                }

                (false, 4)
            }
            0xC1 => {
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
                (false, 12)
            }
            0xC2 => {
                if !self.registers.get_z() {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(low) = memory.get(self.registers.pc as usize) {
                        self.registers.pc = self.registers.pc.wrapping_add(1);
                        if let Some(high) = memory.get(self.registers.pc as usize) {
                            let address = ((*high as u16) << 8) | *low as u16;

                            if self.debug_instructions {
                                println!("Opcode: {:#04X} JP NZ a16, with a16 = {:#06X}, at PC {:#06X}", opcode, address, self.registers.pc.wrapping_sub(2));
                            }

                            self.registers.pc = address;
                            (true, 16)
                        } else {
                            eprintln!("Failed to get high value of jump address at PC {:#06X}", self.registers.pc);
                            (false, 12)
                        }
                    } else {
                        eprintln!("Failed to get low value of jump address at PC {:#06X}", self.registers.pc);
                        (false, 12)
                    }
                } else {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} JP NZ a16 but Z was true, at PC {:#06X}", opcode, self.registers.pc);
                    }
                    
                    self.registers.pc = self.registers.pc.wrapping_add(2);
                    (false, 12)
                }
            }
            0xC3 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(low) = memory.get(self.registers.pc as usize) {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.pc as usize) {
                        let address = ((*high as u16) << 8) | *low as u16;

                        if self.debug_instructions {
                            println!("Opcode: {:#04X} JP a16, with a16 = {:#06X}, at PC {:#06X}", opcode, address, self.registers.pc.wrapping_sub(2));
                        }

                        self.registers.pc = address;
                        (true, 16)
                    } else {
                        eprintln!("Failed to get high value of jump address at PC {:#06X}", self.registers.pc);
                        (false, 16)
                    }
                } else { 
                    eprintln!("Failed to get low value of jump address at PC {:#06X}", self.registers.pc);
                    (false, 16)
                }
            }
            0xC4 => {
                if !self.registers.get_z() {
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
                                println!("Opcode: {:#04X} CALL NZ a16, with a16 = {:#06X}, at PC {:#06X}", opcode, address, self.registers.pc.wrapping_sub(2));
                            }

                            self.registers.pc = address;
                            (true, 24)
                        } else {
                            eprintln!("Failed to get high value of call address at PC {:#06X}", self.registers.pc);
                            (false, 24)
                        }
                    } else {
                        eprintln!("Failed to get low value of call address at PC {:#06X}", self.registers.pc);
                        (false, 24)
                    }
                } else {
                    self.registers.pc = self.registers.pc.wrapping_add(2);

                    if self.debug_instructions {
                        println!("Opcode: {:#04X} CALL NZ a16 but Z was true", opcode);
                    }
                    (false, 12)
                }
            }
            0xC5 => {
                let bc = self.registers.get_bc();
                let low = bc as u8;
                let high = (bc >> 8) as u8;
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, high);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, low);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} PUSH BC, with BC = {:#06X}, SP now {:#06X}, at PC {:#06X}", opcode, bc, self.registers.sp, self.registers.pc);
                }

                (false, 16)
            }
            0xC6 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(value) = memory.get(self.registers.pc as usize) {
                    let a = self.registers.a;
                    self.registers.a = self.registers.a.wrapping_add(*value);
                    self.registers.set_z(self.registers.a == 0x00);
                    self.registers.set_n(false);
                    self.registers.set_h((a & 0x0F) + (*value & 0x0F) > 0x0F);
                    self.registers.set_c(a as u16 + *value as u16 > 0xFF);

                    if self.debug_instructions {
                        println!("Opcode: {:#04X} ADD A n8, with A = {:#04X} & n8 = {:#04X}, at PC {:#06X}", opcode, a, *value, self.registers.pc.wrapping_sub(1));
                    }
                } else {
                    eprintln!("Failed to get n8 at PC {:#06X}", self.registers.pc)
                }
                (false, 8)
            }
            0xC8 => {
                if self.registers.get_z() {
                    if let Some(low) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        if let Some(high) = memory.get(self.registers.sp as usize) {
                            self.registers.sp = self.registers.sp.wrapping_add(1);
                            let return_address = ((*high as u16) << 8) | *low as u16;

                            if self.debug_instructions {
                                println!("Opcode: {:#04X} RET Z to {:#06X}, PC was {:#06X}", opcode, return_address, self.registers.pc);
                            }

                            self.registers.pc = return_address;
                            (true, 12)
                        } else {
                            eprintln!("Failed to get high value of return address at PC {:#06X}", self.registers.pc);
                            (false, 8)
                        }
                    } else {
                        eprintln!("Failed to get low value of return address at PC {:#06X}", self.registers.pc);
                        (false, 8)
                    }
                } else {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} RET Z but Z was false, at PC {:#06X}", opcode, self.registers.pc);
                    }
                    (false, 8)
                }
            }
            0xC9 => {
                if let Some(low) = memory.get(self.registers.sp as usize) {
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        let return_address = ((*high as u16) << 8) | *low as u16;
                        
                        if self.debug_instructions {
                            println!("Opcode: {:#04X} RET to {:#06X}, PC was {:#06X}", opcode, return_address, self.registers.pc);
                        }
                        
                        self.registers.pc = return_address;
                        (true, 16)
                    } else {
                        eprintln!("Failed to get high value of return address at PC {:#06X}", self.registers.pc);
                        (false, 16)
                    }
                } else {
                    eprintln!("Failed to get low value of return address at PC {:#06X}", self.registers.pc);
                    (false, 16)
                }
            }
            0xCA => {
                if self.registers.get_z() {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(low) = memory.get(self.registers.pc as usize) {
                        self.registers.pc = self.registers.pc.wrapping_add(1);
                        if let Some(high) = memory.get(self.registers.pc as usize) {
                            let address = ((*high as u16) << 8) | *low as u16;

                            if self.debug_instructions {
                                println!("Opcode: {:#04X} JP Z a16, with a16 = {:#06X}, at PC {:#06X}", opcode, address, self.registers.pc.wrapping_sub(2));
                            }

                            self.registers.pc = address;
                            (true, 16)
                        } else {
                            eprintln!("Failed to get high value of jump address at PC {:#06X}", self.registers.pc);
                            (false, 12)
                        }
                    } else {
                        eprintln!("Failed to get low value of jump address at PC {:#06X}", self.registers.pc);
                        (false, 12)
                    }
                } else {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} JP Z a16 but Z was false, at PC {:#06X}", opcode, self.registers.pc);
                    }

                    self.registers.pc = self.registers.pc.wrapping_add(2);
                    (false, 12)
                }
            }
            0xCB => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                let mut cycles = 4;
                if let Some(prefix_opcode) = memory.get(self.registers.pc as usize) {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} PREFIX, at PC {:#06X}", opcode, self.registers.pc.wrapping_sub(1));
                    }
                    
                    cycles += self.process_prefix(*prefix_opcode, memory);
                } else {
                    eprintln!("Failed to access prefix_opcode at PC {:#06X}", self.registers.pc);
                }
                (false, cycles)
            }
            0xCD => {
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
                        (true, 24)
                    } else {
                        eprintln!("Failed to get high value of call address at PC {:#06X}", self.registers.pc);
                        (false, 24)
                    }
                } else {
                    eprintln!("Failed to get low value of call address at PC {:#06X}", self.registers.pc);
                    (false, 24)
                }
            }
            0xCE => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(value) = memory.get(self.registers.pc as usize) {
                    let a = self.registers.a;
                    let carry = self.registers.get_c() as u8;
                    let sum = a.wrapping_add(*value).wrapping_add(carry);
                    self.registers.a = sum;

                    self.registers.set_z(sum == 0);
                    self.registers.set_n(false);

                    let a_lower = a & 0x0F;
                    let d8_lower = *value & 0x0F;
                    let sum_lower = a_lower + d8_lower + carry;
                    self.registers.set_h(sum_lower > 0x0F);

                    let sum_full = (a as u16) + (*value as u16) + (carry as u16);
                    self.registers.set_c(sum_full > 0xFF);

                    if self.debug_instructions {
                        println!("Opcode: {:#04X} ADC A n8, with n8 = {:#04X} & A = {:#04X} & C={}, at PC {:#06X}", opcode, *value, a, carry, self.registers.pc.wrapping_sub(1));
                    }
                } else {
                    eprintln!("Failed to retrieve value at PC {:#06X}", self.registers.pc);
                }
                (false, 8)
            }
            0xD0 => {
                if !self.registers.get_c() {
                    if let Some(low) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        if let Some(high) = memory.get(self.registers.sp as usize) {
                            self.registers.sp = self.registers.sp.wrapping_add(1);
                            let return_address = ((*high as u16) << 8) | *low as u16;

                            if self.debug_instructions {
                                println!("Opcode: {:#04X} RET NC to {:#06X}, PC was {:#06X}", opcode, return_address, self.registers.pc);
                            }

                            self.registers.pc = return_address;
                            (true, 20)
                        } else {
                            eprintln!("Failed to get high value of return address at PC {:#06X}", self.registers.pc);
                            (false, 8)
                        }
                    } else {
                        eprintln!("Failed to get low value of return address at PC {:#06X}", self.registers.pc);
                        (false, 8)
                    }
                } else {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} RET NC but C was true, at PC {:#06X}", opcode, self.registers.pc);
                    }
                    (false, 8)
                }
            }
            0xD1 => {
                if let Some(low) = memory.get(self.registers.sp as usize) {
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        self.registers.set_de(((*high as u16) << 8) | *low as u16);

                        if self.debug_instructions {
                            println!("Opcode: {:#04X} POP DE, with DE = {:#06X}, SP = {:#06X}) at PC {:#06X}", opcode, self.registers.get_de(), self.registers.sp, self.registers.pc);
                        }
                    } else {
                        eprintln!("Failed to get high value of jump address at PC {:#06X}", self.registers.pc);
                    }
                } else {
                    eprintln!("Failed to get low value of jump address at PC {:#06X}", self.registers.pc);
                }
                (false, 12)
            }
            0xD5 => {
                let de = self.registers.get_de();
                let low = de as u8;
                let high = (de >> 8) as u8;
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, high);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, low);

                if self.debug_instructions {
                    println!("Opcode: {:#04X} PUSH DE, with DE = {:#06X}, SP now {:#06X}, at PC {:#06X}", opcode, de, self.registers.sp, self.registers.pc);
                }

                (false, 16)
            }
            0xD6 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(value) = memory.get(self.registers.pc as usize) {
                    let a = self.registers.a;
                    self.registers.a = self.registers.a.wrapping_sub(*value);
                    self.registers.set_z(self.registers.a == 0x00);
                    self.registers.set_n(true);
                    self.registers.set_h((a & 0xF) < (*value & 0xF));
                    self.registers.set_c(a < *value);

                    if self.debug_instructions {
                        println!("Opcode: {:#04X} SUB A n8, with A = {:#04X} & n8 = {:#04X}, at PC {:#06X}", opcode, a, *value, self.registers.pc.wrapping_sub(1));
                    }
                } else {
                    eprintln!("Failed to get n8 at PC {:#06X}", self.registers.pc)
                }
                (false, 8)
            }
            0xD8 => {
                if self.registers.get_c() {
                    if let Some(low) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        if let Some(high) = memory.get(self.registers.sp as usize) {
                            self.registers.sp = self.registers.sp.wrapping_add(1);
                            let return_address = ((*high as u16) << 8) | *low as u16;

                            if self.debug_instructions {
                                println!("Opcode: {:#04X} RET C to {:#06X}, PC was {:#06X}", opcode, return_address, self.registers.pc);
                            }

                            self.registers.pc = return_address;
                            (true, 20)
                        } else {
                            eprintln!("Failed to get high value of return address at PC {:#06X}", self.registers.pc);
                            (false, 8)
                        }
                    } else {
                        eprintln!("Failed to get low value of return address at PC {:#06X}", self.registers.pc);
                        (false, 8)
                    }
                } else {
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} RET C but C was false, at PC {:#06X}", opcode, self.registers.pc);
                    }
                    (false, 8)
                }
            }
            0xE0 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(value) = memory.get(self.registers.pc as usize) {
                    let address = 0xFF00 | *value as u16;
                    
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LDH [a8] A, with a8 = {:#04X} & A = {:#04X} at PC {:#06X}", opcode, *value, self.registers.a, self.registers.pc.wrapping_sub(1));
                    }

                    memory.write_memory(address as usize, self.registers.a);
                } else {
                    eprintln!("Failed to get value at PC {:#06X}", self.registers.pc);
                }
                (false, 12)
            }
            0xE1 => {
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
                (false, 12)
            }
            0xE5 => {
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

                (false, 16)
            }
            0xE6 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(value) = memory.get(self.registers.pc as usize) {
                    self.registers.a &= *value;
                    self.registers.set_z(self.registers.a == 0x00);
                    self.registers.set_n(false);
                    self.registers.set_h(true);
                    self.registers.set_c(false);

                    if self.debug_instructions {
                        println!("Opcode: {:#04X} AND A n8, with A = {:#04X} & n8 = {:#04X}, at PC {:#06X}", opcode, self.registers.a, *value, self.registers.pc.wrapping_sub(1));
                    }
                } else {
                    eprintln!("Failed to get n8 at PC {:#06X}", self.registers.pc)
                }
                (false, 8)
            }
            0xE9 => {
                self.registers.pc = self.registers.get_hl();
                (true, 4)
            }
            0xEA => {
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
                        eprintln!("Failed to get high value of a16 at PC {:#06X}", self.registers.pc);
                    }
                } else {
                    eprintln!("Failed to get low value of a16 at PC {:#06X}", self.registers.pc);
                }
                (false, 16)
            }
            0xEE => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(value) = memory.get(self.registers.pc as usize) {
                    self.registers.a ^= *value;
                    self.registers.set_z(self.registers.a == 0x00);
                    self.registers.set_n(false);
                    self.registers.set_h(false);
                    self.registers.set_c(false);

                    if self.debug_instructions {
                        println!("Opcode: {:#04X} XOR A n8, with A = {:#04X} & n8 = {:#04X}, at PC {:#06X}", opcode, self.registers.a, *value, self.registers.pc.wrapping_sub(1));
                    }
                } else {
                    eprintln!("Failed to get n8 at PC {:#06X}", self.registers.pc)
                }
                (false, 8)
            }
            0xF0 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(value) = memory.get(self.registers.pc as usize) {
                    let address = 0xFF00 | *value as u16;
                    if let Some(goal_value) = memory.get(address as usize) {
                        if self.debug_instructions {
                            println!("Opcode: {:#04X} LDH A [a8], with a8 = {:#04X} & A = {:#04X} at PC {:#06X}", opcode, *value, self.registers.a, self.registers.pc.wrapping_sub(1));
                        }

                        self.registers.a = *goal_value;
                    } else {
                        eprintln!("Failed to get value at address = {:#06X}", address);
                    }
                } else {
                    eprintln!("Failed to get value at PC {:#06X}", self.registers.pc);
                }
                (false, 12)
            }
            0xF1 => {
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
                (false, 12)
            }
            0xF3 => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} DI, at PC {:#06X}", opcode, self.registers.pc);
                }

                self.ime = false;
                self.ime_pending = 0;
                (false, 4)
            }
            0xF5 => {
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

                (false, 16)
            }
            0xF8 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(offset) = memory.get(self.registers.pc as usize) {
                    let offset = *offset as i8 as i16 as u16;
                    let sp = self.registers.sp;
                    let sum = sp.wrapping_add(offset);

                    self.registers.set_hl(sum);
                    self.registers.set_z(false);
                    self.registers.set_n(false);
                    self.registers.set_h((sp & 0xFFF) + (offset & 0xFFF) >= 0x1000);
                    self.registers.set_c((sp as u32 + offset as u32) > 0xFFFF);
                    
                    if self.debug_instructions {
                        println!("Opcode: {:#04X} LD HL, SP + e8, with SP = {:#06X} & e8 = {:#06X}, at PC {:#06X}", opcode, sp, offset, self.registers.pc);
                    }
                } else {
                    eprintln!("Failed to get offset at PC = {:#06X}", self.registers.pc);
                }
                (false, 12)
            }
            0xFA => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(low) = memory.get(self.registers.pc as usize) {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.pc as usize) {
                        let address = ((*high as u16) << 8) | *low as u16;
                        if let Some(value_goal) = memory.get(address as usize) {
                            self.registers.a = *value_goal;

                            if self.debug_instructions {
                                println!("Opcode: {:#04X} LD A [a16], a16 = {:#06X} & A = {:#04X} at PC {:#06X}", opcode, address, self.registers.a, self.registers.pc.wrapping_sub(2));
                            }
                        } else {
                            eprintln!("Failed to get value at address = {:#06X}", address);
                        }
                    } else {
                        eprintln!("Failed to get high value of a16 at PC {:#06X}", self.registers.pc);
                    }
                } else {
                    eprintln!("Failed to get low value of a16 at PC {:#06X}", self.registers.pc);
                }
                (false, 16)
            }
            0xFB => {
                if self.debug_instructions {
                    println!("Opcode: {:#04X} EI, at PC {:#06X}", opcode, self.registers.pc);
                }

                self.ime_pending = 2;
                (false, 4)
            }
            0xFE => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(n8) = memory.get(self.registers.pc as usize) {
                    let a = self.registers.a;
                    self.registers.set_z(a == *n8);
                    self.registers.set_n(true);
                    self.registers.set_h((a & 0x0F) < (*n8 & 0x0F));
                    self.registers.set_c(a < *n8);

                    if self.debug_instructions {
                        println!("Opcode: {:#04X} CP A n8, with A = {:#04X} & n8 = {:#04X}, at PC {:#06X}", opcode, a, *n8, self.registers.pc);
                    }
                } else {
                    eprintln!("Failed to get high n8 at PC {:#06X}", self.registers.pc);
                }
                (false, 8)
            }
            _ => {
                panic!("Unimplemented opcode: {:#04X}, at PC {:#06X}", opcode, self.registers.pc);
            }
        }
    }

    fn process_prefix(&mut self, prefix: u8, memory: &mut Memory) -> u64 {
        if self.debug_instructions {
            println!("Prefix: {:#04X}, at PC {:#06X}", prefix, self.registers.pc);
        }
        let operand = prefix & 0x07;
        let bit = (prefix >> 3) & 0x07;
        let group = prefix >> 6;

        let added_cycles = match group {
            0b00 => self.handle_rotate_shift(prefix, operand, memory),
            0b01 => self.handle_bit_test(bit, operand, memory),
            0b10 => self.handle_bit_reset(bit, operand, memory),
            0b11 => self.handle_bit_set(bit, operand, memory),
            _ => unreachable!(),
        };
        4 + added_cycles
    }

    fn handle_rotate_shift(&mut self, opcode: u8, operand: u8, memory: &mut Memory) -> u64 {
        let (value, cycles) = self.get_operand_value(operand, memory);
        let (result, new_c) = match opcode & 0xF8 {
            0x00 => (value.rotate_left(1), (value >> 7) & 1), // RLC
            0x08 => (value.rotate_right(1), value & 1),       // RRC
            0x10 => {
                let carry = self.registers.get_c() as u8;
                let result = (value << 1) | carry;
                let new_c = (value >> 7) & 1;
                (result, new_c)
            } // RL
            0x18 => {
                let carry = self.registers.get_c() as u8;
                let result = (value >> 1) | (carry << 7);
                let new_c = value & 1;
                (result, new_c)
            } // RR
            0x20 => (value << 1, (value >> 7) & 1), // SLA
            0x28 => ((value as i8 >> 1) as u8, value & 1), // SRA (arithmetic shift)
            0x30 => (value.rotate_left(4), 0), // SWAP
            0x38 => (value >> 1, value & 1), // SRL
            _ => panic!("Unimplemented rotate/shift opcode: 0xCB{opcode:#04X}"),
        };

        let added_cycles = self.set_operand_value(operand, result, memory);
        self.registers.set_z(result == 0);
        self.registers.set_n(false);
        self.registers.set_h(false);
        self.registers.set_c(new_c != 0);
        cycles + added_cycles
    }

    fn handle_bit_test(&mut self, bit: u8, operand: u8, memory: &mut Memory) -> u64 {
        let (value, cycles) = self.get_operand_value(operand, memory);
        let mask = 1 << bit;
        self.registers.set_z((value & mask) == 0);
        self.registers.set_n(false);
        self.registers.set_h(true);
        cycles
    }

    fn handle_bit_reset(&mut self, bit: u8, operand: u8, memory: &mut Memory) -> u64 {
        let (value, cycles) = self.get_operand_value(operand, memory);
        let result = value & !(1 << bit);
        let added_cycles = self.set_operand_value(operand, result, memory);
        cycles + added_cycles
    }

    fn handle_bit_set(&mut self, bit: u8, operand: u8, memory: &mut Memory) -> u64 {
        let (value, cycles) = self.get_operand_value(operand, memory);
        let result = value | (1 << bit);
        let added_cycles = self.set_operand_value(operand, result, memory);
        cycles + added_cycles
    }

    fn get_operand_value(&mut self, operand: u8, memory: &mut Memory) -> (u8, u64) {
        match operand {
            0 => (self.registers.b, 0),
            1 => (self.registers.c, 0),
            2 => (self.registers.d, 0),
            3 => (self.registers.e, 0),
            4 => (self.registers.h, 0),
            5 => (self.registers.l, 0),
            6 => {
                (*memory.get(self.registers.get_hl() as usize).unwrap_or_else(|| panic!("Invalid HL address {:#06X}", self.registers.get_hl())), 4)
            }
            7 => (self.registers.a, 0),
            _ => unreachable!(),
        }
    }

    fn set_operand_value(&mut self, operand: u8, value: u8, memory: &mut Memory) -> u64 {
        match operand {
            0 => self.registers.b = value,
            1 => self.registers.c = value,
            2 => self.registers.d = value,
            3 => self.registers.e = value,
            4 => self.registers.h = value,
            5 => self.registers.l = value,
            6 => {
                memory.write_memory(self.registers.get_hl() as usize, value);
                return 4;
            }
            7 => self.registers.a = value,
            _ => unreachable!(),
        }
        0
    }
}