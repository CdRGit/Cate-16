use super::W65C816;

pub enum AddressingMode {
    Immediate(u16),
    Immediate8(u8),

    Rel(i8),
    RelLong(i16),

    Direct(u8),
    DirectIndexedX(u8),
    DirectIndexedY(u8),
    DirectIndexedIndirect(u8),
    DirectIndirect(u8),
    DirectIndirectIndexed(u8),
    DirectIndirectLong(u8),
    DirectIndirectLongIdx(u8),

    Absolute(u16),
    AbsIndexedX(u16),
    AbsIndexedY(u16),
    AbsIndexedIndirect(u16),
    AbsLongIndexedX(u8, u16),
    AbsoluteLong(u8, u16),
    AbsoluteIndirect(u16),
    AbsoluteIndirectLong(u16),

    StackRel(u8)
}

impl AddressingMode {
    pub fn loadb(self, cpu: &mut W65C816) -> u8 {
        match self {
            AddressingMode::Immediate(_) => panic!("loadb on 16-bit immediate"),
            AddressingMode::Immediate8(val) => val,
            _ => {
                let (bank, addr) = self.address(cpu);
                cpu.loadb(bank, addr)
            }
        }
    }
    pub fn loadw(self, cpu: &mut W65C816) -> u16 {
        match self {
            AddressingMode::Immediate(val) => val,
            AddressingMode::Immediate8(_) => panic!("loadw on 8-bit immediate"),
            _ => {
                let (bank, addr) = self.address(cpu);
                cpu.loadw(bank, addr)
            }
        }
    }

    pub fn address(&self, cpu: &mut W65C816) -> (u8, u16) {
        use self::AddressingMode::*;

        match *self {
            _ => todo!()
        }
    }
}