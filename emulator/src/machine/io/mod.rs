pub mod uart;
use uart::UART;

pub struct IO {
    uart: UART
}

impl IO {
    pub fn new() -> Self {
        Self { uart: UART::new() }
    }

    pub fn cycle(&mut self) {
        self.uart.cycle();
    }

    pub fn read(&mut self, addr: u8) -> u8 {
        match addr {
            0x00..=0x0F => todo!(),
            0x10..=0x18 => todo!(),
            0x19..=0xFF => todo!(),
        }
    }

    pub fn write(&mut self, addr: u8) -> u8 {
        match addr {
            0x00..=0x0F => todo!(),
            0x10..=0x18 => todo!(),
            0x19..=0xFF => todo!(),
        }
    }
}