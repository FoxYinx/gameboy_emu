use crate::components::memory::Memory;
use crate::components::registers::Registers;

pub struct Cpu {
    pub(crate) registers: Registers,
    debug_registers: bool,
    pub(crate) ime: bool,
    ime_pending: u8,
    pub(crate) halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: Registers::default(),
            debug_registers: false,
            ime: false,
            ime_pending: 0,
            halted: false,
        }
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

    pub fn check_interrupts(&mut self, memory: &mut Memory) {
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
                }
            }
        }
    }

    #[allow(unreachable_patterns)]
    pub(crate) fn process_opcode(&mut self, opcode: u8, memory: &mut Memory) -> (bool, u64) {
        if self.debug_registers {
            println!(
                "A: {:02X} F: {:02X} B: {:02X} C: {:02X} D: {:02X} E: {:02X} H: {:02X} L: {:02X} SP: {:04X} PC: 00:{:04X} ({:02X} {:02X} {:02X} {:02X})",
                self.registers.a,
                self.registers.f,
                self.registers.b,
                self.registers.c,
                self.registers.d,
                self.registers.e,
                self.registers.h,
                self.registers.l,
                self.registers.sp,
                self.registers.pc,
                *memory.get(self.registers.pc as usize).unwrap(),
                memory
                    .get(self.registers.pc.wrapping_add(1) as usize)
                    .unwrap(),
                memory
                    .get(self.registers.pc.wrapping_add(2) as usize)
                    .unwrap(),
                memory
                    .get(self.registers.pc.wrapping_add(3) as usize)
                    .unwrap()
            );
        }

        match opcode {
            0x00 => (false, 4),
            0x01 => {
                self.ld_r16_n16(memory, opcode);
                (false, 12)
            }
            0x02 => {
                memory.write_memory(self.registers.get_bc() as usize, self.registers.a);
                (false, 8)
            }
            0x03 => {
                self.registers
                    .set_bc(self.registers.get_bc().wrapping_add(1));
                (false, 8)
            }
            0x04 => {
                self.registers.b = self.inc_r8(self.registers.b);
                (false, 4)
            }
            0x05 => {
                self.registers.b = self.dec_r8(self.registers.b);
                (false, 4)
            }
            0x06 => {
                self.registers.b = self.ld_r8_n8(memory);
                (false, 8)
            }
            0x07 => {
                let original_a = self.registers.a;
                self.registers.a = original_a.rotate_left(1);
                self.registers.set_z(false);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(original_a & 0x80 != 0);
                (false, 4)
            }
            0x08 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(low) = memory.get(self.registers.pc as usize) {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.pc as usize) {
                        let address = ((*high as u16) << 8) | (*low as u16);
                        memory.write_memory(address as usize, self.registers.sp as u8);
                        memory.write_memory((address + 1) as usize, (self.registers.sp >> 8) as u8);
                    } else {
                        eprintln!(
                            "Failed to get high value of a16 at PC {:#06X}",
                            self.registers.pc
                        );
                    }
                } else {
                    eprintln!(
                        "Failed to get low value of a16 at PC {:#06X}",
                        self.registers.pc
                    );
                }
                (false, 20)
            }
            0x09 => {
                self.add_hl_r16(opcode);
                (false, 8)
            }
            0x0A => {
                if let Some(value) = memory.get(self.registers.get_bc() as usize) {
                    self.registers.a = *value;
                } else {
                    eprintln!("Failed to get value at BC {:#06X}", self.registers.get_bc());
                }
                (false, 8)
            }
            0x0B => {
                self.registers
                    .set_bc(self.registers.get_bc().wrapping_sub(1));
                (false, 8)
            }
            0x0C => {
                self.registers.c = self.inc_r8(self.registers.c);
                (false, 4)
            }
            0x0D => {
                self.registers.c = self.dec_r8(self.registers.c);
                (false, 4)
            }
            0x0E => {
                self.registers.c = self.ld_r8_n8(memory);
                (false, 8)
            }
            0x0F => {
                let new_carry = self.registers.a & 0x01;
                self.registers.a = self.registers.a.rotate_right(1);
                self.registers.set_z(false);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(new_carry != 0);
                (false, 4)
            }
            0x10 => {
                panic!("STOP");
            }
            0x11 => {
                self.ld_r16_n16(memory, opcode);
                (false, 12)
            }
            0x12 => {
                memory.write_memory(self.registers.get_de() as usize, self.registers.a);
                (false, 8)
            }
            0x13 => {
                self.registers
                    .set_de(self.registers.get_de().wrapping_add(1));
                (false, 8)
            }
            0x14 => {
                self.registers.d = self.inc_r8(self.registers.d);
                (false, 4)
            }
            0x15 => {
                self.registers.d = self.dec_r8(self.registers.d);
                (false, 4)
            }
            0x16 => {
                self.registers.d = self.ld_r8_n8(memory);
                (false, 8)
            }
            0x17 => {
                let original_a = self.registers.a;
                let carry_bit = self.registers.get_c() as u8;
                self.registers.a = (original_a << 1) | carry_bit;
                let new_carry = (original_a & 0x80) != 0;
                self.registers.set_z(false);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(new_carry);
                (false, 4)
            }
            0x18 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                self.jump_relative(memory);
                (false, 12)
            }
            0x19 => {
                self.add_hl_r16(opcode);
                (false, 8)
            }
            0x1A => {
                if let Some(value) = memory.get(self.registers.get_de() as usize) {
                    self.registers.a = *value;
                } else {
                    eprintln!("Failed to get value at DE {:#06X}", self.registers.get_de());
                }
                (false, 8)
            }
            0x1B => {
                self.registers
                    .set_de(self.registers.get_de().wrapping_sub(1));
                (false, 8)
            }
            0x1C => {
                self.registers.e = self.inc_r8(self.registers.e);
                (false, 4)
            }
            0x1D => {
                self.registers.e = self.dec_r8(self.registers.e);
                (false, 4)
            }
            0x1E => {
                self.registers.e = self.ld_r8_n8(memory);
                (false, 8)
            }
            0x1F => {
                let old_carry = self.registers.get_c() as u8;
                let new_carry = self.registers.a & 0x01;
                self.registers.a = (self.registers.a >> 1) | (old_carry << 7);
                self.registers.set_z(false);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(new_carry != 0);
                (false, 4)
            }
            0x20 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if !self.registers.get_z() {
                    self.jump_relative(memory);
                    (false, 12)
                } else {
                    (false, 8)
                }
            }
            0x21 => {
                self.ld_r16_n16(memory, opcode);
                (false, 12)
            }
            0x22 => {
                memory.write_memory(self.registers.get_hl() as usize, self.registers.a);
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_add(1));
                (false, 8)
            }
            0x23 => {
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_add(1));
                (false, 8)
            }
            0x24 => {
                self.registers.h = self.inc_r8(self.registers.h);
                (false, 4)
            }
            0x25 => {
                self.registers.h = self.dec_r8(self.registers.h);
                (false, 4)
            }
            0x26 => {
                self.registers.h = self.ld_r8_n8(memory);
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
                (false, 4)
            }
            0x28 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if self.registers.get_z() {
                    self.jump_relative(memory);
                    (false, 12)
                } else {
                    (false, 8)
                }
            }
            0x29 => {
                self.add_hl_r16(opcode);
                (false, 8)
            }
            0x2A => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    self.registers.a = *value;
                    self.registers
                        .set_hl(self.registers.get_hl().wrapping_add(1));
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x2B => {
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_sub(1));
                (false, 8)
            }
            0x2C => {
                self.registers.l = self.inc_r8(self.registers.l);
                (false, 4)
            }
            0x2D => {
                self.registers.l = self.dec_r8(self.registers.l);
                (false, 4)
            }
            0x2E => {
                self.registers.l = self.ld_r8_n8(memory);
                (false, 8)
            }
            0x2F => {
                self.registers.a = !self.registers.a;
                self.registers.set_n(true);
                self.registers.set_h(true);
                (false, 4)
            }
            0x30 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if !self.registers.get_c() {
                    self.jump_relative(memory);
                    (false, 12)
                } else {
                    (false, 8)
                }
            }
            0x31 => {
                self.ld_r16_n16(memory, opcode);
                (false, 12)
            }
            0x32 => {
                memory.write_memory(self.registers.get_hl() as usize, self.registers.a);
                self.registers
                    .set_hl(self.registers.get_hl().wrapping_sub(1));
                (false, 8)
            }
            0x33 => {
                self.registers.sp = self.registers.sp.wrapping_add(1);
                (false, 8)
            }
            0x34 => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    let result = value.wrapping_add(1);
                    self.registers.set_z(result == 0);
                    self.registers.set_n(false);
                    self.registers.set_h((*value & 0x0F) == 0x0F);
                    memory.write_memory(self.registers.get_hl() as usize, result);
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 12)
            }
            0x35 => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    let original = *value;
                    let result = value.wrapping_sub(1);
                    memory.write_memory(self.registers.get_hl() as usize, result);
                    self.registers.set_z(result == 0);
                    self.registers.set_n(true);
                    self.registers.set_h((original & 0x0F) == 0x00);
                } else {
                    eprintln!(
                        "Failed to access [HL] at HL {:#06X}",
                        self.registers.get_hl()
                    );
                }
                (false, 12)
            }
            0x36 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(imm8) = memory.get(self.registers.pc as usize) {
                    memory.write_memory(self.registers.get_hl() as usize, *imm8);
                } else {
                    eprintln!("Failed to get imm8 at PC {:#06X}", self.registers.pc);
                }
                (false, 12)
            }
            0x37 => {
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(true);
                (false, 4)
            }
            0x38 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if self.registers.get_c() {
                    self.jump_relative(memory);
                    (false, 12)
                } else {
                    (false, 8)
                }
            }
            0x39 => {
                self.add_hl_r16(opcode);
                (false, 8)
            }
            0x3A => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    self.registers.a = *value;
                    self.registers
                        .set_hl(self.registers.get_hl().wrapping_sub(1));
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x3B => {
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                (false, 8)
            }
            0x3C => {
                self.registers.a = self.inc_r8(self.registers.a);
                (false, 4)
            }
            0x3D => {
                self.registers.a = self.dec_r8(self.registers.a);
                (false, 4)
            }
            0x3E => {
                self.registers.a = self.ld_r8_n8(memory);
                (false, 8)
            }
            0x3F => {
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(!self.registers.get_c());
                (false, 4)
            }
            0x40 => (false, 4),
            0x41 => {
                self.registers.b = self.registers.c;
                (false, 4)
            }
            0x42 => {
                self.registers.b = self.registers.d;
                (false, 4)
            }
            0x43 => {
                self.registers.b = self.registers.e;
                (false, 4)
            }
            0x44 => {
                self.registers.b = self.registers.h;
                (false, 4)
            }
            0x45 => {
                self.registers.b = self.registers.l;
                (false, 4)
            }
            0x46 => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    self.registers.b = *value;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x47 => {
                self.registers.b = self.registers.a;
                (false, 4)
            }
            0x48 => {
                self.registers.c = self.registers.b;
                (false, 4)
            }
            0x49 => (false, 4),
            0x4A => {
                self.registers.c = self.registers.d;
                (false, 4)
            }
            0x4B => {
                self.registers.c = self.registers.e;
                (false, 4)
            }
            0x4C => {
                self.registers.c = self.registers.h;
                (false, 4)
            }
            0x4D => {
                self.registers.c = self.registers.l;
                (false, 4)
            }
            0x4E => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    self.registers.c = *value;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x4F => {
                self.registers.c = self.registers.a;
                (false, 4)
            }
            0x50 => {
                self.registers.d = self.registers.b;
                (false, 4)
            }
            0x51 => {
                self.registers.d = self.registers.c;
                (false, 4)
            }
            0x52 => (false, 4),
            0x53 => {
                self.registers.d = self.registers.e;
                (false, 4)
            }
            0x54 => {
                self.registers.d = self.registers.h;
                (false, 4)
            }
            0x55 => {
                self.registers.d = self.registers.l;
                (false, 4)
            }
            0x56 => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    self.registers.d = *value;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x57 => {
                self.registers.d = self.registers.a;
                (false, 4)
            }
            0x58 => {
                self.registers.e = self.registers.b;
                (false, 4)
            }
            0x59 => {
                self.registers.e = self.registers.c;
                (false, 4)
            }
            0x5A => {
                self.registers.e = self.registers.d;
                (false, 4)
            }
            0x5B => (false, 4),
            0x5C => {
                self.registers.e = self.registers.h;
                (false, 4)
            }
            0x5D => {
                self.registers.e = self.registers.l;
                (false, 4)
            }
            0x5E => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    self.registers.e = *value;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x5F => {
                self.registers.e = self.registers.a;
                (false, 4)
            }
            0x60 => {
                self.registers.h = self.registers.b;
                (false, 4)
            }
            0x61 => {
                self.registers.h = self.registers.c;
                (false, 4)
            }
            0x62 => {
                self.registers.h = self.registers.d;
                (false, 4)
            }
            0x63 => {
                self.registers.h = self.registers.e;
                (false, 4)
            }
            0x64 => (false, 4),
            0x65 => {
                self.registers.h = self.registers.l;
                (false, 4)
            }
            0x66 => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    self.registers.h = *value;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x67 => {
                self.registers.h = self.registers.a;
                (false, 4)
            }
            0x68 => {
                self.registers.l = self.registers.b;
                (false, 4)
            }
            0x69 => {
                self.registers.l = self.registers.c;
                (false, 4)
            }
            0x6A => {
                self.registers.l = self.registers.d;
                (false, 4)
            }
            0x6B => {
                self.registers.l = self.registers.e;
                (false, 4)
            }
            0x6C => {
                self.registers.l = self.registers.h;
                (false, 4)
            }
            0x6D => (false, 4),
            0x6E => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    self.registers.l = *value;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x6F => {
                self.registers.l = self.registers.a;
                (false, 4)
            }
            0x70 => {
                memory.write_memory(self.registers.get_hl() as usize, self.registers.b);
                (false, 8)
            }
            0x71 => {
                memory.write_memory(self.registers.get_hl() as usize, self.registers.c);
                (false, 8)
            }
            0x72 => {
                memory.write_memory(self.registers.get_hl() as usize, self.registers.d);
                (false, 8)
            }
            0x73 => {
                memory.write_memory(self.registers.get_hl() as usize, self.registers.e);
                (false, 8)
            }
            0x74 => {
                memory.write_memory(self.registers.get_hl() as usize, self.registers.h);
                (false, 8)
            }
            0x75 => {
                memory.write_memory(self.registers.get_hl() as usize, self.registers.l);
                (false, 8)
            }
            0x76 => {
                self.halted = true;
                (false, 4)
            }
            0x77 => {
                memory.write_memory(self.registers.get_hl() as usize, self.registers.a);
                (false, 8)
            }
            0x78 => {
                self.registers.a = self.registers.b;
                (false, 4)
            }
            0x79 => {
                self.registers.a = self.registers.c;
                (false, 4)
            }
            0x7A => {
                self.registers.a = self.registers.d;
                (false, 4)
            }
            0x7B => {
                self.registers.a = self.registers.e;
                (false, 4)
            }
            0x7C => {
                self.registers.a = self.registers.h;
                (false, 4)
            }
            0x7D => {
                self.registers.a = self.registers.l;
                (false, 4)
            }
            0x7E => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    self.registers.a = *value;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x7F => (false, 4),
            0x80 => {
                self.add_a_r8(self.registers.b);
                (false, 4)
            }
            0x81 => {
                self.add_a_r8(self.registers.c);
                (false, 4)
            }
            0x82 => {
                self.add_a_r8(self.registers.d);
                (false, 4)
            }
            0x83 => {
                self.add_a_r8(self.registers.e);
                (false, 4)
            }
            0x84 => {
                self.add_a_r8(self.registers.h);
                (false, 4)
            }
            0x85 => {
                self.add_a_r8(self.registers.l);
                (false, 4)
            }
            0x86 => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    self.add_a_r8(*value);
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0x87 => {
                self.add_a_r8(self.registers.a);
                (false, 4)
            }
            0x88 => {
                let carry = self.registers.get_c() as u8;
                let result = self
                    .registers
                    .a
                    .wrapping_add(self.registers.b)
                    .wrapping_add(carry);
                self.registers.set_z(result == 0);
                self.registers.set_n(false);
                let h_check = (self.registers.a & 0x0F) + (self.registers.b & 0x0F) + carry;
                self.registers.set_h(h_check > 0x0F);
                let c_check =
                    (self.registers.a as u16) + (self.registers.b as u16) + (carry as u16);
                self.registers.set_c(c_check > 0xFF);
                self.registers.a = result;

                (false, 4)
            }
            0x89 => {
                let carry = self.registers.get_c() as u8;
                let result = self
                    .registers
                    .a
                    .wrapping_add(self.registers.c)
                    .wrapping_add(carry);
                self.registers.set_z(result == 0);
                self.registers.set_n(false);
                let h_check = (self.registers.a & 0x0F) + (self.registers.c & 0x0F) + carry;
                self.registers.set_h(h_check > 0x0F);
                let c_check =
                    (self.registers.a as u16) + (self.registers.c as u16) + (carry as u16);
                self.registers.set_c(c_check > 0xFF);
                self.registers.a = result;

                (false, 4)
            }
            0x8A => {
                let carry = self.registers.get_c() as u8;
                let result = self
                    .registers
                    .a
                    .wrapping_add(self.registers.d)
                    .wrapping_add(carry);
                self.registers.set_z(result == 0);
                self.registers.set_n(false);
                let h_check = (self.registers.a & 0x0F) + (self.registers.d & 0x0F) + carry;
                self.registers.set_h(h_check > 0x0F);
                let c_check =
                    (self.registers.a as u16) + (self.registers.d as u16) + (carry as u16);
                self.registers.set_c(c_check > 0xFF);
                self.registers.a = result;

                (false, 4)
            }
            0x8B => {
                let carry = self.registers.get_c() as u8;
                let result = self
                    .registers
                    .a
                    .wrapping_add(self.registers.e)
                    .wrapping_add(carry);
                self.registers.set_z(result == 0);
                self.registers.set_n(false);
                let h_check = (self.registers.a & 0x0F) + (self.registers.e & 0x0F) + carry;
                self.registers.set_h(h_check > 0x0F);
                let c_check =
                    (self.registers.a as u16) + (self.registers.e as u16) + (carry as u16);
                self.registers.set_c(c_check > 0xFF);
                self.registers.a = result;

                (false, 4)
            }
            0x8C => {
                let carry = self.registers.get_c() as u8;
                let result = self
                    .registers
                    .a
                    .wrapping_add(self.registers.h)
                    .wrapping_add(carry);
                self.registers.set_z(result == 0);
                self.registers.set_n(false);
                let h_check = (self.registers.a & 0x0F) + (self.registers.h & 0x0F) + carry;
                self.registers.set_h(h_check > 0x0F);
                let c_check =
                    (self.registers.a as u16) + (self.registers.h as u16) + (carry as u16);
                self.registers.set_c(c_check > 0xFF);
                self.registers.a = result;

                (false, 4)
            }
            0x8D => {
                let carry = self.registers.get_c() as u8;
                let result = self
                    .registers
                    .a
                    .wrapping_add(self.registers.l)
                    .wrapping_add(carry);
                self.registers.set_z(result == 0);
                self.registers.set_n(false);
                let h_check = (self.registers.a & 0x0F) + (self.registers.l & 0x0F) + carry;
                self.registers.set_h(h_check > 0x0F);
                let c_check =
                    (self.registers.a as u16) + (self.registers.l as u16) + (carry as u16);
                self.registers.set_c(c_check > 0xFF);
                self.registers.a = result;

                (false, 4)
            }
            0x8E => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    let carry = self.registers.get_c() as u8;
                    let result = self.registers.a.wrapping_add(*value).wrapping_add(carry);
                    self.registers.set_z(result == 0);
                    self.registers.set_n(false);
                    let h_check = (self.registers.a & 0x0F) + (*value & 0x0F) + carry;
                    self.registers.set_h(h_check > 0x0F);
                    let c_check = (self.registers.a as u16) + (*value as u16) + (carry as u16);
                    self.registers.set_c(c_check > 0xFF);
                    self.registers.a = result;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }

                (false, 8)
            }
            0x8F => {
                let carry = self.registers.get_c() as u8;
                let result = self
                    .registers
                    .a
                    .wrapping_add(self.registers.a)
                    .wrapping_add(carry);
                self.registers.set_z(result == 0);
                self.registers.set_n(false);
                let h_check = (self.registers.a & 0x0F) + (self.registers.a & 0x0F) + carry;
                self.registers.set_h(h_check > 0x0F);
                let c_check =
                    (self.registers.a as u16) + (self.registers.a as u16) + (carry as u16);
                self.registers.set_c(c_check > 0xFF);
                self.registers.a = result;

                (false, 4)
            }
            0x90 => {
                let result = self.registers.a.wrapping_sub(self.registers.b);
                self.registers.set_z(result == 0);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < (self.registers.b & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.b);
                self.registers.a = result;

                (false, 4)
            }
            0x91 => {
                let result = self.registers.a.wrapping_sub(self.registers.c);
                self.registers.set_z(result == 0);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < (self.registers.c & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.c);
                self.registers.a = result;

                (false, 4)
            }
            0x92 => {
                let result = self.registers.a.wrapping_sub(self.registers.d);
                self.registers.set_z(result == 0);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < (self.registers.d & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.d);
                self.registers.a = result;

                (false, 4)
            }
            0x93 => {
                let result = self.registers.a.wrapping_sub(self.registers.e);
                self.registers.set_z(result == 0);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < (self.registers.e & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.e);
                self.registers.a = result;

                (false, 4)
            }
            0x94 => {
                let result = self.registers.a.wrapping_sub(self.registers.h);
                self.registers.set_z(result == 0);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < (self.registers.h & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.h);
                self.registers.a = result;

                (false, 4)
            }
            0x95 => {
                let result = self.registers.a.wrapping_sub(self.registers.l);
                self.registers.set_z(result == 0);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < (self.registers.l & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.l);
                self.registers.a = result;

                (false, 4)
            }
            0x96 => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    let result = self.registers.a.wrapping_sub(*value);
                    self.registers.set_z(result == 0);
                    self.registers.set_n(true);
                    self.registers
                        .set_h((self.registers.a & 0x0F) < (*value & 0x0F));
                    self.registers.set_c(self.registers.a < *value);
                    self.registers.a = result;
                }

                (false, 8)
            }
            0x97 => {
                self.registers.set_z(true);
                self.registers.set_n(true);
                self.registers.set_h(false);
                self.registers.set_c(false);
                self.registers.a = 0;

                (false, 4)
            }
            0x98 => {
                let carry = self.registers.get_c() as u8;
                let result = self
                    .registers
                    .a
                    .wrapping_sub(self.registers.b)
                    .wrapping_sub(carry);

                self.registers.set_z(result == 0);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < ((self.registers.b & 0x0F) + carry));
                self.registers
                    .set_c((self.registers.a as u16) < (self.registers.b as u16 + carry as u16));
                self.registers.a = result;

                (false, 4)
            }
            0x99 => {
                let carry = self.registers.get_c() as u8;
                let result = self
                    .registers
                    .a
                    .wrapping_sub(self.registers.c)
                    .wrapping_sub(carry);

                self.registers.set_z(result == 0);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < ((self.registers.c & 0x0F) + carry));
                self.registers
                    .set_c((self.registers.a as u16) < (self.registers.c as u16 + carry as u16));
                self.registers.a = result;

                (false, 4)
            }
            0x9A => {
                let carry = self.registers.get_c() as u8;
                let result = self
                    .registers
                    .a
                    .wrapping_sub(self.registers.d)
                    .wrapping_sub(carry);

                self.registers.set_z(result == 0);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < ((self.registers.d & 0x0F) + carry));
                self.registers
                    .set_c((self.registers.a as u16) < (self.registers.d as u16 + carry as u16));
                self.registers.a = result;

                (false, 4)
            }
            0x9B => {
                let carry = self.registers.get_c() as u8;
                let result = self
                    .registers
                    .a
                    .wrapping_sub(self.registers.e)
                    .wrapping_sub(carry);

                self.registers.set_z(result == 0);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < ((self.registers.e & 0x0F) + carry));
                self.registers
                    .set_c((self.registers.a as u16) < (self.registers.e as u16 + carry as u16));
                self.registers.a = result;

                (false, 4)
            }
            0x9C => {
                let carry = self.registers.get_c() as u8;
                let result = self
                    .registers
                    .a
                    .wrapping_sub(self.registers.h)
                    .wrapping_sub(carry);

                self.registers.set_z(result == 0);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < ((self.registers.h & 0x0F) + carry));
                self.registers
                    .set_c((self.registers.a as u16) < (self.registers.h as u16 + carry as u16));
                self.registers.a = result;

                (false, 4)
            }
            0x9D => {
                let carry = self.registers.get_c() as u8;
                let result = self
                    .registers
                    .a
                    .wrapping_sub(self.registers.l)
                    .wrapping_sub(carry);

                self.registers.set_z(result == 0);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < ((self.registers.l & 0x0F) + carry));
                self.registers
                    .set_c((self.registers.a as u16) < (self.registers.l as u16 + carry as u16));
                self.registers.a = result;

                (false, 4)
            }
            0x9E => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    let carry = self.registers.get_c() as u8;
                    let result = self.registers.a.wrapping_sub(*value).wrapping_sub(carry);

                    self.registers.set_z(result == 0);
                    self.registers.set_n(true);
                    self.registers
                        .set_h((self.registers.a & 0x0F) < ((*value & 0x0F) + carry));
                    self.registers
                        .set_c((self.registers.a as u16) < (*value as u16 + carry as u16));
                    self.registers.a = result;
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }

                (false, 8)
            }
            0x9F => {
                let carry = self.registers.get_c() as u8;
                let result = 0u8.wrapping_sub(carry);

                self.registers.set_z(result == 0);
                self.registers.set_n(true);
                self.registers.set_h(carry != 0);
                self.registers.a = result;

                (false, 4)
            }
            0xA0 => {
                self.registers.a &= self.registers.b;
                self.registers.set_z(self.registers.a == 0);
                self.registers.set_n(false);
                self.registers.set_h(true);
                self.registers.set_c(false);

                (false, 4)
            }
            0xA1 => {
                self.registers.a &= self.registers.c;
                self.registers.set_z(self.registers.a == 0);
                self.registers.set_n(false);
                self.registers.set_h(true);
                self.registers.set_c(false);

                (false, 4)
            }
            0xA2 => {
                self.registers.a &= self.registers.d;
                self.registers.set_z(self.registers.a == 0);
                self.registers.set_n(false);
                self.registers.set_h(true);
                self.registers.set_c(false);

                (false, 4)
            }
            0xA3 => {
                self.registers.a &= self.registers.e;
                self.registers.set_z(self.registers.a == 0);
                self.registers.set_n(false);
                self.registers.set_h(true);
                self.registers.set_c(false);

                (false, 4)
            }
            0xA4 => {
                self.registers.a &= self.registers.h;
                self.registers.set_z(self.registers.a == 0);
                self.registers.set_n(false);
                self.registers.set_h(true);
                self.registers.set_c(false);

                (false, 4)
            }
            0xA5 => {
                self.registers.a &= self.registers.l;
                self.registers.set_z(self.registers.a == 0);
                self.registers.set_n(false);
                self.registers.set_h(true);
                self.registers.set_c(false);

                (false, 4)
            }
            0xA6 => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    self.registers.a &= *value;
                    self.registers.set_z(self.registers.a == 0);
                    self.registers.set_n(false);
                    self.registers.set_h(true);
                    self.registers.set_c(false);
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }

                (false, 8)
            }
            0xA7 => {
                self.registers.a &= self.registers.a;
                self.registers.set_z(self.registers.a == 0);
                self.registers.set_n(false);
                self.registers.set_h(true);
                self.registers.set_c(false);

                (false, 4)
            }
            0xA8 => {
                self.registers.a ^= self.registers.b;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xA9 => {
                self.registers.a ^= self.registers.c;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xAA => {
                self.registers.a ^= self.registers.d;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xAB => {
                self.registers.a ^= self.registers.e;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xAC => {
                self.registers.a ^= self.registers.h;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xAD => {
                self.registers.a ^= self.registers.l;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xAE => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    self.registers.a ^= value;
                    self.registers.set_z(self.registers.a == 0x00);
                    self.registers.set_n(false);
                    self.registers.set_h(false);
                    self.registers.set_c(false);
                }
                (false, 8)
            }
            0xAF => {
                self.registers.a ^= self.registers.a;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xB0 => {
                self.registers.a |= self.registers.b;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xB1 => {
                self.registers.a |= self.registers.c;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xB2 => {
                self.registers.a |= self.registers.d;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xB3 => {
                self.registers.a |= self.registers.e;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xB4 => {
                self.registers.a |= self.registers.h;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xB5 => {
                self.registers.a |= self.registers.l;
                self.registers.set_z(self.registers.a == 0x00);
                self.registers.set_n(false);
                self.registers.set_h(false);
                self.registers.set_c(false);
                (false, 4)
            }
            0xB6 => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    self.registers.a |= *value;
                    self.registers.set_z(self.registers.a == 0x00);
                    self.registers.set_n(false);
                    self.registers.set_h(false);
                    self.registers.set_c(false);
                } else {
                    eprintln!(
                        "Failed to access [HL] at HL {:#06X}",
                        self.registers.get_hl()
                    );
                }

                (false, 8)
            }
            0xB7 => {
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
                self.registers
                    .set_h((self.registers.a & 0x0F) < (self.registers.b & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.b);

                (false, 4)
            }
            0xB9 => {
                self.registers.set_z(self.registers.a == self.registers.c);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < (self.registers.c & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.c);

                (false, 4)
            }
            0xBA => {
                self.registers.set_z(self.registers.a == self.registers.d);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < (self.registers.d & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.d);

                (false, 4)
            }
            0xBB => {
                self.registers.set_z(self.registers.a == self.registers.e);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < (self.registers.e & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.e);

                (false, 4)
            }
            0xBC => {
                self.registers.set_z(self.registers.a == self.registers.h);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < (self.registers.h & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.h);

                (false, 4)
            }
            0xBD => {
                self.registers.set_z(self.registers.a == self.registers.l);
                self.registers.set_n(true);
                self.registers
                    .set_h((self.registers.a & 0x0F) < (self.registers.l & 0x0F));
                self.registers.set_c(self.registers.a < self.registers.l);

                (false, 4)
            }
            0xBE => {
                if let Some(value) = memory.get(self.registers.get_hl() as usize) {
                    self.registers.set_z(self.registers.a == *value);
                    self.registers.set_n(true);
                    self.registers
                        .set_h((self.registers.a & 0x0F) < (*value & 0x0F));
                    self.registers.set_c(self.registers.a < *value);
                } else {
                    eprintln!("Failed to get value at HL {:#06X}", self.registers.get_hl());
                }
                (false, 8)
            }
            0xBF => {
                self.registers.set_z(true);
                self.registers.set_n(true);
                self.registers.set_h(false);
                self.registers.set_c(false);

                (false, 4)
            }
            0xC0 => {
                if !self.registers.get_z() {
                    if let Some(low) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        if let Some(high) = memory.get(self.registers.sp as usize) {
                            self.registers.sp = self.registers.sp.wrapping_add(1);
                            let return_address = ((*high as u16) << 8) | *low as u16;
                            self.registers.pc = return_address;
                            (true, 20)
                        } else {
                            eprintln!(
                                "Failed to get high value of return address at PC {:#06X}",
                                self.registers.pc
                            );
                            (false, 8)
                        }
                    } else {
                        eprintln!(
                            "Failed to get low value of return address at PC {:#06X}",
                            self.registers.pc
                        );
                        (false, 8)
                    }
                } else {
                    (false, 8)
                }
            }
            0xC1 => {
                if let Some(low) = memory.get(self.registers.sp as usize) {
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        self.registers.set_bc(((*high as u16) << 8) | *low as u16);
                    } else {
                        eprintln!(
                            "Failed to get high value of jump address at PC {:#06X}",
                            self.registers.pc
                        );
                    }
                } else {
                    eprintln!(
                        "Failed to get low value of jump address at PC {:#06X}",
                        self.registers.pc
                    );
                }
                (false, 12)
            }
            0xC2 => {
                if !self.registers.get_z() {
                    self.jump_absolute(memory);
                    (true, 16)
                } else {
                    self.registers.pc = self.registers.pc.wrapping_add(2);
                    (false, 12)
                }
            }
            0xC3 => {
                self.jump_absolute(memory);
                (true, 16)
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
                            memory.write_memory(
                                self.registers.sp as usize,
                                (return_address >> 8) as u8,
                            );
                            self.registers.sp = self.registers.sp.wrapping_sub(1);
                            memory.write_memory(self.registers.sp as usize, return_address as u8);
                            self.registers.pc = address;
                            (true, 24)
                        } else {
                            eprintln!(
                                "Failed to get high value of call address at PC {:#06X}",
                                self.registers.pc
                            );
                            (false, 24)
                        }
                    } else {
                        eprintln!(
                            "Failed to get low value of call address at PC {:#06X}",
                            self.registers.pc
                        );
                        (false, 24)
                    }
                } else {
                    self.registers.pc = self.registers.pc.wrapping_add(2);
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
                } else {
                    eprintln!("Failed to get n8 at PC {:#06X}", self.registers.pc)
                }
                (false, 8)
            }
            0xC7 => {
                let return_address = self.registers.pc.wrapping_add(1);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, (return_address >> 8) as u8);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, return_address as u8);
                self.registers.pc = 0x0000;

                (true, 16)
            }
            0xC8 => {
                if self.registers.get_z() {
                    if let Some(low) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        if let Some(high) = memory.get(self.registers.sp as usize) {
                            self.registers.sp = self.registers.sp.wrapping_add(1);
                            let return_address = ((*high as u16) << 8) | *low as u16;
                            self.registers.pc = return_address;
                            (true, 20)
                        } else {
                            eprintln!(
                                "Failed to get high value of return address at PC {:#06X}",
                                self.registers.pc
                            );
                            (false, 8)
                        }
                    } else {
                        eprintln!(
                            "Failed to get low value of return address at PC {:#06X}",
                            self.registers.pc
                        );
                        (false, 8)
                    }
                } else {
                    (false, 8)
                }
            }
            0xC9 => {
                if let Some(low) = memory.get(self.registers.sp as usize) {
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        let return_address = ((*high as u16) << 8) | *low as u16;
                        self.registers.pc = return_address;
                        (true, 16)
                    } else {
                        eprintln!(
                            "Failed to get high value of return address at PC {:#06X}",
                            self.registers.pc
                        );
                        (false, 16)
                    }
                } else {
                    eprintln!(
                        "Failed to get low value of return address at PC {:#06X}",
                        self.registers.pc
                    );
                    (false, 16)
                }
            }
            0xCA => {
                if self.registers.get_z() {
                    self.jump_absolute(memory);
                    (true, 16)
                } else {
                    self.registers.pc = self.registers.pc.wrapping_add(2);
                    (false, 12)
                }
            }
            0xCB => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                let mut cycles = 4;
                if let Some(prefix_opcode) = memory.get(self.registers.pc as usize) {
                    cycles += self.process_prefix(*prefix_opcode, memory);
                } else {
                    eprintln!(
                        "Failed to access prefix_opcode at PC {:#06X}",
                        self.registers.pc
                    );
                }
                (false, cycles)
            }
            0xCC => {
                if self.registers.get_z() {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(low) = memory.get(self.registers.pc as usize) {
                        self.registers.pc = self.registers.pc.wrapping_add(1);
                        if let Some(high) = memory.get(self.registers.pc as usize) {
                            let address = ((*high as u16) << 8) | *low as u16;
                            let return_address = self.registers.pc.wrapping_add(1);
                            self.registers.sp = self.registers.sp.wrapping_sub(1);
                            memory.write_memory(
                                self.registers.sp as usize,
                                (return_address >> 8) as u8,
                            );
                            self.registers.sp = self.registers.sp.wrapping_sub(1);
                            memory.write_memory(self.registers.sp as usize, return_address as u8);
                            self.registers.pc = address;
                            (true, 24)
                        } else {
                            eprintln!(
                                "Failed to get high value of call address at PC {:#06X}",
                                self.registers.pc
                            );
                            (false, 24)
                        }
                    } else {
                        eprintln!(
                            "Failed to get low value of call address at PC {:#06X}",
                            self.registers.pc
                        );
                        (false, 24)
                    }
                } else {
                    self.registers.pc = self.registers.pc.wrapping_add(2);
                    (false, 12)
                }
            }
            0xCD => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(low) = memory.get(self.registers.pc as usize) {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.pc as usize) {
                        let address = ((*high as u16) << 8) | *low as u16;
                        let return_address = self.registers.pc.wrapping_add(1);
                        self.registers.sp = self.registers.sp.wrapping_sub(1);
                        memory
                            .write_memory(self.registers.sp as usize, (return_address >> 8) as u8);
                        self.registers.sp = self.registers.sp.wrapping_sub(1);
                        memory.write_memory(self.registers.sp as usize, return_address as u8);
                        self.registers.pc = address;
                        (true, 24)
                    } else {
                        eprintln!(
                            "Failed to get high value of call address at PC {:#06X}",
                            self.registers.pc
                        );
                        (false, 24)
                    }
                } else {
                    eprintln!(
                        "Failed to get low value of call address at PC {:#06X}",
                        self.registers.pc
                    );
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
                } else {
                    eprintln!("Failed to retrieve value at PC {:#06X}", self.registers.pc);
                }
                (false, 8)
            }
            0xCF => {
                let return_address = self.registers.pc.wrapping_add(1);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, (return_address >> 8) as u8);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, return_address as u8);
                self.registers.pc = 0x0008;

                (true, 16)
            }
            0xD0 => {
                if !self.registers.get_c() {
                    if let Some(low) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        if let Some(high) = memory.get(self.registers.sp as usize) {
                            self.registers.sp = self.registers.sp.wrapping_add(1);
                            let return_address = ((*high as u16) << 8) | *low as u16;
                            self.registers.pc = return_address;
                            (true, 20)
                        } else {
                            eprintln!(
                                "Failed to get high value of return address at PC {:#06X}",
                                self.registers.pc
                            );
                            (false, 8)
                        }
                    } else {
                        eprintln!(
                            "Failed to get low value of return address at PC {:#06X}",
                            self.registers.pc
                        );
                        (false, 8)
                    }
                } else {
                    (false, 8)
                }
            }
            0xD1 => {
                if let Some(low) = memory.get(self.registers.sp as usize) {
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        self.registers.set_de(((*high as u16) << 8) | *low as u16);
                    } else {
                        eprintln!(
                            "Failed to get high value of jump address at PC {:#06X}",
                            self.registers.pc
                        );
                    }
                } else {
                    eprintln!(
                        "Failed to get low value of jump address at PC {:#06X}",
                        self.registers.pc
                    );
                }
                (false, 12)
            }
            0xD2 => {
                if !self.registers.get_c() {
                    self.jump_absolute(memory);
                    (true, 16)
                } else {
                    self.registers.pc = self.registers.pc.wrapping_add(2);
                    (false, 12)
                }
            }
            0xD3 => {
                panic!(
                    "This opcode: {:#04X} doesn't exist, at PC {:#06X}",
                    opcode, self.registers.pc
                );
            }
            0xD4 => {
                if !self.registers.get_c() {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(low) = memory.get(self.registers.pc as usize) {
                        self.registers.pc = self.registers.pc.wrapping_add(1);
                        if let Some(high) = memory.get(self.registers.pc as usize) {
                            let address = ((*high as u16) << 8) | *low as u16;
                            let return_address = self.registers.pc.wrapping_add(1);
                            self.registers.sp = self.registers.sp.wrapping_sub(1);
                            memory.write_memory(
                                self.registers.sp as usize,
                                (return_address >> 8) as u8,
                            );
                            self.registers.sp = self.registers.sp.wrapping_sub(1);
                            memory.write_memory(self.registers.sp as usize, return_address as u8);
                            self.registers.pc = address;
                            (true, 24)
                        } else {
                            eprintln!(
                                "Failed to get high value of call address at PC {:#06X}",
                                self.registers.pc
                            );
                            (false, 24)
                        }
                    } else {
                        eprintln!(
                            "Failed to get low value of call address at PC {:#06X}",
                            self.registers.pc
                        );
                        (false, 24)
                    }
                } else {
                    self.registers.pc = self.registers.pc.wrapping_add(2);
                    (false, 12)
                }
            }
            0xD5 => {
                let de = self.registers.get_de();
                let low = de as u8;
                let high = (de >> 8) as u8;
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, high);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, low);
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
                } else {
                    eprintln!("Failed to get n8 at PC {:#06X}", self.registers.pc)
                }
                (false, 8)
            }
            0xD7 => {
                let return_address = self.registers.pc.wrapping_add(1);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, (return_address >> 8) as u8);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, return_address as u8);
                self.registers.pc = 0x0010;
                (true, 16)
            }
            0xD8 => {
                if self.registers.get_c() {
                    if let Some(low) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        if let Some(high) = memory.get(self.registers.sp as usize) {
                            self.registers.sp = self.registers.sp.wrapping_add(1);
                            let return_address = ((*high as u16) << 8) | *low as u16;
                            self.registers.pc = return_address;
                            (true, 20)
                        } else {
                            eprintln!(
                                "Failed to get high value of return address at PC {:#06X}",
                                self.registers.pc
                            );
                            (false, 8)
                        }
                    } else {
                        eprintln!(
                            "Failed to get low value of return address at PC {:#06X}",
                            self.registers.pc
                        );
                        (false, 8)
                    }
                } else {
                    (false, 8)
                }
            }
            0xD9 => {
                if let Some(low) = memory.get(self.registers.sp as usize) {
                    self.registers.sp = self.registers.sp.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.sp as usize) {
                        self.registers.sp = self.registers.sp.wrapping_add(1);
                        let return_address = ((*high as u16) << 8) | *low as u16;
                        self.registers.pc = return_address;
                        self.ime_pending = 1;
                        (true, 16)
                    } else {
                        eprintln!(
                            "Failed to get high value of return address at PC {:#06X}",
                            self.registers.pc
                        );
                        (false, 16)
                    }
                } else {
                    eprintln!(
                        "Failed to get low value of return address at PC {:#06X}",
                        self.registers.pc
                    );
                    (false, 16)
                }
            }
            0xDA => {
                if self.registers.get_c() {
                    self.jump_absolute(memory);
                    (true, 16)
                } else {
                    self.registers.pc = self.registers.pc.wrapping_add(2);
                    (false, 12)
                }
            }
            0xDB => {
                panic!(
                    "This opcode: {:#04X} doesn't exist, at PC {:#06X}",
                    opcode, self.registers.pc
                );
            }
            0xDC => {
                if self.registers.get_c() {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(low) = memory.get(self.registers.pc as usize) {
                        self.registers.pc = self.registers.pc.wrapping_add(1);
                        if let Some(high) = memory.get(self.registers.pc as usize) {
                            let address = ((*high as u16) << 8) | *low as u16;
                            let return_address = self.registers.pc.wrapping_add(1);
                            self.registers.sp = self.registers.sp.wrapping_sub(1);
                            memory.write_memory(
                                self.registers.sp as usize,
                                (return_address >> 8) as u8,
                            );
                            self.registers.sp = self.registers.sp.wrapping_sub(1);
                            memory.write_memory(self.registers.sp as usize, return_address as u8);
                            self.registers.pc = address;
                            (true, 24)
                        } else {
                            eprintln!(
                                "Failed to get high value of call address at PC {:#06X}",
                                self.registers.pc
                            );
                            (false, 24)
                        }
                    } else {
                        eprintln!(
                            "Failed to get low value of call address at PC {:#06X}",
                            self.registers.pc
                        );
                        (false, 24)
                    }
                } else {
                    self.registers.pc = self.registers.pc.wrapping_add(2);
                    (false, 12)
                }
            }
            0xDD => {
                panic!(
                    "This opcode: {:#04X} doesn't exist, at PC {:#06X}",
                    opcode, self.registers.pc
                );
            }
            0xDE => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(value) = memory.get(self.registers.pc as usize) {
                    let a = self.registers.a;
                    let c = self.registers.get_c() as u8;
                    let total_sub = *value as u16 + c as u16;
                    self.registers.a = a.wrapping_sub(*value).wrapping_sub(c);

                    self.registers.set_z(self.registers.a == 0);
                    self.registers.set_n(true);
                    self.registers
                        .set_h(((a & 0x0F) as u16) < ((*value & 0x0F) as u16 + c as u16));
                    self.registers.set_c((a as u16) < total_sub);
                } else {
                    eprintln!("Failed to get n8 at PC {:#06X}", self.registers.pc);
                }
                (false, 8)
            }
            0xDF => {
                let return_address = self.registers.pc.wrapping_add(1);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, (return_address >> 8) as u8);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, return_address as u8);
                self.registers.pc = 0x0018;
                (true, 16)
            }
            0xE0 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(value) = memory.get(self.registers.pc as usize) {
                    let address = 0xFF00 | *value as u16;
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
                    } else {
                        eprintln!(
                            "Failed to get high value of pop at PC {:#06X}",
                            self.registers.pc
                        );
                    }
                } else {
                    eprintln!(
                        "Failed to get low value of pop at PC {:#06X}",
                        self.registers.pc
                    );
                }
                (false, 12)
            }
            0xE2 => {
                let address = 0xFF00 | self.registers.c as u16;
                memory.write_memory(address as usize, self.registers.a);
                (false, 8)
            }
            0xE3 => {
                panic!(
                    "This opcode: {:#04X} doesn't exist, at PC {:#06X}",
                    opcode, self.registers.pc
                );
            }
            0xE4 => {
                panic!(
                    "This opcode: {:#04X} doesn't exist, at PC {:#06X}",
                    opcode, self.registers.pc
                );
            }
            0xE5 => {
                let hl = self.registers.get_hl();
                let low = hl as u8;
                let high = (hl >> 8) as u8;
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, high);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, low);
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
                } else {
                    eprintln!("Failed to get n8 at PC {:#06X}", self.registers.pc)
                }
                (false, 8)
            }
            0xE7 => {
                let return_address = self.registers.pc.wrapping_add(1);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, (return_address >> 8) as u8);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, return_address as u8);
                self.registers.pc = 0x0020;
                (true, 16)
            }
            0xE8 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(e8) = memory.get(self.registers.pc as usize) {
                    let offset = *e8 as i8 as i16;
                    let original_sp = self.registers.sp;
                    self.registers.sp = original_sp.wrapping_add_signed(offset);

                    self.registers.set_z(false);
                    self.registers.set_n(false);
                    let sp_lo = (original_sp & 0xFF) as u8;
                    let e8_u8 = *e8;
                    let sum = sp_lo as u16 + e8_u8 as u16;
                    self.registers.set_h((sp_lo & 0x0F) + (e8_u8 & 0x0F) > 0x0F);
                    self.registers.set_c(sum > 0xFF);
                } else {
                    eprintln!("Failed to get e8 at PC {:#06X}", self.registers.pc);
                }
                (false, 16)
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
                    } else {
                        eprintln!(
                            "Failed to get high value of a16 at PC {:#06X}",
                            self.registers.pc
                        );
                    }
                } else {
                    eprintln!(
                        "Failed to get low value of a16 at PC {:#06X}",
                        self.registers.pc
                    );
                }
                (false, 16)
            }
            0xEB => {
                panic!(
                    "This opcode: {:#04X} doesn't exist, at PC {:#06X}",
                    opcode, self.registers.pc
                );
            }
            0xEC => {
                panic!(
                    "This opcode: {:#04X} doesn't exist, at PC {:#06X}",
                    opcode, self.registers.pc
                );
            }
            0xED => {
                panic!(
                    "This opcode: {:#04X} doesn't exist, at PC {:#06X}",
                    opcode, self.registers.pc
                );
            }
            0xEE => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(value) = memory.get(self.registers.pc as usize) {
                    self.registers.a ^= *value;
                    self.registers.set_z(self.registers.a == 0x00);
                    self.registers.set_n(false);
                    self.registers.set_h(false);
                    self.registers.set_c(false);
                } else {
                    eprintln!("Failed to get n8 at PC {:#06X}", self.registers.pc)
                }
                (false, 8)
            }
            0xEF => {
                let return_address = self.registers.pc.wrapping_add(1);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, (return_address >> 8) as u8);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, return_address as u8);
                self.registers.pc = 0x0028;
                (true, 16)
            }
            0xF0 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(value) = memory.get(self.registers.pc as usize) {
                    let address = 0xFF00 | *value as u16;
                    if let Some(goal_value) = memory.get(address as usize) {
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
                    } else {
                        eprintln!(
                            "Failed to get high value of pop at PC {:#06X}",
                            self.registers.pc
                        );
                    }
                } else {
                    eprintln!(
                        "Failed to get low value of pop at PC {:#06X}",
                        self.registers.pc
                    );
                }
                (false, 12)
            }
            0xF2 => {
                let address = 0xFF00 | self.registers.c as u16;
                if let Some(value) = memory.get(address as usize) {
                    self.registers.a = *value;
                } else {
                    eprintln!("Failed to get value at address = {:#06X}", address);
                }

                (false, 8)
            }
            0xF3 => {
                self.ime = false;
                self.ime_pending = 0;
                (false, 4)
            }
            0xF4 => {
                panic!(
                    "This opcode: {:#04X} doesn't exist, at PC {:#06X}",
                    opcode, self.registers.pc
                );
            }
            0xF5 => {
                let af = self.registers.get_af();
                let low = af as u8;
                let high = (af >> 8) as u8;
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, high);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, low);
                (false, 16)
            }
            0xF6 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(value) = memory.get(self.registers.pc as usize) {
                    self.registers.a |= *value;
                    self.registers.set_z(self.registers.a == 0x00);
                    self.registers.set_n(false);
                    self.registers.set_h(false);
                    self.registers.set_c(false);
                } else {
                    eprintln!("Failed to access n8 at PC {:#06X}", self.registers.pc);
                }

                (false, 8)
            }
            0xF7 => {
                let return_address = self.registers.pc.wrapping_add(1);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, (return_address >> 8) as u8);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, return_address as u8);
                self.registers.pc = 0x0030;
                (true, 16)
            }
            0xF8 => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(offset) = memory.get(self.registers.pc as usize) {
                    let sp = self.registers.sp;
                    let offset = *offset as i8 as i16 as u16;
                    let result = sp.wrapping_add(offset);

                    self.registers.set_z(false);
                    self.registers.set_n(false);
                    self.registers.set_h(((sp & 0x0F) + (offset & 0x0F)) > 0x0F);
                    self.registers.set_c(((sp & 0xFF) + (offset & 0xFF)) > 0xFF);
                    self.registers.set_hl(result);
                } else {
                    eprintln!("Failed to get offset at PC = {:#06X}", self.registers.pc);
                }
                (false, 12)
            }
            0xF9 => {
                self.registers.sp = self.registers.get_hl();
                (false, 8)
            }
            0xFA => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(low) = memory.get(self.registers.pc as usize) {
                    self.registers.pc = self.registers.pc.wrapping_add(1);
                    if let Some(high) = memory.get(self.registers.pc as usize) {
                        let address = ((*high as u16) << 8) | *low as u16;
                        if let Some(value_goal) = memory.get(address as usize) {
                            self.registers.a = *value_goal;
                        } else {
                            eprintln!("Failed to get value at address = {:#06X}", address);
                        }
                    } else {
                        eprintln!(
                            "Failed to get high value of a16 at PC {:#06X}",
                            self.registers.pc
                        );
                    }
                } else {
                    eprintln!(
                        "Failed to get low value of a16 at PC {:#06X}",
                        self.registers.pc
                    );
                }
                (false, 16)
            }
            0xFB => {
                self.ime_pending = 2;
                (false, 4)
            }
            0xFC => {
                panic!(
                    "This opcode: {:#04X} doesn't exist, at PC {:#06X}",
                    opcode, self.registers.pc
                );
            }
            0xFD => {
                panic!(
                    "This opcode: {:#04X} doesn't exist, at PC {:#06X}",
                    opcode, self.registers.pc
                );
            }
            0xFE => {
                self.registers.pc = self.registers.pc.wrapping_add(1);
                if let Some(n8) = memory.get(self.registers.pc as usize) {
                    let a = self.registers.a;
                    self.registers.set_z(a == *n8);
                    self.registers.set_n(true);
                    self.registers.set_h((a & 0x0F) < (*n8 & 0x0F));
                    self.registers.set_c(a < *n8);
                } else {
                    eprintln!("Failed to get high n8 at PC {:#06X}", self.registers.pc);
                }
                (false, 8)
            }
            0xFF => {
                let return_address = self.registers.pc.wrapping_add(1);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, (return_address >> 8) as u8);
                self.registers.sp = self.registers.sp.wrapping_sub(1);
                memory.write_memory(self.registers.sp as usize, return_address as u8);
                self.registers.pc = 0x0038;
                (true, 16)
            }
            _ => unreachable!(),
        }
    }

    fn add_a_r8(&mut self, value: u8) {
        let a = self.registers.a;
        let result = a.wrapping_add(value);
        self.registers.set_z(result == 0);
        self.registers.set_n(false);
        self.registers.set_h((a & 0x0F) + (value & 0x0F) > 0x0F);
        self.registers.set_c((a as u16) + (value as u16) > 0xFF);
        self.registers.a = result;
    }

    fn ld_r8_n8(&mut self, memory: &mut Memory) -> u8 {
        self.registers.pc = self.registers.pc.wrapping_add(1);
        if let Some(n8) = memory.get(self.registers.pc as usize) {
            *n8
        } else {
            eprintln!("Failed to get imm8 at PC {:#06X}", self.registers.pc);
            0
        }
    }

    fn dec_r8(&mut self, reg: u8) -> u8 {
        let original = reg;
        let new_value = reg.wrapping_sub(1);
        self.registers.set_z(new_value == 0);
        self.registers.set_n(true);
        self.registers.set_h((original & 0x0F) == 0x00);
        new_value
    }

    fn inc_r8(&mut self, reg: u8) -> u8 {
        let original = reg;
        let new_value = reg.wrapping_add(1);
        self.registers.set_z(new_value == 0);
        self.registers.set_n(false);
        self.registers.set_h((original & 0x0F) == 0x0F);
        new_value
    }

    fn add_hl_r16(&mut self, opcode: u8) {
        let hl = self.registers.get_hl();
        let r16 = match (opcode & 0x30) >> 4 {
            0 => self.registers.get_bc(),
            1 => self.registers.get_de(),
            2 => self.registers.get_hl(),
            3 => self.registers.sp,
            _ => unreachable!()
        };
        let sum = hl as u32 + r16 as u32;
        let new_hl = sum as u16;
        self.registers.set_hl(new_hl);
        self.registers.set_n(false);
        self.registers.set_h((hl & 0x0FFF) + (r16 & 0x0FFF) > 0x0FFF);
        self.registers.set_c(sum > 0xFFFF);
    }

    fn ld_r16_n16(&mut self, memory: &mut Memory, opcode: u8) {
        self.registers.pc = self.registers.pc.wrapping_add(1);
        if let Some(low) = memory.get(self.registers.pc as usize) {
            self.registers.pc = self.registers.pc.wrapping_add(1);
            if let Some(high) = memory.get(self.registers.pc as usize) {
                let immediate = ((*high as u16) << 8) | *low as u16;
                match (opcode & 0x30) >> 4 {
                    0 => self.registers.set_bc(immediate),
                    1 => self.registers.set_de(immediate),
                    2 => self.registers.set_hl(immediate),
                    3 => self.registers.sp = immediate,
                    _ => unreachable!(),
                }
            } else {
                eprintln!(
                    "Failed to get high value of immediate at PC {:#06X}",
                    self.registers.pc
                );
            }
        } else {
            eprintln!(
                "Failed to get low value of immediate at PC {:#06X}",
                self.registers.pc
            );
        }
    }

    fn jump_absolute(&mut self, memory: &mut Memory) {
        self.registers.pc = self.registers.pc.wrapping_add(1);
        if let Some(low) = memory.get(self.registers.pc as usize) {
            self.registers.pc = self.registers.pc.wrapping_add(1);
            if let Some(high) = memory.get(self.registers.pc as usize) {
                let address = ((*high as u16) << 8) | *low as u16;
                self.registers.pc = address;
            } else {
                eprintln!(
                    "Failed to get high value of jump address at PC {:#06X}",
                    self.registers.pc
                );
            }
        } else {
            eprintln!(
                "Failed to get low value of jump address at PC {:#06X}",
                self.registers.pc
            );
        }
    }

    fn jump_relative(&mut self, memory: &mut Memory) {
        if let Some(offset) = memory.get(self.registers.pc as usize) {
            self.registers.pc = self.registers.pc.wrapping_add_signed(*offset as i8 as i16);
        } else {
            eprintln!(
                "Failed to get offset for jump at PC {:#06X}",
                self.registers.pc
            );
        }
    }

    fn process_prefix(&mut self, prefix: u8, memory: &mut Memory) -> u64 {
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
            0x20 => (value << 1, (value >> 7) & 1),           // SLA
            0x28 => ((value as i8 >> 1) as u8, value & 1),    // SRA (arithmetic shift)
            0x30 => (value.rotate_left(4), 0),                // SWAP
            0x38 => (value >> 1, value & 1),                  // SRL
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
            6 => (
                *memory
                    .get(self.registers.get_hl() as usize)
                    .unwrap_or_else(|| {
                        panic!("Invalid HL address {:#06X}", self.registers.get_hl())
                    }),
                4,
            ),
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
