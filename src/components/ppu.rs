use std::cmp::PartialEq;
use crate::components::memory::Memory;
use crate::components::ppu::PpuMode::*;
use crate::window::emulator_app::{HEIGHT, WIDTH};

#[derive(PartialEq, Clone)]
enum PpuMode {
    OAMScan,
    PixelDrawing,
    HBlank,
    VBlank,
}

pub struct PPU {
    prev_mode: PpuMode,
    mode: PpuMode,
    pub framebuffer: [u8; (WIDTH * HEIGHT * 4) as usize],
    prev_line: u8,
    line: u8,
    mode_clock: u64,
    window_line_counter: u8,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            prev_mode: VBlank,
            mode: OAMScan,
            framebuffer: [0; (WIDTH * HEIGHT * 4) as usize],
            prev_line: 153,
            line: 0,
            mode_clock: 0,
            window_line_counter: 0,
        }
    }

    pub(crate) fn step(&mut self, cycles: u64, memory: &mut Memory) {
        self.mode_clock += cycles;
        while self.mode_clock > 0 {
            match self.mode {
                OAMScan => {
                    if let Some(stat) = memory.get_mut(0xFF41) {
                        *stat = (*stat & 0b1111_1100) | 0b10;
                    }

                    if self.mode_clock >= 80 {
                        self.mode_clock -= 80;
                        self.prev_mode = self.mode.clone();
                        self.mode = PixelDrawing;
                        self.update_stat(memory);
                    } else {
                        break;
                    }
                }
                PixelDrawing => {
                    if let Some(stat) = memory.get_mut(0xFF41) {
                        *stat = (*stat & 0b1111_1100) | 0b11;
                    }

                    if self.mode_clock >= 172 {
                        self.mode_clock -= 172;
                        self.prev_mode = self.mode.clone();
                        self.mode = HBlank;
                        self.update_stat(memory);
                        self.render_scanline(memory);
                    } else {
                        break;
                    }
                }
                HBlank => {
                    if let Some(stat) = memory.get_mut(0xFF41) {
                        *stat &= 0b1111_1100;
                    }

                    if self.mode_clock >= 204 {
                        self.mode_clock -= 204;
                        self.line += 1;
                        memory.write_memory(0xFF44, self.line);

                        if self.line >= 144 {
                            self.prev_mode = self.mode.clone();
                            self.mode = VBlank;
                            self.update_stat(memory);
                            self.window_line_counter = 0;
                            
                            //fixme: This code makes alleywey not run...
                            if let Some(lcdc) = memory.get(0xFF40) {
                                if (*lcdc & 0x80) != 0 {
                                    if let Some(flag) = memory.get_mut(0xFF0F) {
                                        *flag |= 0x01;
                                    }
                                }
                            }
                        } else {
                            self.prev_mode = self.mode.clone();
                            self.mode = OAMScan;
                            self.update_stat(memory);
                        }
                    } else {
                        break;
                    }
                }
                VBlank => {
                    if let Some(stat) = memory.get_mut(0xFF41) {
                        *stat = (*stat & 0b1111_1100) | 0b01;
                    }

                    if self.mode_clock >= 456 {
                        self.mode_clock -= 456;
                        self.line += 1;
                        memory.write_memory(0xFF44, self.line);
                        self.update_stat(memory);

                        if self.line > 153 {
                            self.line = 0;
                            self.prev_mode = self.mode.clone();
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
        let lcdc = memory.get(0xFF40).copied().unwrap_or(0);
        let lyc = memory.get(0xFF45).copied().unwrap_or(0);

        if (lcdc & 0x80) == 0 {
            if let Some(stat_reg) = memory.get_mut(0xFF41) {
                *stat_reg = (*stat_reg & 0b1111_1100) | 0b01;
                *stat_reg &= !0x04;
            }
            self.framebuffer.fill(0xFF);
            return;
        }

        if self.line != self.prev_line {
            if let Some(stat_reg) = memory.get_mut(0xFF41) {
                *stat_reg &= !0x04;
                if self.line == lyc {
                    *stat_reg |= 0x04;
                }
            }
            self.prev_line = self.line;
        }

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

        if self.mode != self.prev_mode {
            let mut trigger_interrupt = false;
            let stat = memory.get(0xFF41).unwrap_or(&0);

            match self.mode {
                OAMScan if (stat & 0x20) != 0 => trigger_interrupt = true, // Mode 2
                VBlank if (stat & 0x10) != 0 => trigger_interrupt = true,   // Mode 1
                HBlank if (stat & 0x08) != 0 => trigger_interrupt = true,   // Mode 0
                _ => {}
            }

            if trigger_interrupt {
                if let Some(flag) = memory.get_mut(0xFF0F) {
                    *flag |= 0x02;
                }
            }
        }
    }

    fn render_scanline(&mut self, memory: &Memory) {
        let lcdc = memory.get(0xFF40).copied().unwrap_or(0);
        if (lcdc & 0x80) == 0 {
            return;
        }

        let bg_window_enable = (lcdc & 0x01) != 0;
        let obj_enable = (lcdc & 0x02) != 0;
        let obj_size = (lcdc & 0x04) != 0;
        let window_enable = (lcdc & 0x20) != 0;

        let bg_tile_map = if (lcdc & 0x08) != 0 { 0x9C00 } else { 0x9800 };
        let window_tile_map = if (lcdc & 0x40) != 0 { 0x9C00 } else { 0x9800 };
        let tile_data = if (lcdc & 0x10) != 0 { 0x8000 } else { 0x8800 };

        let scy = memory.get(0xFF42).copied().unwrap_or(0);
        let scx = memory.get(0xFF43).copied().unwrap_or(0);
        let wy = memory.get(0xFF4A).copied().unwrap_or(0);
        let wx = memory.get(0xFF4B).copied().unwrap_or(0).wrapping_sub(7);

        let wx_effective = memory.get(0xFF4B).copied().unwrap_or(0);
        let window_visible = window_enable && self.line >= wy && (7..=166).contains(&wx_effective);

        if window_visible {
            self.window_line_counter += 1;
        }

        for x in 0..WIDTH {
            let (pixel_x, pixel_y, tile_map) = if window_visible && x as u8 >= wx {
                let pixel_x = (x as u8).wrapping_sub(wx);
                let pixel_y = self.window_line_counter.wrapping_sub(1);
                (pixel_x, pixel_y, window_tile_map)
            } else {
                let pixel_x = (x as u8).wrapping_add(scx);
                let pixel_y = self.line.wrapping_add(scy);
                (pixel_x, pixel_y, bg_tile_map)
            };

            let tile_x = (pixel_x as u16) / 8;
            let tile_y = (pixel_y as u16) / 8;
            let tile_address = tile_map + tile_y * 32 + tile_x;
            let tile_num = memory.get(tile_address as usize).copied().unwrap_or(0) as u16;

            let tile_data_address = if tile_data == 0x8000 {
                tile_data + tile_num * 16
            } else {
                tile_data + ((tile_num as i8 as i16 + 128) as u16) * 16
            };

            let row = (pixel_y % 8) as u16 * 2;
            let byte1 = memory
                .get(tile_data_address as usize + row as usize)
                .copied()
                .unwrap_or(0);
            let byte2 = memory
                .get(tile_data_address as usize + row as usize + 1)
                .copied()
                .unwrap_or(0);

            let bit_index = 7 - (pixel_x as u16 % 8);
            let color_bit_low = (byte1 >> bit_index) & 1;
            let color_bit_high = (byte2 >> bit_index) & 1;
            let color_id = (color_bit_high << 1) | color_bit_low;

            let bgp = memory.get(0xFF47).copied().unwrap_or(0);
            let color = if !bg_window_enable {
                0xFF
            } else {
                match (bgp >> (color_id * 2)) & 0b11 {
                    0 => 0xFF,
                    1 => 0xAA,
                    2 => 0x55,
                    3 => 0x00,
                    _ => 0xFF,
                }
            };

            let index = (self.line as usize * WIDTH as usize + x as usize) * 4;
            self.framebuffer[index] = color;
            self.framebuffer[index + 1] = color;
            self.framebuffer[index + 2] = color;
            self.framebuffer[index + 3] = 0xFF;
        }

        if obj_enable {
            let mut sprites: Vec<(u8, u8, u8, u8)> = Vec::new();
            let sprite_height = if obj_size { 16 } else { 8 };
            for i in 0..40 {
                let sprite_index = i * 4;
                let y_pos = memory.get(0xFE00 + sprite_index).copied().unwrap_or(0);
                let x_pos = memory.get(0xFE01 + sprite_index).copied().unwrap_or(0);
                let mut tile_num = memory.get(0xFE02 + sprite_index).copied().unwrap_or(0);
                let attributes = memory.get(0xFE03 + sprite_index).copied().unwrap_or(0);

                if obj_size {
                    tile_num &= 0xFE;
                }

                if self.line >= y_pos.wrapping_sub(16)
                    && self.line < y_pos.wrapping_sub(16).wrapping_add(sprite_height)
                {
                    sprites.push((x_pos, y_pos, tile_num, attributes));
                }
            }

            sprites.sort_by_key(|&(x_pos, _, _, _)| x_pos);

            for &(x_pos, y_pos, tile_num, attributes) in sprites.iter().take(10).rev() {
                let row = if attributes & 0x40 != 0 {
                    sprite_height - 1 - (self.line.wrapping_sub(y_pos.wrapping_sub(16)))
                } else {
                    self.line.wrapping_sub(y_pos.wrapping_sub(16))
                } as u16
                    * 2;

                let tile_data_address = 0x8000 + (tile_num as u16 * 16) + row;
                let byte1 = memory.get(tile_data_address as usize).copied().unwrap_or(0);
                let byte2 = memory
                    .get(tile_data_address as usize + 1)
                    .copied()
                    .unwrap_or(0);

                for x in 0..8 {
                    let bit_index = if attributes & 0x20 != 0 { x } else { 7 - x };
                    let color_bit_low = (byte1 >> bit_index) & 1;
                    let color_bit_high = (byte2 >> bit_index) & 1;
                    let color_id = (color_bit_high << 1) | color_bit_low;

                    if color_id == 0 {
                        continue;
                    }

                    let obp = if attributes & 0x10 != 0 {
                        memory.get(0xFF49).copied().unwrap_or(0)
                    } else {
                        memory.get(0xFF48).copied().unwrap_or(0)
                    };

                    let color = match (obp >> (color_id * 2)) & 0b11 {
                        0 => 0xFF,
                        1 => 0xAA,
                        2 => 0x55,
                        3 => 0x00,
                        _ => 0xFF,
                    };

                    let pixel_x = x_pos.wrapping_sub(8).wrapping_add(x as u8);
                    if pixel_x < WIDTH as u8 {
                        let index = (self.line as usize * WIDTH as usize + pixel_x as usize) * 4;

                        if (attributes & 0x80) == 0 || self.framebuffer[index] == 0xFF {
                            self.framebuffer[index] = color;
                            self.framebuffer[index + 1] = color;
                            self.framebuffer[index + 2] = color;
                            self.framebuffer[index + 3] = 0xFF;
                        }
                    }
                }
            }
        }
    }

    pub fn copy_to_framebuffer(&self, output: &mut [u8]) {
        output.copy_from_slice(&self.framebuffer);
    }
}
