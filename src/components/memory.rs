pub struct Memory {
    memory: [u8; 0x10000]
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            memory: [0; 0x10000]
        }
    }
    
    pub fn get(&self, index: usize) -> Option<&u8> {
        self.memory.get(index)
    }

    pub fn write_memory(&mut self, address: usize, value: u8) {
        if address == 0xFF02 {
            if value == 0x81 {
                let byte = self.memory[0xFF01];
                print!("{}", byte as char);
                self.memory[address] = 0x00;
            } else {
                self.memory[address] = value;
            }
        } else if (0xFEA0..=0xFEFF).contains(&address) {
            // Do nothing
        } else if (0xC000..=0xDDFF).contains(&address) {
            self.memory[address] = value;
            self.memory[address + 0x2000] = value;
        } else {
            self.memory[address] = value;
        }
    }

    pub fn write_cartridge(&mut self, cartridge_data: &[u8]) {
        self.memory[0x0000..cartridge_data.len()].copy_from_slice(cartridge_data);
    }
}