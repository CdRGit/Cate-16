pub struct Bus {
    low_ram:   Box<[u8; 512 * 1024]>,
    flash_rom: Box<[u8; 512 * 1024]>,
    high_ram:  Box<[u8;2048 * 1024]>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            low_ram:   Box::new([0u8; 512 * 1024]),
            flash_rom: Box::new([0u8; 512 * 1024]),
            high_ram:  Box::new([0u8;2048 * 1024]),
        }
    }

    pub fn read(&mut self, bank: u8, addr: u16) -> u8 {
        match bank {
            0..=0x0F => {
                match addr {
                    0..=0x7EFF => {
                        self.low_ram[bank as usize * 0x8000 + addr as usize]
                    }
                    0x7F00..=0x7FFF => {
                        panic!("MMIO not implemented yet!!!!");
                    }
                    0x8000..=0xFFFF => {
                        self.flash_rom[bank as usize * 0x8000 + (addr as usize & 0x7FFF)]
                    }
                }
            }
            0x10..=0x1f => {
                panic!("Large MMIO not implemented yet!!!!");
            }
            0x20..=0x3f => {
                self.high_ram[(bank - 0x20) as usize * 0x8000 + (addr as usize)]
            }
            _ => {
                panic!("Final 12 MiB not implemented yet!!!!");
            }
        }
    }

    pub fn write(&mut self, bank: u8, addr: u16, value: u8) {
        match bank {
            0..=0x0F => {
                match addr {
                    0..=0x7EFF => {
                        self.low_ram[bank as usize * 0x8000 + addr as usize] = value;
                    }
                    0x7F00..=0x7FFF => {
                        panic!("MMIO not implemented yet!!!!");
                    }
                    0x8000..=0xFFFF => {
                        panic!("Write to Flash ROM!!!!");
                    }
                }
            }
            0x10..=0x1f => {
                panic!("Large MMIO not implemented yet!!!!");
            }
            0x20..=0x3f => {
                self.high_ram[(bank - 0x20) as usize * 0x8000 + (addr as usize)] = value;
            }
            _ => {
                panic!("Final 12 MiB not implemented yet!!!!");
            }
        }
    }
}