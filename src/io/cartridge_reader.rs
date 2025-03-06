use std::fs;

pub fn read_cartridge(filename: String) -> Vec<u8> {
    fs::read(filename).unwrap()
}