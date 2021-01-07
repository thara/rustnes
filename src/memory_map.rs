use std::cell::RefCell;
use std::rc::Rc;

use crate::rom::Mapper;
use crate::types::{Byte, Memory, Mirroring, Word};

use crate::ppu::PPU;

pub struct CPUBus {
    wram: [u8; 0x2000],
    mapper: Rc<RefCell<dyn Mapper>>,

    ppu: Rc<RefCell<PPU>>,
}

impl CPUBus {
    pub fn new(mapper: Rc<RefCell<dyn Mapper>>, ppu: Rc<RefCell<PPU>>) -> CPUBus {
        Self {
            wram: [0; 0x2000],
            mapper,
            ppu,
        }
    }
}

fn to_ppu_addr(addr: u16) -> u16 {
    // repears every 8 bytes
    0x2000u16.wrapping_add(addr) % 8
}

impl Memory for CPUBus {
    fn read(&self, addr: Word) -> Byte {
        let addr_u16: u16 = addr.into();
        match addr_u16 {
            0x0000..=0x1FFF => self.wram[addr_u16 as usize].into(),
            0x2000..=0x3FFF => self.ppu.borrow_mut().read_register(to_ppu_addr(addr_u16)),
            0x4020..=0xFFFF => self.mapper.borrow().read(addr),
            _ => 0.into(),
        }
    }

    fn write(&mut self, addr: Word, value: Byte) {
        let addr_u16: u16 = addr.into();
        match addr_u16 {
            0x0000..=0x1FFF => self.wram[addr_u16 as usize] = value.into(),
            0x2000..=0x3FFF => self
                .ppu
                .borrow_mut()
                .write_register(to_ppu_addr(addr_u16), value),
            0x4020..=0xFFFF => self.mapper.borrow_mut().write(addr, value),
            _ => {}
        }
    }
}

pub struct PPUBus {
    name_table: [Byte; 0x1000],
    pallete_ram_idx: [Byte; 0x0020],

    mapper: Rc<RefCell<dyn Mapper>>,
    mirroring: Mirroring,
}

impl PPUBus {
    pub fn new(mapper: Rc<RefCell<dyn Mapper>>) -> Self {
        let mirroring = mapper.borrow().mirroring();
        Self {
            name_table: [Default::default(); 0x1000],
            pallete_ram_idx: [Default::default(); 0x0020],
            mapper,
            mirroring,
        }
    }

    fn to_name_table_address(&self, base: u16) -> usize {
        match self.mirroring {
            Mirroring::Vertical() => base & 0x0800,
            Mirroring::Horizontal() => {
                if 0x2800 <= base {
                    0x0800u16.wrapping_add(base) % 0x0400
                } else {
                    base % 0x0400
                }
            }
        }
        .into()
    }

    fn to_pallete_address(&self, base: u16) -> usize {
        // http://wiki.nesdev.com/w/index.php/PPU_palettes#Memory_Map
        let addr = base % 32;
        if addr % 4 == 0 { addr | 0x10 } else { addr }.into()
    }
}

impl Memory for PPUBus {
    fn read(&self, addr: Word) -> Byte {
        let addr_u16: u16 = addr.into();
        match addr_u16 {
            0x0000..=0x1FFF => self.mapper.borrow().read(addr),
            0x2000..=0x2FFF => self.name_table[self.to_name_table_address(addr_u16)],
            0x3000..=0x3EFF => self.name_table[self.to_name_table_address(addr_u16 - 0x1000)],
            0x3F00..=0x3FFF => self.pallete_ram_idx[self.to_pallete_address(addr_u16)],
            _ => 0.into(),
        }
    }

    fn write(&mut self, addr: Word, value: Byte) {
        let addr_u16: u16 = addr.into();
        match addr_u16 {
            0x0000..=0x1FFF => self.mapper.borrow_mut().write(addr, value),
            0x2000..=0x2FFF => self.name_table[self.to_name_table_address(addr_u16)] = value,
            0x3000..=0x3EFF => {
                self.name_table[self.to_name_table_address(addr_u16 - 0x1000)] = value;
            }
            0x3F00..=0x3FFF => self.pallete_ram_idx[self.to_pallete_address(addr_u16)] = value,
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
