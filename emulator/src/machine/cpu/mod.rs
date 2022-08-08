pub mod addressing;
pub mod status;

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

        bus.cycles = 0;

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

    fn compare(&mut self, a: u16, b: u16) {
        self.p.set_zero(a == b);
        self.p.set_carry(a >= b);
        self.p.set_negative(a.wrapping_sub(b) & 0x8000 != 0);
    }
    
    fn compare8(&mut self, a: u8, b: u8) {
        self.p.set_zero(a == b);
        self.p.set_carry(a >= b);
        self.p.set_negative(a.wrapping_sub(b) & 0x80 != 0);
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

    fn branch(&mut self, target: (u8, u16)) {
        self.pbr = target.0;
        self.pc = target.1;
    }

    fn set_p(&mut self, new: u8) {
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

    fn storeb(&mut self, bank: u8, addr: u16, value: u8) {
        self.bus.write(bank, addr, value)
    }
    
    fn storew(&mut self, bank: u8, addr: u16, value: u16) {
        self.storeb(bank, addr, value as u8);
        if addr == 0xffff {
            self.storeb(bank + 1, 0, (value >> 8) as u8);
        } else {
            self.storeb(bank, addr + 1, (value >> 8) as u8);
        }
    }

    pub fn instruction(&mut self) -> RunStatus {
        if self.run_status != RunStatus::Running {
            return self.run_status;
        }
        let pbr = self.pbr;
        let pc = self.pc;
        let opcode = self.fetchb();

        macro_rules! instr {
            ( $name:ident ) => {{
                self.$name();
            }};
            ( $name:ident $am:ident ) => {{
                let am = self.$am();
                self.$name(am);
            }};
        }

        match opcode {
            // branches
            0x80 => instr!( bra rel ),
            0xD0 => instr!( bne rel ),

            // comparisons
            0xDD => instr!( cmp absolute_indexed_x ),
            0xE0 => instr!( cpx immediate_index ),

            // register load + store
            0xA9 => instr!( lda immediate_acc ),
            0xA5 => instr!( lda direct ),
            0xA2 => instr!( ldx immediate_index ),

            0x85 => instr!( sta direct ),
            0x8D => instr!( sta absolute ),
            0x9D => instr!( sta absolute_indexed_x ),

            // register transfers
            0x8A => instr!( txa ),
            0x9A => instr!( txs ),

            // register manipulation
            0xCA => instr!( dex ),

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
            _ => { panic!("Opcode {:02X} not implemented yet [at {:02X}{:04X}]", opcode, self.pbr, self.pc.wrapping_sub(1)); }
        }

        self.run_status
    }

    fn bra(&mut self, am: AddressingMode) {
        let a = am.address(self);
        self.branch(a);
    }

    fn bne(&mut self, am: AddressingMode) {
        let a = am.address(self);
        if !self.p.zero() {
            self.branch(a);
        }
    }

    fn cmp(&mut self, am: AddressingMode) {
        if self.p.small_acc() {
            let a = self.a as u8;
            let b = am.loadb(self);
            self.compare8(a, b);
        } else {
            let a = self.a;
            let b = am.loadw(self);
            self.compare(a, b);
        }
    }

    fn cpx(&mut self, am: AddressingMode) {
        if self.p.small_idx() {
            let x = self.x as u8;
            let val = am.loadb(self);
            self.compare8(x, val);
        } else {
            let x = self.x;
            let val = am.loadw(self);
            self.compare(x, val);
        }
    }

    fn lda(&mut self, am: AddressingMode) {
        if self.p.small_acc() {
            let val = am.loadb(self);
            self.a = self.p.set_nz_8(val) as u16;
        } else {
            let val = am.loadw(self);
            self.a = self.p.set_nz(val);
        }
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

    fn sta(&mut self, am: AddressingMode) {
        if self.p.small_acc() {
            let b = self.a as u8;
            am.storeb(self, b);
        } else {
            let w = self.a;
            am.storew(self, w);
        }
    }

    fn txs(&mut self) {
        if self.emulation {
            self.s = 0x0100 | (self.x & 0xFF);
        } else {
            self.s = self.x;
        }
    }

    fn txa(&mut self) {
        if self.p.small_acc() {
            self.a = (self.a & 0xFF00) | self.p.set_nz_8(self.x as u8) as u16;
        } else {
            self.a = self.p.set_nz(self.x);
        }
    }

    fn dex(&mut self) {
        if self.p.small_idx() {
            let res = self.p.set_nz_8((self.x as u8).wrapping_sub(1));
            self.x = (self.x & 0xFF00) | res as u16;
        } else {
            self.x = self.p.set_nz(self.x.wrapping_sub(1));
        }
    }

    fn clc(&mut self) { self.p.set_carry(false);  }
    fn sec(&mut self) { self.p.set_carry(true);  }
    fn cld(&mut self) { self.p.set_decimal(false);  }
    fn sed(&mut self) { self.p.set_decimal(true);  }
    fn cli(&mut self) { self.p.set_interrupt(false);  }
    fn sei(&mut self) { self.p.set_interrupt(true);  }
    fn clv(&mut self) { self.p.set_overflow(false);  }

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

    fn direct(&mut self) -> AddressingMode {
        AddressingMode::Direct(self.fetchb())
    }

    fn absolute(&mut self) -> AddressingMode {
        AddressingMode::Absolute(self.fetchw())
    }

    fn absolute_indexed_x(&mut self) -> AddressingMode {
        AddressingMode::AbsIndexedX(self.fetchw())
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

    fn rel(&mut self) -> AddressingMode {
        AddressingMode::Rel(self.fetchb() as i8)
    }
}