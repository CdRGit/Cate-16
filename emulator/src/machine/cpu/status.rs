#[derive(Debug)]
pub struct Status(pub u8);

const CARRY_FLAG: u8        = 1 << 0;
const ZERO_FLAG: u8         = 1 << 1;
const IRQ_FLAG: u8          = 1 << 2;
const DEC_FLAG: u8          = 1 << 3;
const SMALL_IDX_FLAG: u8    = 1 << 4;
const SMALL_ACC_FLAG: u8    = 1 << 5;
const OVERFLOW_FLAG: u8      = 1 << 6;
const NEG_FLAG: u8          = 1 << 7;

impl Status {
    pub fn new() -> Status {
        let mut s: Status = Status(0);
        s.0 = SMALL_IDX_FLAG | SMALL_ACC_FLAG | IRQ_FLAG;
        s
    }

    fn set(&mut self, flag: u8, value: bool) {
        if value {
            self.0 |= flag;
        } else {
            self.0 &= !flag;
        }
    }

    pub fn carry(&self) -> bool { (self.0 & CARRY_FLAG) != 0 }
    pub fn zero(&self) -> bool { (self.0 & ZERO_FLAG) != 0 }
    pub fn small_idx(&self) -> bool { (self.0 & SMALL_IDX_FLAG) != 0 }
    pub fn small_acc(&self) -> bool { (self.0 & SMALL_ACC_FLAG) != 0 }

    pub fn set_carry(&mut self, value: bool) { self.set(CARRY_FLAG, value); }
    pub fn set_zero(&mut self, value: bool) { self.set(ZERO_FLAG, value); }
    pub fn set_interrupt(&mut self, value: bool) { self.set(IRQ_FLAG, value); }
    pub fn set_decimal(&mut self, value: bool) { self.set(DEC_FLAG, value); }
    pub fn set_small_idx(&mut self, value: bool) { self.set(SMALL_IDX_FLAG, value); }
    pub fn set_small_acc(&mut self, value: bool) { self.set(SMALL_ACC_FLAG, value); }
    pub fn set_overflow(&mut self, value: bool) { self.set(OVERFLOW_FLAG, value); }
    pub fn set_negative(&mut self, value: bool) { self.set(NEG_FLAG, value); }

    pub fn set_nz_8(&mut self, val: u8) -> u8 {
        self.set_zero(val == 0);
        self.set_negative(val & 0x80 != 0);
        val
    }

    pub fn set_nz(&mut self, val: u16) -> u16 {
        self.set_zero(val == 0);
        self.set_negative(val & 0x8000 != 0);
        val
    }
}