pub fn cartridge_type_decoder(code: u8) -> String {
    match code {
        0x00 => "ROM ONLY",
        0x01 => "MBC1",
        0x02 => "MBC1+RAM",
        0x03 => "MBC1+RAM+BATTERY",
        0x05 => "MBC2",
        0x06 => "MBC2+BATTERY",
        0x08 => "ROM+RAM",
        0x09 => "ROM+RAM+BATTERY",
        0x0B => "MMM01",
        0x0C => "MMM01+RAM",
        0x0D => "MMM01+RAM+BATTERY",
        0x0F => "MBC3+TIMER+BATTERY",
        0x10 => "MBC3+TIMER+RAM+BATTERY",
        0x11 => "MBC3",
        0x12 => "MBC3+RAM",
        0x13 => "MBC3+RAM+BATTERY",
        0x19 => "MBC5",
        0x1A => "MBC5+RAM",
        0x1B => "MBC5+RAM+BATTERY",
        0x1C => "MBC5+RUMBLE",
        0x1D => "MBC5+RUMBLE+RAM",
        0x1E => "MBC5+RUMBLE+RAM+BATTERY",
        0x20 => "MBC6",
        0x22 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
        0xFC => "POCKET CAMERA",
        0xFD => "BANDAI TAMA5",
        0xFE => "HuC3",
        0xFF => "HuC1+RAM+BATTERY",
        _ => "Do not correspond to any hardware type",
    }
    .to_string()
}

pub fn rom_size_decoder(code: u8) -> String {
    match code {
        0x00 => "32 KiB",
        0x01 => "64 KiB",
        0x02 => "128 KiB",
        0x03 => "256 KiB",
        0x04 => "512 KiB",
        0x05 => "1 MiB",
        0x06 => "2 MiB",
        0x07 => "4 MiB",
        0x08 => "8 MiB",
        0x52 => "1.1 MiB",
        0x53 => "1.2 MiB",
        0x54 => "1.5 MiB",
        _ => "Do not correspond to any ROM size",
    }
    .to_string()
}

pub fn ram_size_decoder(code: u8) -> String {
    match code {
        0x00 => "No RAM",
        0x01 => "Unused",
        0x02 => "8 KiB",
        0x03 => "32 KiB",
        0x04 => "128 KiB",
        0x05 => "64 KiB",
        _ => "Do not correspond to any RAM size",
    }
    .to_string()
}

pub fn destination_decoder(code: u8) -> String {
    match code {
        0x00 => "Japan (and possibly overseas)",
        0x01 => "Overseas only",
        _ => "Do not correspond to any destination",
    }
    .to_string()
}
