use crate::components::registers::Registers;
use crate::io;

pub struct Cpu {
    registers: Registers,
    rom: Vec<u8>,
    debug: bool
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            registers: Registers::default(),
            rom: Vec::new(),
            debug: false
        }
    }
    
    pub fn cartridge_to_rom(&mut self, filename: String) {
        self.rom = io::cartridge_reader::read_cartridge(filename)
    }
    
    pub fn toggle_debug(&mut self) {
        self.debug = !self.debug
    }
    
    pub fn start(&mut self) {
        for _i in 0..1000 {
            if let Some(opcode) = self.rom.get(self.registers.pc as usize) {
                self.process_opcode(*opcode);
                self.registers.pc += 1;
            } else {
                panic!("Tried to access address outside of ROM")
            }
        }
    }
    
    fn process_opcode(&mut self, opcode: u8) {
        match opcode { 
            0x00 => {
                if self.debug {
                    println!("Opcode: {:#04X} NOP, at PC {:#06X}", opcode, self.registers.pc);
                }
            }
            _ => {
                eprintln!("Unimplemented opcode: {:#04X}, at PC {:#06X}", opcode, self.registers.pc);
            }
        }
    }
}