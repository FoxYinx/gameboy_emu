use crate::components::memory::Memory;
use blip_buf::BlipBuf;

const CLOCK_RATE : f64 = 4_194_304.0;
const SAMPLE_RATE : u32 = 44100;

pub struct SquareWave {
    enabled: bool,
    pub(crate) buffer: BlipBuf,
    phase: f64,
    frequency: f64,
    duty: f64,
    volume: f64,
    length_timer: u16,
    envelope_volume: u8,
    envelope_pace: u8,
    envelope_direction: bool,
    period_value: u16,
    length_enable: bool
}

impl SquareWave {
    fn new() -> Self {
        let mut buffer = BlipBuf::new(SAMPLE_RATE);
        buffer.set_rates(CLOCK_RATE, SAMPLE_RATE as f64);

        SquareWave {
            enabled: true,
            buffer,
            phase: 0.0,
            frequency: 440.0,
            duty: 0.5,
            volume: 0.15,
            length_timer: 0,
            envelope_volume: 0,
            envelope_pace: 0,
            envelope_direction: false,
            period_value: 0,
            length_enable: false,
        }
    }

    fn handle_nr21(&mut self, value: u8) {
        self.duty =  match (value >> 6) & 0b11 {
            0 => 0.125,
            1 => 0.25,
            2 => 0.5,
            3 => 0.75,
            _ => unreachable!(),
        };
        self.length_timer = 64 - (value & 0x3F) as u16
    }
    
    fn handle_nr22(&mut self, value: u8) {
        self.envelope_volume = value >> 4;
        self.envelope_direction = (value >> 3) & 1 != 0;
        self.envelope_pace = value & 0b111;
    }
    
    fn handle_nr23(&mut self, value: u8) {
        self.period_value = (self.period_value & 0x0700) | (value as u16);
        self.update_frequency();
    }
    
    fn handle_nr24(&mut self, value: u8) {
        self.period_value = (self.period_value & 0x00FF) | (((value & 0b111) as u16) << 8);
        self.length_enable = (value >> 6) & 1 != 0;
        if (value >> 7) & 1 != 0 {
            self.trigger();
        }
        self.update_frequency();
    }
    
    fn update_frequency(&mut self) {
        self.frequency = 131072.0 / (2048.0 - self.period_value as f64);
    }
    
    fn trigger(&mut self) {
        self.enabled = true;
        self.phase = 0.0;
        self.length_timer = 64;
        self.envelope_volume = (self.volume * 15.0) as u8;
    }

    pub fn render(&mut self, cycles: u64, memory: &mut Memory) {
        if !self.enabled {
            return;
        }

        self.handle_nr21(*memory.get(0xFF16).unwrap());
        self.handle_nr22(*memory.get(0xFF17).unwrap());
        self.handle_nr23(*memory.get(0xFF18).unwrap());
        self.handle_nr24(*memory.get(0xFF19).unwrap());
        
        self.length_timer = self.length_timer.saturating_sub(1);
        if self.length_timer == 0 && self.length_enable {
            self.enabled = false;
        }
        
        if cycles % 64 == 0 && self.envelope_pace > 0{
            if self.envelope_direction {
                self.envelope_volume = self.envelope_volume.saturating_add(1);
            } else { 
                self.envelope_volume = self.envelope_volume.saturating_sub(1);
            }
            self.volume = self.envelope_volume as f64 / 15.0;
        }

        let period = 1.0 / self.frequency;
        let mut current_time: u32 = 0;

        for _ in 0..cycles {
            let old_phase = self.phase;
            self.phase += 1.0 / CLOCK_RATE;
            
            if self.phase >= period {
                self.phase -= period;
            }
            
            let was_high = (old_phase / period) < self.duty;
            let is_high = (self.phase / period) < self.duty;
            
            if was_high != is_high {
                let value = if is_high { self.volume } else { -self.volume };
                self.buffer.add_delta(
                    current_time,
                    (value * 32767.0) as i32,
                );
            }

            current_time = current_time.wrapping_add(1);
        }

        self.buffer.end_frame(cycles as u32);
    }
}


pub struct APU {
    pub(crate) channel2: SquareWave,
}

impl APU {
    pub fn new() -> Self {
        APU {
            channel2: SquareWave::new(),
        }
    }
    
    pub fn step(&mut self, cycles: u64, memory: &mut Memory) {
        self.channel2.render(cycles, memory);
    }
}
