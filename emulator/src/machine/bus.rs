use std::io::Read;
use std::fs::File;
use std::path::Path;

pub struct Bus {
    low_ram:   Box<[u8; 512 * 1024]>,
    flash_rom: Box<[u8; 512 * 1024]>,
    high_ram:  Box<[u8;2048 * 1024]>,

    pub cycles: u64,
}

impl Bus {
    pub fn new(file_path: String) -> Bus {
        let mut flash_rom = [0u8; 512 * 1024];

        let path = Path::new(&file_path);
        let mut file = File::open(&path).unwrap();

        file.read_exact(&mut flash_rom).unwrap();

        Bus {
            low_ram:   Box::new([0u8; 512 * 1024]),
            flash_rom: Box::new(flash_rom),
            high_ram:  Box::new([0u8;2048 * 1024]),
            cycles: 0
        }
    }

    fn cycle(&mut self) {
        self.cycles = self.cycles.wrapping_add(1);
    }

    pub fn read(&mut self, bank: u8, addr: u16) -> u8 {
        self.cycle();
        match bank {
            0..=0x0F => {
                match addr {
                    0..=0x7EFF => {
                        self.low_ram[bank as usize * 0x8000 + addr as usize]
                    }
                    0x7F00..=0x7FFF => {
                        panic!("MMIO not implemented yet!!!! {:02X}{:04X}", bank, addr);
                    }
                    0x8000..=0xFFFF => {
                        self.flash_rom[bank as usize * 0x8000 + (addr as usize & 0x7FFF)]
                    }
                }
            }
            0x10..=0x1f => {
                panic!("Large MMIO not implemented yet!!!! {:02X}{:04X}", bank, addr);
            }
            0x20..=0x3f => {
                self.high_ram[(bank - 0x20) as usize * 0x8000 + (addr as usize)]
            }
            _ => {
                panic!("Final 12 MiB not implemented yet!!!! {:02X}{:04X}", bank, addr);
            }
        }
    }

    pub fn write(&mut self, bank: u8, addr: u16, value: u8) {
        self.cycle();
        match bank {
            0..=0x0F => {
                match addr {
                    0..=0x7EFF => {
                        self.low_ram[bank as usize * 0x8000 + addr as usize] = value;
                    }
                    0x7F00..=0x7FFF => {
                        panic!("MMIO not implemented yet!!!! {:02X}{:04X} = {:02X}", bank, addr, value);
                    }
                    0x8000..=0xFFFF => {
                        panic!("Write to Flash ROM!!!! {:02X}{:04X} = {:02X}", bank, addr, value);
                    }
                }
            }
            0x10..=0x1f => {
                panic!("Large MMIO not implemented yet!!!! {:02X}{:04X} = {:02X}", bank, addr, value);
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