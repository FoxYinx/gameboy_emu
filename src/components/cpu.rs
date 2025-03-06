use crate::components::registers::Registers;

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