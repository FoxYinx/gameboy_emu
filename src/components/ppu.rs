use crate::components::memory::Memory;
use crate::components::ppu::PpuMode::*;
use crate::window::emulator_app::{HEIGHT, WIDTH};

enum PpuMode {
    OAMScan,
    PixelDrawing,
    HBlank,
    VBlank,
}

pub struct PPU {
    mode: PpuMode,
    pub framebuffer: [u8; (WIDTH * HEIGHT * 4) as usize],
    line: u8,
    mode_clock: u64
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            mode: OAMScan,
            framebuffer: [0; (WIDTH * HEIGHT * 4) as usize],
            line: 0,
            mode_clock: 0
        }
    }

    pub(crate) fn step(&mut self, cycles: u64, memory: &mut Memory) {
        self.mode_clock += cycles;
        while self.mode_clock > 0 {
            match self.mode {
                OAMScan => {
                    if self.mode_clock >= 80 {
                        self.mode_clock -= 80;
                        self.mode = PixelDrawing;
                    } else {
                        break;
                    }
                }
                PixelDrawing => {
                    if self.mode_clock >= 172 {
                        self.mode_clock -= 172;
                        self.mode = HBlank;
                        self.render_scanline(memory);
                    } else {
                        break;
                    }
                }
                HBlank => {
                    if self.mode_clock >= 204 {
                        self.mode_clock -= 204;
                        self.line += 1;
                        memory.write_memory(0xFF44, self.line);
                        self.update_stat(memory);
                        
                        if self.line >= 144 {
                            self.mode = VBlank;
                            if let Some(flag) = memory.get_mut(0xFF0F) {
                                *flag |= 0x01;
                            }
                        } else {
                            self.mode = OAMScan;
                        }
                    } else {
                        break;
                    }
                }
                VBlank => {
                    if self.mode_clock >= 456 {
                        self.mode_clock -= 456;
                        self.line += 1;
                        memory.write_memory(0xFF44, self.line);
                        self.update_stat(memory);
                        
                        if self.line > 153 {
                            self.line = 0;
                            self.mode = OAMScan;
                            memory.write_memory(0xFF44, self.line);
                            self.update_stat(memory);
                        }
                    } else {
                        break;
                    }
                }
            }
        }
    }
    
    fn update_stat(&mut self, memory: &mut Memory) {
        let lyc = memory.get(0xFF45).copied().unwrap_or(0);
        if let Some(stat) = memory.get_mut(0xFF41) {
            if self.line == lyc {
                *stat |= 0x04;
                if (*stat & 0x40) != 0 {
                    if let Some(flag) = memory.get_mut(0xFF0F) {
                        *flag |= 0x02;
                    }
                }
            }
        }
    }
    
    fn render_scanline(&mut self, memory: &Memory) {
        let lcdc = memory.get(0xFF40).copied().unwrap_or(0);
        if (lcdc & 0x80) == 0 {
            return;
        }

        let bg_window_enable = (lcdc & 0x01) == 1;
        
        let bg_tile_map = if (lcdc & 0x08) != 0 {0x9C00} else {0x9800};
        let tile_data = if (lcdc & 0x10) != 0 {0x8000} else {0x8800};

        let scy = memory.get(0xFF42).copied().unwrap_or(0);
        let scx = memory.get(0xFF43).copied().unwrap_or(0);
        
        for x in 0..WIDTH {
            let pixel_x = (x as u8).wrapping_add(scx);
            let pixel_y = self.line.wrapping_add(scy);

            let tile_x = (pixel_x as u16) / 8;
            let tile_y = (pixel_y as u16) / 8;
            let tile_address = bg_tile_map + tile_y * 32 + tile_x;
            let tile_num = memory.get(tile_address as usize).copied().unwrap_or(0) as u16;
            
            let tile_data_address = if tile_data == 0x8000 {
                tile_data + tile_num * 16
            } else {
                tile_data + ((tile_num as i8 as i16 + 128) as u16) * 16  
            };
            
            let row = (self.line % 8) as u16 * 2;
            let byte1 = memory.get(tile_data_address as usize + row as usize).copied().unwrap_or(0);
            let byte2 = memory.get(tile_data_address as usize + row as usize + 1).copied().unwrap_or(0);

            let bit_index = 7 - (pixel_x as u16 % 8);
            let color_bit_high = (byte1 >> bit_index) & 1;
            let color_bit_low = (byte2 >> bit_index) & 1;
            let color_id = (color_bit_high << 1) | color_bit_low;
            
            let bgp = memory.get(0xFF47).copied().unwrap_or(0);
            let color = if !bg_window_enable {0xFF} else {match (bgp >> (color_id * 2)) & 0b11 {
                0 => 0xFF,
                1 => 0x55,
                2 => 0xAA,
                3 => 0x00,
                _ => 0xFF,
            }};
            
            let index = (self.line as usize * WIDTH as usize + x as usize) * 4;
            self.framebuffer[index] = color;
            self.framebuffer[index + 1] = color;
            self.framebuffer[index + 2] = color;
            self.framebuffer[index + 3] = 0xFF;
        }
    }
    
    pub fn copy_to_framebuffer(&self, output: &mut [u8]) {
        output.copy_from_slice(&self.framebuffer);
    }
}