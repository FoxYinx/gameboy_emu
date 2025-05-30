pub struct Registers {
    pub(crate) a: u8,
    pub(crate) f: u8,
    pub(crate) b: u8,
    pub(crate) c: u8,
    pub(crate) d: u8,
    pub(crate) e: u8,
    pub(crate) h: u8,
    pub(crate) l: u8,
    pub(crate) sp: u16,
    pub(crate) pc: u16,
}

impl Registers {
    pub fn default() -> Registers {
        Registers {
            a: 0x01,
            f: 0b1000_0000,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0000,
        }
    }

    pub fn get_af(&self) -> u16 {
        (self.a as u16) << 8 | self.f as u16
    }

    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    pub fn get_z(&self) -> bool {
        self.f & 0b1000_0000 != 0
    }

    pub fn get_n(&self) -> bool {
        self.f & 0b0100_0000 != 0
    }

    pub fn get_h(&self) -> bool {
        self.f & 0b0010_0000 != 0
    }

    pub fn get_c(&self) -> bool {
        self.f & 0b0001_0000 != 0
    }

    pub fn set_af(&mut self, val: u16) {
        self.a = (val >> 8) as u8;
        self.f = (val & 0x00F0) as u8;
    }

    pub fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = val as u8;
    }

    pub fn set_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = val as u8;
    }

    pub fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = val as u8;
    }

    pub fn set_z(&mut self, toggle: bool) {
        if toggle {
            self.f |= 0b1000_0000;
        } else {
            self.f &= !0b1000_0000;
        }
    }

    pub fn set_n(&mut self, toggle: bool) {
        if toggle {
            self.f |= 0b0100_0000;
        } else {
            self.f &= !0b0100_0000;
        }
    }

    pub fn set_h(&mut self, toggle: bool) {
        if toggle {
            self.f |= 0b0010_0000;
        } else {
            self.f &= !0b0010_0000;
        }
    }

    pub fn set_c(&mut self, toggle: bool) {
        if toggle {
            self.f |= 0b0001_0000;
        } else {
            self.f &= !0b0001_0000;
        }
    }
}
