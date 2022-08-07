pub mod addressing;
pub mod status;

use std::ops::Add;

use super::bus::*;
use addressing::AddressingMode;
use status::Status;

#[derive(Clone, Copy, PartialEq)]
pub enum RunStatus {
    Running, Waiting, Stopped
}

pub struct W65C816 {
    a: u16,
    x: u16,
    y: u16,
    s: u16,

    dbr: u8,
    pbr: u8,

    d:  u16,
    pc: u16,

    emulation: bool,
    p: Status,

    run_status: RunStatus,

    bus: Bus,
}

const RESET_VEC8: u16 = 0xFFFC;

impl W65C816 {
    pub fn new(mut bus: Bus) -> Self {
        let pcl = bus.read(0, RESET_VEC8) as u16;
        let pch = bus.read(0, RESET_VEC8 + 1) as u16;

        let pc = (pch << 8) | pcl;

        W65C816 {
            a: 0,
            x: 0,
            y: 0,
            s: 0x100,
            dbr: 0, pbr: 0,
            d:   0, pc,
            emulation: true,
            p: Status::new(),
            run_status: RunStatus::Running,
            bus,
        }
    }

    fn set_emulation(&mut self, value: bool) {
        if !self.emulation && value {
            // Enter emulation mode
            self.s = 0x0100 | (self.s & 0xFF);
            self.p.set_small_acc(true);
            self.p.set_small_idx(true);
            self.x &= 0xFF;
            self.y &= 0xFF;
        }
    }

    fn set_p(&mut self, mut new: u8) {
        let small_idx = self.p.small_idx();
        self.p.0 = new;
        if !small_idx && self.p.small_idx() {
            self.x &= 0xff;
            self.y &= 0xff;
        }
    }

    fn fetchb(&mut self) -> u8 {
        let val = self.loadb(self.pbr, self.pc);
        self.pc = self.pc.wrapping_add(1);
        val
    }

    fn fetchw(&mut self) -> u16 {
        let low = self.fetchb() as u16;
        let high = self.fetchb() as u16;
        (high << 8) | low
    }

    fn loadb(&mut self, bank: u8, addr: u16) -> u8 {
        self.bus.read(bank, addr)
    }
    
    fn loadw(&mut self, bank: u8, addr: u16) -> u16 {
        assert!(addr < 0xffff, "loadw on bank boundary");
        // ^ if this should be supported, make sure to fix the potential overflow below

        let lo = self.loadb(bank, addr) as u16;
        let hi = self.loadb(bank, addr + 1) as u16;
        (hi << 8) | lo
    }

    pub fn instruction(&mut self) -> RunStatus {
        if self.run_status != RunStatus::Running {
            return self.run_status;
        }

        let opcode = self.fetchb();

        macro_rules! instr {
            ( $name:ident ) => {{
                self.$name()
            }};
            ( $name:ident $am:ident ) => {{
                let am = self.$am();
                self.$name(am)
            }};
        }

        match opcode {
            // register load + store
            0xA9 => instr!( lda immediate_acc ),
            0xA2 => instr!( ldx immediate_index ),

            // register transfers
            0x9A => instr!( txs ),

            // flag manipulation
            0x18 => instr!( clc ),
            0xD8 => instr!( cld ),
            0x58 => instr!( cli ),
            0xB8 => instr!( clv ),

            0x38 => instr!( sec ),
            0xF8 => instr!( sed ),
            0x78 => instr!( sei ),

            0xFB => instr!( xce ),

            0xC2 => instr!( rep immediate8 ),
            0xE2 => instr!( sep immediate8 ),
            // processor control
            0xDB => instr!( stp ),
            0xCB => instr!( wai ),
            // other
            _ => { panic!("Opcode {:02X} not implemented yet", opcode); }
        }

        self.run_status
    }

    fn ldx(&mut self, am: AddressingMode) {
        if self.p.small_idx() {
            let val = am.loadb(self);
            self.x = self.p.set_nz_8(val) as u16;
        } else {
            let val = am.loadw(self);
            self.x = self.p.set_nz(val);
        }
    }

    fn txs(&mut self) {
        if self.emulation {
            self.s = 0x0100 | (self.x & 0xff);
        } else {
            self.s = self.x;
        }
    }

    fn clc(&mut self) { self.p.set_carry(false); }
    fn sec(&mut self) { self.p.set_carry(true); }
    fn cld(&mut self) { self.p.set_decimal(false); }
    fn sed(&mut self) { self.p.set_decimal(true); }
    fn cli(&mut self) { self.p.set_interrupt(false); }
    fn sei(&mut self) { self.p.set_interrupt(true); }
    fn clv(&mut self) { self.p.set_overflow(false); }

    fn rep(&mut self, am: AddressingMode) {
        let p = self.p.0 & !am.loadb(self);
        self.set_p(p);
    }

    fn sep(&mut self, am: AddressingMode) {
        let p = self.p.0 | am.loadb(self);
        self.set_p(p);
    }

    fn xce(&mut self) {
        let carry = self.p.carry();
        let e = self.emulation;
        self.p.set_carry(e);
        self.set_emulation(carry);
    }

    fn stp(&mut self) {
        self.run_status = RunStatus::Stopped
    }

    fn wai(&mut self) {
        self.run_status = RunStatus::Waiting
    }

    fn immediate8(&mut self) -> AddressingMode {
        AddressingMode::Immediate8(self.fetchb())
    }

    fn immediate_acc(&mut self) -> AddressingMode {
        if self.p.small_acc() {
            AddressingMode::Immediate8(self.fetchb())
        } else {
            AddressingMode::Immediate(self.fetchw())
        }
    }

    fn immediate_index(&mut self) -> AddressingMode {
        if self.p.small_idx() {
            AddressingMode::Immediate8(self.fetchb())
        } else {
            AddressingMode::Immediate(self.fetchw())
        }
    }
}