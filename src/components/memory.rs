use crate::components::memory::Mbc::{MBC0, MBC1, MBC2, MBC3, MBC5, MBC6, MBC7, MMM01};
use crate::io::cartridge_reader::read_cartridge;
use crate::io::serialoutput::SerialOutput;

pub struct Memory {
    memory: [u8; 0x10000],
    start_cartridge: [u8; 0x100],
    serial_output: SerialOutput,
    cycles_div: u64,
    cycles_tima: u64,
    mbc: Mbc,
    pub(crate) input_buffer: u8,
}

#[derive(PartialEq)]
enum Mbc {
    MBC0,
    MBC1,
    MBC2,
    MBC3,
    MBC5,
    MBC6,
    MBC7,
    MMM01
}

impl Memory {
    pub fn new() -> Self {
        let mut mem = Memory {
            memory: [0; 0x10000],
            start_cartridge: [0; 0x100],
            serial_output: SerialOutput::new(),
            cycles_div: 0,
            cycles_tima: 0,
            mbc: MBC0,
            input_buffer: 0xFF
        };

        mem.memory[0xFF00] = 0xCF; //P1
        mem.memory[0xFF01] = 0x00; //SB
        mem.memory[0xFF02] = 0x7E; //SC
        mem.memory[0xFF04] = 0xAB; //DIV
        mem.memory[0xFF05] = 0x00; //TIMA
        mem.memory[0xFF06] = 0x00; //TMA
        mem.memory[0xFF07] = 0xF8; //TAC
        mem.memory[0xFF0F] = 0xE1; //IF
        mem.memory[0xFF10] = 0x80; //NR10
        mem.memory[0xFF11] = 0xBF; //NR11
        mem.memory[0xFF12] = 0xF3; //NR12
        mem.memory[0xFF13] = 0xFF; //NR13
        mem.memory[0xFF14] = 0xBF; //NR14
        mem.memory[0xFF16] = 0x3F; //NR21
        mem.memory[0xFF17] = 0x00; //NR22
        mem.memory[0xFF18] = 0xFF; //NR23
        mem.memory[0xFF19] = 0xBF; //NR24
        mem.memory[0xFF1A] = 0x7F; //NR30
        mem.memory[0xFF1B] = 0xFF; //NR31
        mem.memory[0xFF1C] = 0x9F; //NR32
        mem.memory[0xFF1D] = 0xFF; //NR33
        mem.memory[0xFF1E] = 0xBF; //NR34
        mem.memory[0xFF20] = 0xFF; //NR41
        mem.memory[0xFF21] = 0x00; //NR42
        mem.memory[0xFF22] = 0x00; //NR43
        mem.memory[0xFF23] = 0xBF; //NR44
        mem.memory[0xFF24] = 0x77; //NR50
        mem.memory[0xFF25] = 0xF3; //NR51
        mem.memory[0xFF26] = 0xF1; //NR52
        mem.memory[0xFF40] = 0x91; //LCDC
        mem.memory[0xFF41] = 0x85; //STAT
        mem.memory[0xFF42] = 0x00; //SCY
        mem.memory[0xFF43] = 0x00; //SCX
        mem.memory[0xFF44] = 0x00; //LY
        mem.memory[0xFF45] = 0x00; //LYC
        mem.memory[0xFF46] = 0xFF; //DMA
        mem.memory[0xFF47] = 0xFC; //BGP
        mem.memory[0xFF48] = 0xFF; //OBP0
        mem.memory[0xFF49] = 0xFF; //OBP1
        mem.memory[0xFF4A] = 0x00; //WY
        mem.memory[0xFF4B] = 0x00; //WX
        mem.memory[0xFFFF] = 0x00; //IE

        mem
    }

    pub fn get(&self, index: usize) -> Option<&u8> {
        self.memory.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut u8> {
        self.memory.get_mut(index)
    }

    pub fn write_memory(&mut self, address: usize, value: u8) {
        match address {
            0x0000..0x8000 => {
                if self.mbc != MBC0 {
                    self.memory[address] = value;
                }
            }
            0xA000..0xC000 => {}
            0xC000..=0xDDFF => {
                self.memory[address] = value;
                self.memory[address + 0x2000] = value;
            }
            0xFEA0..=0xFEFF => {
                // Do nothing
            }
            0xFF00 => {
                let current_inputs = match (value & 0x30) >> 4 { 
                    0 => (self.input_buffer & 0x0F) | (self.input_buffer >> 4), //both selected
                    1 => self.input_buffer >> 4, //buttons selected
                    2 => self.input_buffer & 0x0F, //d-pad selected
                    3 => 0xF, //nothing selected
                    _ => 0xF
                };
                self.memory[address] = value | current_inputs;
            }
            0xFF02 => {
                if value == 0x81 {
                    let byte = self.memory[0xFF01];
                    self.serial_output.write_byte(byte);
                    print!("{}", byte as char);
                    self.memory[address] = 0x00;
                } else {
                    self.memory[address] = value;
                }
            }
            0xFF04 => {
                self.cycles_div = 0;
                self.cycles_tima = 0;
                self.memory[address] = 0x00;
            }
            0xFF46 => {
                let source_start = (value as u16) << 8;
                for i in 0..0xA0 {
                    let src = source_start + i;
                    let dest = 0xFE00 + i;
                    self.memory[dest as usize] = self.memory[src as usize];
                }
                self.memory[address] = value;
            }
            _ => {
                self.memory[address] = value;
            }
        }
    }

    pub fn update_timer(&mut self, cycles: u64) {
        self.cycles_div += cycles;

        while self.cycles_div >= 256 {
            self.memory[0xFF04] = self.memory[0xFF04].wrapping_add(1);
            self.cycles_div -= 256;
        }

        if self.tac_enabled() {
            let required_cycles = match self.get_tac_select() {
                0 => 1024,
                1 => 16,
                2 => 64,
                3 => 256,
                _ => unreachable!(),
            };

            self.cycles_tima += cycles;
            while self.cycles_tima >= required_cycles {
                let new_tima = self.memory[0xFF05].wrapping_add(1);
                if new_tima == 0 {
                    self.memory[0xFF0F] |= 0x04;
                    self.memory[0xFF05] = self.memory[0xFF06];
                } else {
                    self.memory[0xFF05] = new_tima;
                }
                self.cycles_tima -= required_cycles;
            }
        }
    }

    fn tac_enabled(&self) -> bool {
        self.memory[0xFF07] & 0x04 != 0
    }

    fn get_tac_select(&self) -> u8 {
        if let Some(tac) = self.memory.get(0xFF07) {
            *tac & 0x3
        } else {
            eprintln!("Failed to access register TAC at 0xFF07");
            0
        }
    }

    pub fn write_cartridge(&mut self, cartridge_data: &[u8]) {
        let rom = read_cartridge("resources/boot/dmg_boot.bin".to_string());

        let data_len = cartridge_data.len();
        self.start_cartridge
            .copy_from_slice(&cartridge_data[0x0000..=0x00FF]);

        self.memory[0x0000..=0x00FF].copy_from_slice(&rom);
        self.memory[0x0100..data_len].copy_from_slice(&cartridge_data[0x0100..data_len]);
    }

    pub fn disable_rom(&mut self) {
        self.memory[0x0000..=0x00FF].copy_from_slice(&self.start_cartridge);
    }

    pub fn get_serial_output(&self) -> &SerialOutput {
        &self.serial_output
    }

    pub(crate) fn select_mbc(&mut self, code: u8) {
        self.mbc = match code {
            0x00 => MBC0,
            0x01..=0x03 => MBC1,
            0x05..=0x06 => MBC2,
            0x0B..=0x0D => MMM01,
            0x0F..=0x13 => MBC3,
            0x19..=0x1E => MBC5,
            0x20 => MBC6,
            0x22 => MBC7,
            _ => MBC0
        };
    }
}
