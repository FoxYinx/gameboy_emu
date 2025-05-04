use blip_buf::BlipBuf;

const CLOCK_RATE : f64 = 4_194_304.0;
const SAMPLE_RATE : u32 = 44100;

pub struct SquareWave {
    enabled: bool,
    pub(crate) buffer: BlipBuf,
    phase: f64,
    frequency: f64,
    amplitude: f64,
    volume: f64,
}

impl SquareWave {
    fn new() -> Self {
        let mut  buffer = BlipBuf::new(SAMPLE_RATE);
        buffer.set_rates(CLOCK_RATE, SAMPLE_RATE as f64);

        SquareWave {
            enabled: true,
            buffer,
            phase: 0.0,
            frequency: 440.0,
            amplitude: 0.5,
            volume: 0.5,
        }
    }

    pub fn render(&mut self, cycles: u64) {
        if !self.enabled {
            return;
        }

        let period = 1.0 / self.frequency;
        let mut current_time: u32 = 0;

        for _ in 0..cycles {
            let old_phase = self.phase;
            self.phase += 1.0 / CLOCK_RATE;
            
            if self.phase >= period {
                self.phase -= period;
            }
            
            let was_high = (old_phase / period) < self.amplitude;
            let is_high = (self.phase / period) < self.amplitude;
            
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
    pub(crate) channel1: SquareWave,
}

impl APU {
    pub fn new() -> Self {
        APU {
            channel1: SquareWave::new(),
        }
    }
    
    pub fn step(&mut self, cycles: u64) {
        self.channel1.render(cycles);
    }
}
