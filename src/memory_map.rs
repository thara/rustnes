use crate::rom::Mapper;
use crate::types::{Byte, Memory, Word};

pub struct CPUBus {
    wram: [u8; 0x2000],
    mapper: Box<dyn Mapper>,
}

impl CPUBus {
    pub fn new(mapper: Box<dyn Mapper>) -> CPUBus {
        Self {
            wram: [0; 0x2000],
            mapper,
        }
    }
}

impl Memory for CPUBus {
    fn read(&self, addr: Word) -> Byte {
        let addr_u16: u16 = addr.into();
        match addr_u16 {
            0x0000..=0x1FFF => self.wram[addr_u16 as usize].into(),
            0x4020..=0xFFFF => self.mapper.read(addr),
            _ => 0.into(),
        }
    }

    fn write(&mut self, addr: Word, value: Byte) {
        let addr_u16: u16 = addr.into();
        match addr_u16 {
            0x0000..=0x1FFF => self.wram[addr_u16 as usize] = value.into(),
            0x4020..=0xFFFF => self.mapper.write(addr, value),
            _ => {}
        }
    }
}

impl Memory for [u8; 0x10000] {
    fn read(&self, addr: Word) -> Byte {
        let addr: u16 = addr.into();
        self[addr as usize].into()
    }
    fn write(&mut self, addr: Word, value: Byte) {
        let addr: u16 = addr.into();
        self[addr as usize] = value.into()
    }
}
