pub struct Registers {
    a: u8
}

impl Registers {
    pub fn default() -> Registers {
        Registers {
            a: 5
        }
    }
}