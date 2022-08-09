use super::super::super::terminal::*;

pub struct UART {
    term: Terminal,
    ier: u8,
    fcr: u8,
    isr: u8,
    lcr: u8,
    mcr: u8,
    lsr: u8,
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
            lsr: 0x60,
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

    pub fn cycle(&mut self) {
        self.cycles = self.cycles.wrapping_add(1);
    }
}