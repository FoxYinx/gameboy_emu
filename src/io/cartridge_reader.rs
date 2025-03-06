use std::fs;

pub(crate) fn read_cartridge(filename: String) -> Vec<u8> {
    fs::read(filename).unwrap()
}