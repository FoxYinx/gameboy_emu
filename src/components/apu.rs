use crate::components::memory::Memory;
use blip_buf::BlipBuf;

const WAVE_PATTERN : [[i32; 8]; 4] = [[-1,-1,-1,-1,1,-1,-1,-1],[-1,-1,-1,-1,1,1,-1,-1],[-1,-1,1,1,1,1,-1,-1],[1,1,1,1,-1,-1,1,1]];
const CLOCK_RATE : u32 = 4_194_304;
const CLOCK_PER_FRAME: u32 = CLOCK_RATE / 512;
const SAMPLE_RATE : u32 = 44100;

struct VolumeEnvelope {
    period: u8,
    goes_up: bool,
    delay: u8,
    initial_volume: u8,
    volume: u8,
}

impl VolumeEnvelope {
    fn new() -> Self {
        VolumeEnvelope {
            period: 0,
            goes_up: false,
            delay: 0,
            initial_volume: 0,
            volume: 0,
        }
    }
    
    fn get(&self, address: u16) -> u8 {
        match address {
            0xFF12 | 0xFF17 | 0xFF21 => {
                ((self.initial_volume & 0xF) << 4) |
                if self.goes_up { 0x08 } else { 0 } |
                (self.period & 0x7)
            },
            _ => unreachable!(),
        }
    }
    
    fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF12 | 0xFF17 | 0xFF21 => {
                self.period = value & 0x7;
                self.goes_up = (value >> 7) & 1 != 0;
                self.initial_volume = value >> 4;
                self.volume = self.initial_volume;
            },
            0xFF14 | 0xFF19 | 0xFF23 if value & 0x80 != 0 => {
                self.delay = self.period;
                self.volume = self.initial_volume;
            }
            _ => (),
        }
    }
    
    fn step(&mut self) {
        if self.delay > 1 {
            self.delay -= 1;
        } else if self.delay == 1 {
            self.delay = self.period;
            if self.goes_up && self.volume < 15 {
                self.volume += 1;
            } else if !self.goes_up && self.volume > 0{
                self.volume -= 1;
            }
        }
    }
}

struct LengthTimer {
    enabled: bool,
    value: u16,
    max: u16
}

impl LengthTimer {
    fn new(max: u16) -> Self {
        LengthTimer {
            enabled: false,
            value: 0,
            max,
        }
    }
    
    fn is_active(&self) -> bool {
        self.value > 0
    }
    
    fn extra_step(frame_step: u8) -> bool {
        frame_step % 2 == 1
    }
    
    fn enable(&mut self, enable: bool, frame_step: u8) {
        let was_enabled = self.enabled;
        self.enabled = enable;
        if !was_enabled && LengthTimer::extra_step(frame_step) {
            self.step();
        }
    }
    
    fn set(&mut self, minus_value: u8) {
        self.value = self.max - minus_value as u16;
    }
    
    fn trigger(&mut self, frame_step: u8) {
        if self.value == 0 {
            self.value = self.max;
            if LengthTimer::extra_step(frame_step) {
                self.step();
            }
        }
    }
    
    fn step(&mut self) {
        if self.enabled && self.value > 0 {
            self.value -= 1;
        }
    }
}

pub struct SquareWave {
    enabled: bool,
    duty: u8,
    phase: u8,
    length_timer: LengthTimer,
    volume_envelope: VolumeEnvelope,
    frequency: u16,
    period: u32,
    last_amp: i32,
    delay: u32,
    pub(crate) buffer: BlipBuf,
}

impl SquareWave {
    fn new() -> Self {
        let mut buffer = BlipBuf::new(SAMPLE_RATE);
        buffer.set_rates(CLOCK_RATE as f64, SAMPLE_RATE as f64);

        SquareWave {
            enabled: true,
            duty: 1,
            phase: 1,
            length_timer: LengthTimer::new(64),
            volume_envelope: VolumeEnvelope::new(),
            frequency: 0,
            period: 0,
            last_amp: 0,
            delay: 0,
            buffer
        }
    }

    fn handle_nr21(&mut self, value: u8) {
        self.duty = value >> 6;
        self.length_timer.set(value & 0x3F);
    }
    
    fn handle_nr22(&mut self, value: u8) {
        self.volume_envelope.write(0xFF17, value);
    }
    
    fn handle_nr23(&mut self, value: u8) {
        self.frequency = (self.frequency & 0x0700) | (value as u16);
        self.calculate_period();
    }
    
    fn handle_nr24(&mut self, value: u8, frame_step: u8) {
        self.frequency = (self.frequency & 0x00FF) | (((value & 0b111) as u16) << 8);
        self.length_timer.enable((value >> 6) & 1 != 0, frame_step);
        self.enabled &= self.length_timer.enabled;
        
        if (value >> 7) & 1 != 0 {
            self.length_timer.trigger(frame_step);
        }
        self.calculate_period();
    }
    
    fn calculate_period(&mut self) {
        if self.frequency > 2047 {
            self.period = 0;
        } else {
            self.period = (2048 - self.frequency as u32) * 4;
        }
    }
    
    fn step_length(&mut self) {
        self.length_timer.step();
        self.enabled &= self.length_timer.enabled;
    }

    pub fn run(&mut self, memory: &mut Memory, frame_step: u8, start_time: u32, end_time: u32) {
        if !self.enabled || self.period == 0 {
            if self.last_amp != 0 {
                self.buffer.add_delta(start_time, -self.last_amp);
                self.last_amp = 0;
                self.delay = 0;
            }
        } else {
            self.handle_nr21(*memory.get(0xFF16).unwrap());
            self.handle_nr22(*memory.get(0xFF17).unwrap());
            self.handle_nr23(*memory.get(0xFF18).unwrap());
            self.handle_nr24(*memory.get(0xFF19).unwrap(), frame_step);

            let mut time = start_time + self.delay;
            let pattern = WAVE_PATTERN[self.duty as usize];
            let vol = self.volume_envelope.volume as i32;

            while time < end_time {
                let amp = vol * pattern[self.phase as usize];
                if amp != self.last_amp {
                    self.buffer.add_delta(time, amp - self.last_amp);
                    self.last_amp = amp;
                }
                time += self.period;
                self.phase = (self.phase + 1) % 8;
            }
            
            self.delay = time - end_time;
        }
    }
}


pub struct APU {
    enabled: bool,
    time: u32,
    prev_time: u32,
    next_time: u32,
    frame_step: u8,
    output_period: u32,
    pub(crate) channel2: SquareWave,
}

impl APU {
    pub fn new() -> Self {
        //fixme: revoir ça
        let output_period = ((SAMPLE_RATE as u64 * CLOCK_RATE as u64) / (SAMPLE_RATE as u64)) as u32;
        
        APU {
            enabled: true,
            time: 0,
            prev_time: 0,
            next_time: CLOCK_PER_FRAME,
            frame_step: 0,
            output_period,
            channel2: SquareWave::new(),
        }
    }
    
    pub fn step(&mut self, cycles: u32, memory: &mut Memory) {
        if !self.enabled {
            return;
        }
        
        self.time += cycles;
        if self.time >= self.output_period {
            self.do_output(memory);
        }
    }
    
    fn do_output(&mut self, memory: &mut Memory) {
        self.run(memory);
        
        self.channel2.buffer.end_frame(self.time);
        self.next_time -= self.time;
        self.time = 0;
        self.prev_time = 0;
    }
    
    fn run(&mut self, memory: &mut Memory) {
        while self.next_time <= self.time {
            self.channel2.run(memory, self.frame_step, self.prev_time, self.next_time);
            
            if self.frame_step % 2 == 0 {
                self.channel2.step_length();
            }
            if self.frame_step == 7 {
                self.channel2.volume_envelope.step();
            }
            
            self.frame_step = (self.frame_step + 1) % 8;
            self.prev_time = self.next_time;
            self.next_time += CLOCK_PER_FRAME;
        }
        
        if self.prev_time != self.time {
            self.channel2.run(memory, self.frame_step, self.prev_time, self.time);
            
            self.prev_time = self.time;
        }
    }
}
