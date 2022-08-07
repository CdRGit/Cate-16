use super::bus::*;

pub struct Status(pub u8);

const CARRY_FLAG: u8        = 1 << 0;
const ZERO_FLAG: u8         = 1 << 1;
const IRQ_FLAG: u8          = 1 << 2;
const DEC_FLAG: u8          = 1 << 3;
const SMALL_IDX_FLAG: u8    = 1 << 4;
const SMALL_ACC_FLAG: u8    = 1 << 5;
const OVEFLOW_FLAG: u8      = 1 << 6;
const NEG_FLAG: u8          = 1 << 7;

impl Status {
    pub fn new() -> Status {
        Status(SMALL_IDX_FLAG | SMALL_ACC_FLAG | IRQ_FLAG)
    }
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

    emu: bool,
    flags: Status,

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
            emu: true,
            flags: Status::new(),
            bus,
        }
    }
}

