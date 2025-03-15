#[cfg(test)]
pub struct SerialOutput {
    buffer: Vec<u8>,
}

#[cfg(test)]
impl SerialOutput {
    pub fn new() -> Self {
        SerialOutput { buffer: Vec::new() }
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.buffer.push(byte);
    }

    pub fn get_output(&self) -> String {
        String::from_utf8_lossy(&self.buffer).to_string()
    }
}
