use super::W65C816;

#[derive(Debug)]
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

    pub fn storeb(self, cpu: &mut W65C816, value: u8) {
        let (bank, addr) = self.address(cpu);
        cpu.storeb(bank, addr, value)
    }

    pub fn storew(self, cpu: &mut W65C816, value: u16) {
        let (bank, addr) = self.address(cpu);
        cpu.storew(bank, addr, value)
    }


    pub fn address(&self, cpu: &mut W65C816) -> (u8, u16) {
        use self::AddressingMode::*;

        match *self {
            Direct(offset) => {
                (0, cpu.d.wrapping_add(offset as u16))
            }
            Absolute(addr) => {
                (cpu.dbr, addr)
            }
            AbsoluteLong(bank, addr) => {
                (bank, addr)
            }
            AbsIndexedX(offset) => {
                (cpu.dbr, offset.wrapping_add(cpu.x))
            }
            AbsIndexedY(offset) => {
                (cpu.dbr, offset.wrapping_add(cpu.y))
            }
            Rel(rel) => {
                (cpu.pbr, (cpu.pc as i16).wrapping_add(rel as i16) as u16)
            }
            DirectIndirectLong(offset) => {
                let addr_ptr = cpu.d.wrapping_add(offset as u16);
                let lo = cpu.loadb(0, addr_ptr) as u16;
                let hi = cpu.loadb(0, addr_ptr + 1) as u16;
                let bank = cpu.loadb(0, addr_ptr + 2);
                (bank, (hi << 8) | lo)
            }
            DirectIndirectLongIdx(offset) => {
                // "The 24-bit base address is pointed to by the sum of the second byte of the
                // instruction and the Direct Register. The effective address is this 24-bit base
                // address plus the Y Index Register."
                let addr_ptr = cpu.d.wrapping_add(offset as u16);
                let lo = cpu.loadb(0, addr_ptr) as u32;
                let hi = cpu.loadb(0, addr_ptr + 1) as u32;
                let bank = cpu.loadb(0, addr_ptr + 2) as u32;
                let base_address = (bank << 16) | (hi << 8) | lo;
                let eff_addr = base_address + cpu.y as u32;
                assert!(eff_addr & 0xff000000 == 0, "address overflow");

                let bank = (eff_addr >> 16) as u8;
                let addr = eff_addr as u16;
                (bank, addr)
            }
            _ => todo!()
        }
    }
}