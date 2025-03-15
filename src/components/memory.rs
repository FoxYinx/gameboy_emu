use crate::io::serialoutput::SerialOutput;

pub struct Memory {
    memory: [u8; 0x10000],
    serial_output: SerialOutput,
    cycles_div: u64,
    cycles_tima: u64,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            memory: [0; 0x10000],
            serial_output: SerialOutput::new(),
            cycles_div: 0,
            cycles_tima: 0,
        }
    }

    pub fn get(&self, index: usize) -> Option<&u8> {
        self.memory.get(index)
    }

    pub fn write_memory(&mut self, address: usize, value: u8) {
        match address {
            0xC000..=0xDDFF => {
                self.memory[address] = value;
                self.memory[address + 0x2000] = value;
            }
            0xFEA0..=0xFEFF => {
                // Do nothing
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
        self.memory[0x0000..cartridge_data.len()].copy_from_slice(cartridge_data);
    }

    pub fn get_serial_output(&self) -> &SerialOutput {
        &self.serial_output
    }
}
