use crate::cpu::registers::Registers;

mod registers;

pub struct Cpu {
    registers: Registers,
    pc: u16
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: Registers::default(),
            pc: 0x100
        }
    }
}