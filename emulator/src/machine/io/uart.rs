use super::super::super::terminal::*;

use std::io::{Read, Write};

pub struct UART {
    term: Terminal,
    ier: u8,
    fcr: u8,
    isr: u8,
    lcr: u8,
    mcr: u8,
    msr: u8,
    spr: u8,
    brg: u16,

    tx_fifo: [u8;16],
    tx_count: u8,
    rx_fifo: [u8;16],
    rx_count: u8,

    cycles: u64,
}

impl UART {
    pub fn new() -> Self {
        Self { 
            term: Terminal::new(),
            ier: 0x00,
            fcr: 0x00,
            isr: 0x01,
            lcr: 0x00,
            mcr: 0x00,
            msr: 0x00,
            spr: 0xFF,
            brg: 0x0000,
            tx_fifo: [0;16],
            rx_fifo: [0;16],
            tx_count: 0,
            rx_count: 0,
            cycles: 0,
        }
    }

    pub fn handle_term(&mut self) -> bool {
        (
            if self.tx_count > 0 {
                self.term.write(&[self.tx_fifo[0]]).unwrap();
                self.term.flush().unwrap();
                for i in 1..self.tx_count as usize {
                    self.tx_fifo[i - 1] = self.tx_fifo[i];
                }
                self.tx_count -= 1;
                true
            } else { false }
            |
            if self.rx_count < 16 {
                let mut buf = [0u8;1];
                let count = self.term.read(&mut buf).unwrap();
                if count == 1 {
                    for i in 1..=self.rx_count as usize {
                        self.rx_fifo[i] = self.rx_fifo[i - 1];
                    }
                    self.rx_fifo[0] = buf[0];
                    self.rx_count += 1;
                    true
                } else {false}
            } else { false }
        )
    }

    pub fn cycle(&mut self) {
        self.cycles = self.cycles.wrapping_add(1);
        let cycles_between_char = ((25_175_000f64 / 8.0) / (1_843_200f64 / self.brg as f64 / 16.0) * 8.0) as u64;
        if self.cycles >= cycles_between_char {
            self.cycles = 0;
            self.handle_term();
        }
    }

    pub fn read(&mut self, addr: u8) -> u8 {
        match addr {
            0x00 => {
                if (self.lcr & 0x80) != 0 {
                    // BRG
                    todo!("BRG read");
                } else {
                    if self.rx_count == 0 {
                        panic!("UART empty")
                    }
                    let val = self.rx_fifo[0];
                    for i in 1..self.rx_count as usize {
                        self.rx_fifo[i - 1] = self.rx_fifo[i];
                    }
                    self.rx_count -= 1;

                    val
                }
            }
            0x03 => self.lcr,
            0x05 => {
                let mut val = 0u8;
                // bit 0: any data in RX
                if self.rx_count > 0 {
                    val |= 0x01;
                }
                // bit 1: overrun error (not possible rn)
                // bit 2: parity error (not possible rn)
                // bit 3: framing error (not possible rn)
                // bit 4: break condition (not possible rn)
                // bit 5 & 6: transmit empty (slightly different semantics on hardware)
                if self.tx_count == 0 {
                    val |= 0x60;
                }
                // bit 7: error (not possible rn)

                val
            }
            _ => todo!("r {:02}", addr),
        }
    }

    pub fn write(&mut self, addr: u8, value: u8) {
        match addr {
            0x00 => {
                if (self.lcr & 0x80) != 0 {
                    // BRG
                    self.brg &= 0xFF00;
                    self.brg |= value as u16;
                    println!("[UART] BAUDRATE: {} BAUD", (1_843_200f64 / self.brg as f64) / 16f64);
                } else {
                    if self.tx_count == 16 {
                        panic!("UART filled too far")
                    }
                    self.tx_fifo[self.tx_count as usize] = value;
                    self.tx_count += 1;
                }
            }
            0x01 => {
                if (self.lcr & 0x80) != 0 {
                    // BRG
                    self.brg &= 0x00FF;
                    self.brg |= (value as u16) << 8;
                    println!("[UART] BAUDRATE: {} BAUD", (1_843_200f64 / self.brg as f64) / 16f64);
                } else {
                    // IER
                    todo!("IER");
                }
            }
            0x02 => {
                self.fcr = value & 0b11001111; // ignored for now
                println!("[UART] FCR: {:02X}", self.fcr);
            }
            0x03 => {
                self.lcr = value;
                println!("[UART] LCR: {:02X}", self.lcr);
            },
            _ => todo!("w {:02}", addr),
        }
    }
}

impl Drop for UART {
    fn drop(&mut self) {
        while self.handle_term() {}
    }
}