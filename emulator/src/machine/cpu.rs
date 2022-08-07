pub type CPU_FLAGS = u8;

const CPU_C = 1 << 0;
const CPU_Z = 1 << 1;
const CPU_I = 1 << 2;
const CPU_D = 1 << 3;
const CPU_X = 1 << 4;
const CPU_M = 1 << 5;
const CPU_V = 1 << 6;
const CPU_N = 1 << 7;

#[derive(Debug)]
pub struct W65C816 {
    aLo: u8, aHi: u8,
    xLo: u8, xHi: u8,
    yLo: u8, yHi: u8,
    sLo: u8, sHi: u8,

    dbr: u8, dp: u16,

    pck: u8, pc: u16,

    emu: bool,
    flags: CPU_FLAGS,
}

impl W65C816 {
    pub fn new() -> Self {
        W65C816 {
            aLo: 0, aHi: 0,
            xLo: 0, xHi: 0,
            yLo: 0, yHi: 0,
            sLo: 0, sHi: 0,
            dbr: 0, dp:  0,
            pck: 0, pc:  0,
            emu: true,
            flags: CPU_I,
        }
    }
}