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

    fn pushb(&mut self, value: u8) {
        let s = self.s;
        self.storeb(0, s, value);
        if self.emulation {
            // stack must stay in 0x01xx
            assert_eq!(self.s & 0xff00, 0x0100);
            let s = self.s as u8 - 1;
            self.s = (self.s & 0xff00) | s as u16;
        } else {
            self.s -= 1;
        }
    }

    fn pushw(&mut self, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = value as u8;
        self.pushb(hi);
        self.pushb(lo);
    }

    fn popb(&mut self) -> u8 {
        if self.emulation {
            // stack must stay in 0x01xx
            assert_eq!(self.s & 0xff00, 0x0100);
            let s = self.s as u8 + 1;
            self.s = (self.s & 0xff00) | s as u16;
        } else {
            self.s += 1;
        }

        let s = self.s;
        self.loadb(0, s)
    }

    fn popw(&mut self) -> u16 {
        let lo = self.popb() as u16;
        let hi = self.popb() as u16;
        (hi << 8) | lo
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
        let opcode = self.fetchb();

        macro_rules! instr {
            ( $name:ident ) => {{
                //println!("{:02X}{:04X} {}", self.pbr, self.pc.wrapping_sub(1), stringify!($name));
                self.$name();
            }};
            ( $name:ident $am:ident ) => {{
                //println!("{:02X}{:04X} {}", self.pbr, self.pc.wrapping_sub(1), stringify!($name));
                let am = self.$am();
                self.$name(am);
            }};
        }

        match opcode {
            // math
            0xE9 => instr!( sbc immediate_acc ),

            0x0A => instr!( asl_a ),
            0x4A => instr!( lsr_a ),

            // logic
            0x29 => instr!( and immediate_acc ),

            0x05 => instr!( ora direct ),
            0x09 => instr!( ora immediate_acc ),

            // branches
            0x80 => instr!( bra rel ),
            0x82 => instr!( bra relative_long ),

            0xF0 => instr!( beq rel ),
            0x30 => instr!( bmi rel ),
            0xD0 => instr!( bne rel ),

            // jumps
            0x4C => instr!( jmp absolute ),
            0x5C => instr!( jml absolute_long ),

            0x20 => instr!( jsr absolute ),
            0x22 => instr!( jsl absolute_long ),

            0x60 => instr!( rts ),
            0x6B => instr!( rtl ),

            // comparisons
            0xCD => instr!( cmp absolute ),
            0xDD => instr!( cmp absolute_indexed_x ),
            0xC9 => instr!( cmp immediate_acc ),

            0xEC => instr!( cpx absolute ),
            0xE0 => instr!( cpx immediate_index ),

            0xC0 => instr!( cpy immediate_index ),

            // register load + store
            0xA9 => instr!( lda immediate_acc ),

            0xA5 => instr!( lda direct ),
            0xA7 => instr!( lda direct_indirect_long ),
            0xB7 => instr!( lda direct_indirect_long_idx ),
            0xAD => instr!( lda absolute ),
            0xBD => instr!( lda absolute_indexed_x ),
            0xBF => instr!( lda absolute_long_indexed_x ),

            0xA2 => instr!( ldx immediate_index ),

            0xA6 => instr!( ldx direct ),
            0xAE => instr!( ldx absolute ),
            
            0xA0 => instr!( ldy immediate_index ),

            0xA4 => instr!( ldy direct ),

            0x85 => instr!( sta direct ),
            0x8D => instr!( sta absolute ),
            0x9D => instr!( sta absolute_indexed_x ),
            0x99 => instr!( sta absolute_indexed_y ),

            0x97 => instr!( sta direct_indirect_long_idx ),

            0x86 => instr!( stx direct ),
            0x8E => instr!( stx absolute ),

            0x84 => instr!( sty direct ),
            0x8C => instr!( sty absolute ),

            // register transfers
            0xAA => instr!( tax ),
            0x8A => instr!( txa ),
            0x9A => instr!( txs ),
            0xBB => instr!( tyx ),

            0xEB => instr!( xba ),

            // stack manipulation
            0x48 => instr!( pha ),
            0xDA => instr!( phx ),
            0x5A => instr!( phy ),

            0x68 => instr!( pla ),
            0xFA => instr!( plx ),
            0x7A => instr!( ply ),

            // increment/decrement
            0xE8 => instr!( inx ),
            0xCA => instr!( dex ),

            0xC8 => instr!( iny ),
            0x88 => instr!( dey ),

            0xEE => instr!( inc absolute ),

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

    fn sbc(&mut self, am: AddressingMode) {
        // Sets N, Z, C and V
        let c: i16 = if self.p.carry() { 1 } else { 0 };

        if self.p.small_acc() {
            let a = self.a as i16 & 0xff;
            let v = am.loadb(self) as i16 ^ 0xff;
            let mut res: i16 = if self.p.decimal() {
                let mut low: i16 = (a & 0x0f) + (v & 0x0f) + c;
                if low < 0x10 { low -= 6; }

                (a & 0xf0) + (v & 0xf0) + (low & 0x0f) + if low > 0x0f { 0x10 } else { 0x00 }
            } else {
                a + v + c
            };
            self.p.set_overflow((a & 0x80) == (v & 0x80) && (a & 0x80) != (res & 0x80));
            if self.p.decimal() && res < 0x100 { res -= 0x60; }
            self.p.set_carry(res > 255);

            self.a = (self.a & 0xff00) | self.p.set_nz_8(res as u8) as u16;
        } else {
            let a = self.a as i32;
            let v = am.loadw(self) as i32 ^ 0xffff;
            let mut res: i32 = if self.p.decimal() {
                let mut res0 = (a & 0x000f) + (v & 0x000f) + c as i32;
                if res0 < 0x0010 { res0 -= 0x0006; }

                let mut res1 = (a & 0x00f0) + (v & 0x00f0) + (res0 & 0x000f) +
                    if res0 > 0x000f { 0x10 } else { 0x00 };
                if res1 < 0x0100 { res1 -= 0x0060; }

                let mut res2 = (a & 0x0f00) + (v & 0x0f00) + (res1 & 0x00ff) +
                    if res1 > 0x00ff { 0x100 } else { 0x000 };
                if res2 < 0x1000 { res2 -= 0x0600; }

                (a as i32 & 0xf000) + (v as i32 & 0xf000) + (res2 as i32 & 0x0fff) +
                    if res2 > 0x0fff { 0x1000 } else { 0x0000 }
            } else {
                self.a as i32 + v as i32 + c as i32
            };
            self.p.set_overflow((self.a ^ res as u16) & 0x8000 != 0 && (self.a ^ v as u16) & 0x8000 == 0);
            if self.p.decimal() && res < 0x10000 { res -= 0x6000; }
            self.p.set_carry(res > 65535);

            self.a = self.p.set_nz(res as u16);
        }
    }


    fn asl_a(&mut self) {
        // Sets N, Z and C. The rightmost bit is filled with 0.
        if self.p.small_acc() {
            let a = self.a as u8;
            self.p.set_carry(self.a & 0x80 != 0);
            self.a = (self.a & 0xff00) | self.p.set_nz_8(a << 1) as u16;
        } else {
            self.p.set_carry(self.a & 0x8000 != 0);
            self.a = self.p.set_nz(self.a << 1);
        }
    }

    fn lsr_a(&mut self) {
        // Sets N (always cleared), Z and C. The leftmost bit is filled with 0.
        if self.p.small_acc() {
            let a = self.a as u8;
            self.p.set_carry(self.a & 0x01 != 0);
            self.a = (self.a & 0xff00) | self.p.set_nz_8(a >> 1) as u16;
        } else {
            self.p.set_carry(self.a & 0x0001 != 0);
            self.a = self.p.set_nz(self.a >> 1);
        }
    }

    fn and(&mut self, am: AddressingMode) {
        if self.p.small_acc() {
            let val = am.loadb(self);
            let res = self.a as u8 & val;
            self.p.set_nz_8(res);
            self.a = (self.a & 0xFF00) | res as u16;
        } else {
            let val = am.loadw(self);
            let res = self.a & val;
            self.a = self.p.set_nz(res);
        }
    }

    fn ora(&mut self, am: AddressingMode) {
        if self.p.small_acc() {
            let val = am.loadb(self);
            let res = self.a as u8 | val;
            self.p.set_nz_8(res);
            self.a = (self.a & 0xFF00) | res as u16;
        } else {
            let val = am.loadw(self);
            let res = self.a | val;
            self.a = self.p.set_nz(res);
        }
    }

    fn bra(&mut self, am: AddressingMode) {
        let a = am.address(self);
        self.branch(a);
    }

    fn beq(&mut self, am: AddressingMode) {
        let a = am.address(self);
        if self.p.zero() {
            self.branch(a);
        }
    }

    fn bmi(&mut self, am: AddressingMode) {
        let a = am.address(self);
        if self.p.negative() {
            self.branch(a);
        }
    }

    fn bne(&mut self, am: AddressingMode) {
        let a = am.address(self);
        if !self.p.zero() {
            self.branch(a);
        }
    }

    fn jmp(&mut self, am: AddressingMode) {
        let (_, addr) = am.address(self);
        self.pc = addr;
    }

    fn jml(&mut self, am: AddressingMode) {
        let a = am.address(self);
        self.branch(a);
    }

    fn jsr(&mut self, am: AddressingMode) {
        let pc = self.pc - 1;
        self.pushb((pc >> 8) as u8);
        self.pushb(pc as u8);

        self.pc = am.address(self).1;
    }

    fn jsl(&mut self, am: AddressingMode) {
        let pbr = self.pbr;
        self.pushb(pbr);
        let pc = self.pc - 1;
        self.pushb((pc >> 8) as u8);
        self.pushb(pc as u8);

        let (pbr, pc) = am.address(self);
        self.pbr = pbr;
        self.pc = pc;
    }

    fn rts(&mut self) {
        let pcl = self.popb() as u16;
        let pch = self.popb() as u16;
        let pc = (pch << 8) | pcl;
        self.pc = pc + 1;   // +1 since the last byte of the JSR was saved
    }

    fn rtl(&mut self) {
        let pcl = self.popb() as u16;
        let pch = self.popb() as u16;
        let pbr = self.popb();
        let pc = (pch << 8) | pcl;
        self.pbr = pbr;
        self.pc = pc + 1;   // +1 since the last byte of the JSL was saved
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

    fn cpy(&mut self, am: AddressingMode) {
        if self.p.small_idx() {
            let y = self.y as u8;
            let val = am.loadb(self);
            self.compare8(y, val);
        } else {
            let y = self.y;
            let val = am.loadw(self);
            self.compare(y, val);
        }
    }

    fn lda(&mut self, am: AddressingMode) {
        if self.p.small_acc() {
            let val = am.loadb(self);
            self.a = (self.a & 0xFF00) | self.p.set_nz_8(val) as u16;
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

    fn ldy(&mut self, am: AddressingMode) {
        if self.p.small_idx() {
            let val = am.loadb(self);
            self.y = self.p.set_nz_8(val) as u16;
        } else {
            let val = am.loadw(self);
            self.y = self.p.set_nz(val);
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

    fn stx(&mut self, am: AddressingMode) {
        if self.p.small_idx() {
            let b = self.x as u8;
            am.storeb(self, b);
        } else {
            let w = self.x;
            am.storew(self, w);
        }
    }

    fn sty(&mut self, am: AddressingMode) {
        if self.p.small_idx() {
            let b = self.y as u8;
            am.storeb(self, b);
        } else {
            let w = self.y;
            am.storew(self, w);
        }
    }

    fn tax(&mut self) {
        if self.p.small_idx() {
            self.x = (self.x & 0xFF00) | self.p.set_nz_8(self.a as u8) as u16;
        } else {
            self.x = self.p.set_nz(self.a);
        }
    }

    fn txa(&mut self) {
        if self.p.small_acc() {
            self.a = (self.a & 0xFF00) | self.p.set_nz_8(self.x as u8) as u16;
        } else {
            self.a = self.p.set_nz(self.x);
        }
    }

    fn tyx(&mut self) {
        if self.p.small_idx() {
            self.x = (self.x & 0xFF00) | self.p.set_nz_8(self.y as u8) as u16;
        } else {
            self.x = self.p.set_nz(self.y);
        }
    }

    fn txs(&mut self) {
        if self.emulation {
            self.s = 0x0100 | (self.x & 0xFF);
        } else {
            self.s = self.x;
        }
    }

    fn xba(&mut self) {
        // Changes N and Z: "The flags are changed based on the new value of the low byte, the A
        // accumulator (that is, on the former value of the high byte, the B accumulator), even in
        // sixteen-bit accumulator mode."
        let lo = self.a & 0xff;
        let hi = self.a >> 8;
        self.a = (lo << 8) | self.p.set_nz_8(hi as u8) as u16;
    }

    fn pha(&mut self) {
        if self.p.small_acc() {
            let a = self.a as u8;
            self.pushb(a);
        } else {
            let a = self.a;
            self.pushw(a);
        }
    }

    fn pla(&mut self) {
        if self.p.small_acc() {
            let a = self.popb();
            self.a = (self.a & 0xFF00) | self.p.set_nz_8(a) as u16;
        } else {
            let a = self.popw();
            self.a = self.p.set_nz(a);
        }
    }

    fn phx(&mut self) {
        if self.p.small_idx() {
            let a = self.x as u8;
            self.pushb(a);
        } else {
            let a = self.x;
            self.pushw(a);
        }
    }

    fn plx(&mut self) {
        if self.p.small_idx() {
            let a = self.popb();
            self.x = (self.x & 0xFF00) | self.p.set_nz_8(a) as u16;
        } else {
            let a = self.popw();
            self.x = self.p.set_nz(a);
        }
    }

    fn phy(&mut self) {
        if self.p.small_idx() {
            let a = self.y as u8;
            self.pushb(a);
        } else {
            let a = self.y;
            self.pushw(a);
        }
    }

    fn ply(&mut self) {
        if self.p.small_idx() {
            let a = self.popb();
            self.y = (self.y & 0xFF00) | self.p.set_nz_8(a) as u16;
        } else {
            let a = self.popw();
            self.y = self.p.set_nz(a);
        }
    }

    fn inx(&mut self) {
        if self.p.small_idx() {
            println!("[inx] SMALL_IDX");
            let res = self.p.set_nz_8((self.x as u8).wrapping_add(1));
            self.x = (self.x & 0xFF00) | res as u16;
        } else {
            self.x = self.p.set_nz(self.x.wrapping_add(1));
        }
    }

    fn iny(&mut self) {
        if self.p.small_idx() {
            println!("[inx] SMALL_IDX");
            let res = self.p.set_nz_8((self.y as u8).wrapping_add(1));
            self.y = (self.y & 0xFF00) | res as u16;
        } else {
            self.y = self.p.set_nz(self.y.wrapping_add(1));
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

    fn dey(&mut self) {
        if self.p.small_idx() {
            let res = self.p.set_nz_8((self.y as u8).wrapping_sub(1));
            self.y = (self.y & 0xFF00) | res as u16;
        } else {
            self.y = self.p.set_nz(self.y.wrapping_sub(1));
        }
    }

    fn inc(&mut self, am: AddressingMode) {
        let (bank, addr) = am.address(self);
        if self.p.small_acc() {
            let res = self.loadb(bank, addr).wrapping_add(1);
            self.p.set_nz_8(res);
            self.storeb(bank, addr, res);
        } else {
            let res = self.loadw(bank, addr).wrapping_add(1);
            self.p.set_nz(res);
            self.storew(bank, addr, res);
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

    fn absolute_indexed_y(&mut self) -> AddressingMode {
        AddressingMode::AbsIndexedY(self.fetchw())
    }

    fn absolute_long(&mut self) -> AddressingMode {
        let addr = self.fetchw();
        let bank = self.fetchb();
        AddressingMode::AbsoluteLong(bank, addr)
    }
    
    fn absolute_long_indexed_x(&mut self) -> AddressingMode {
        let addr = self.fetchw();
        let bank = self.fetchb();
        AddressingMode::AbsLongIndexedX(bank, addr)
    }

    fn direct_indirect_long(&mut self) -> AddressingMode {
        AddressingMode::DirectIndirectLong(self.fetchb())
    }

    fn direct_indirect_long_idx(&mut self) -> AddressingMode {
        AddressingMode::DirectIndirectLongIdx(self.fetchb())
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
    fn relative_long(&mut self) -> AddressingMode {
        AddressingMode::RelLong(self.fetchw() as i16)
    }
}