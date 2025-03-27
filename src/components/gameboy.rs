use crate::components::cpu::CPU;
use crate::components::memory::Memory;
use crate::components::ppu::PPU;
use crate::io;
use crate::utils::hardware_identification::{
    cartridge_type_decoder, destination_decoder, ram_size_decoder, rom_size_decoder,
};
use crate::utils::licensee::{new_licensee_code_decryption, old_licensee_code_decryption};

pub struct Gameboy {
    cpu: CPU,
    pub(crate) ppu: PPU,
    memory: Memory,
    pub(crate) cycles: u64,
}

impl Gameboy {
    pub fn new() -> Self {
        Gameboy {
            cpu: CPU::new(),
            ppu: PPU::new(),
            memory: Memory::new(),
            cycles: 0,
        }
    }

    pub fn cartridge_to_rom(&mut self, filename: String) {
        println!("Loading ROM: {filename}");
        let cartridge_data = io::cartridge_reader::read_cartridge(filename);
        self.memory.write_cartridge(&cartridge_data);

        let title_bytes: Vec<u8> = (0x0134..=0x0143)
            .filter_map(|addr| self.memory.get(addr).copied())
            .collect();
        if let Ok(title) = String::from_utf8(title_bytes) {
            println!("Game Title: {}", title);
        } else {
            println!("Failed to read game title.");
        }

        let manufacturer_bytes: Vec<u8> = (0x013F..=0x0142)
            .filter_map(|addr| self.memory.get(addr).copied())
            .collect();
        if let Ok(code) = String::from_utf8(manufacturer_bytes) {
            println!("Manufacturer Code: {}", code);
        } else {
            println!("Failed to read manufacturer code.");
        }

        let cartridge_type = self.memory.get(0x0147).unwrap_or(&0);
        println!(
            "Hardware present: {}",
            cartridge_type_decoder(*cartridge_type)
        );

        let rom_size = self.memory.get(0x0148).unwrap_or(&0);
        println!("Rom size: {}", rom_size_decoder(*rom_size));

        let ram_size = self.memory.get(0x0149).unwrap_or(&0);
        println!("Ram size: {}", ram_size_decoder(*ram_size));

        let destination_code = self.memory.get(0x014A).unwrap_or(&0);
        println!("Destination: {}", destination_decoder(*destination_code));

        let old_licensee_code = self.memory.get(0x014B).unwrap_or(&0);
        if *old_licensee_code != 0x33 {
            println!(
                "Licensee: {}",
                old_licensee_code_decryption(*old_licensee_code)
            );
        } else {
            let new_licensee_bytes: Vec<u8> = (0x0144..=0x0145)
                .filter_map(|addr| self.memory.get(addr).copied())
                .collect();
            if let Ok(new_licensee) = String::from_utf8(new_licensee_bytes) {
                println!("Licensee: {}", new_licensee_code_decryption(new_licensee));
            } else {
                println!("Failed to read licensee.");
            }
        }

        let version_number = self.memory.get(0x014C).unwrap_or(&0);
        println!("Version number: {}", *version_number);

        if let Some(header_checksum) = self.memory.get(0x014D) {
            if *header_checksum != 0x00 {
                self.cpu.registers.set_h(true);
                self.cpu.registers.set_c(true);
            }
        } else {
            eprintln!("Unable to access header checksum at 0x014D");
        }
    }

    pub fn toggle_debug_registers(&mut self) {
        self.cpu.toggle_debug_registers();
    }

    fn start(&mut self, test: Option<u64>) {
        if let Some(iterations) = test {
            for _i in 0..iterations {
                self.execute_cycle();
            }
        } else {
            loop {
                self.execute_cycle();
            }
        }
    }

    pub(crate) fn execute_cycle(&mut self) {
        if self.cpu.halted {
            self.ppu.step(4, &mut self.memory);
            self.memory.update_timer(4);
            self.cycles += 4;

            let ie = self.memory.get(0xFFFF).copied().unwrap_or(0);
            let if_ = self.memory.get(0xFF0F).copied().unwrap_or(0);
            let pending = ie & if_;
            if pending != 0 {
                self.cpu.halted = false;
            }

            return;
        }

        if let Some(opcode) = self.memory.get(self.cpu.registers.pc as usize).copied() {
            let (jumped, cycles) = self.cpu.process_opcode(opcode, &mut self.memory);
            self.memory.update_timer(cycles);
            self.cpu.update_ime();

            if !jumped {
                self.cpu.registers.pc = self.cpu.registers.pc.wrapping_add(1);
            }
            self.cpu.check_interrupts(&mut self.memory);

            self.ppu.step(cycles, &mut self.memory);

            self.cycles += cycles;
            if self.cpu.registers.pc == 0x0100 {
                self.memory.disable_rom();
            }
        } else {
            panic!("Tried to access address outside of ROM");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rom_01_special() {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(String::from(
            "resources/roms/blargg/cpu_instrs/individual/01-special.gb",
        ));
        gameboy.start(Some(2_000_000));
        let output = gameboy.memory.get_serial_output().get_output();
        assert!(output.contains("Passed"), "Test failed. Output: {}", output);
    }

    #[test]
    fn rom_02_interrupts() {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(String::from(
            "resources/roms/blargg/cpu_instrs/individual/02-interrupts.gb",
        ));
        gameboy.start(Some(2_000_000));
        let output = gameboy.memory.get_serial_output().get_output();
        assert!(output.contains("Passed"), "Test failed. Output: {}", output);
    }

    #[test]
    fn rom_03_op_sp_hl() {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(String::from(
            "resources/roms/blargg/cpu_instrs/individual/03-op sp,hl.gb",
        ));
        gameboy.start(Some(2_000_000));
        let output = gameboy.memory.get_serial_output().get_output();
        assert!(output.contains("Passed"), "Test failed. Output: {}", output);
    }

    #[test]
    fn rom_04_op_r_imm() {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(String::from(
            "resources/roms/blargg/cpu_instrs/individual/04-op r,imm.gb",
        ));
        gameboy.start(Some(2_000_000));
        let output = gameboy.memory.get_serial_output().get_output();
        assert!(output.contains("Passed"), "Test failed. Output: {}", output);
    }

    #[test]
    fn rom_05_op_rp() {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(String::from(
            "resources/roms/blargg/cpu_instrs/individual/05-op rp.gb",
        ));
        gameboy.start(Some(2_000_000));
        let output = gameboy.memory.get_serial_output().get_output();
        assert!(output.contains("Passed"), "Test failed. Output: {}", output);
    }

    #[test]
    fn rom_06_ld_r_r() {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(String::from(
            "resources/roms/blargg/cpu_instrs/individual/06-ld r,r.gb",
        ));
        gameboy.start(Some(2_000_000));
        let output = gameboy.memory.get_serial_output().get_output();
        assert!(output.contains("Passed"), "Test failed. Output: {}", output);
    }

    #[test]
    fn rom_07_jr_jp_call_ret_rst() {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(String::from(
            "resources/roms/blargg/cpu_instrs/individual/07-jr,jp,call,ret,rst.gb",
        ));
        gameboy.start(Some(2_000_000));
        let output = gameboy.memory.get_serial_output().get_output();
        assert!(output.contains("Passed"), "Test failed. Output: {}", output);
    }

    #[test]
    fn rom_08_misc() {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(String::from(
            "resources/roms/blargg/cpu_instrs/individual/08-misc instrs.gb",
        ));
        gameboy.start(Some(2_000_000));
        let output = gameboy.memory.get_serial_output().get_output();
        assert!(output.contains("Passed"), "Test failed. Output: {}", output);
    }

    #[test]
    fn rom_09_op_r_r() {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(String::from(
            "resources/roms/blargg/cpu_instrs/individual/09-op r,r.gb",
        ));
        gameboy.start(Some(4_500_000));
        let output = gameboy.memory.get_serial_output().get_output();
        assert!(output.contains("Passed"), "Test failed. Output: {}", output);
    }

    #[test]
    fn rom_10_bit_ops() {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(String::from(
            "resources/roms/blargg/cpu_instrs/individual/10-bit ops.gb",
        ));
        gameboy.start(Some(7_000_000));
        let output = gameboy.memory.get_serial_output().get_output();
        assert!(output.contains("Passed"), "Test failed. Output: {}", output);
    }

    #[test]
    fn rom_11_op_a_hl() {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(String::from(
            "resources/roms/blargg/cpu_instrs/individual/11-op a,(hl).gb",
        ));
        gameboy.start(Some(7_500_000));
        let output = gameboy.memory.get_serial_output().get_output();
        assert!(output.contains("Passed"), "Test failed. Output: {}", output);
    }

    #[test]
    fn rom_instr_timing() {
        let mut gameboy = Gameboy::new();
        gameboy.cartridge_to_rom(String::from(
            "resources/roms/blargg/instr_timing/instr_timing.gb",
        ));
        gameboy.start(Some(300_000));
        let output = gameboy.memory.get_serial_output().get_output();
        assert!(output.contains("Passed"), "Test failed. Output: {}", output);
    }
}
