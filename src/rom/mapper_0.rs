use crate::types::{Byte, Memory, Mirroring, Word};

use super::nesfile::{NESFile, NESFileHeader};
use super::Mapper;

pub struct Mapper0 {
    prg: Vec<u8>,
    chr: Vec<u8>,
    mirroring: Mirroring,
    mirrored: bool,
}

impl Mapper0 {
    pub fn new(rom: NESFile) -> Self {
        let (prg, next) = rom.read_prg_rom(NESFileHeader::SIZE, 0x4000);
        let chr = if let Some((prg, _)) = rom.read_chr_rom(next, 0x2000) {
            prg
        } else {
            [0; 0x2000].into()
        };
        let mirrored = prg.len() == 0x4000;
        Self {
            prg,
            chr,
            mirroring: rom.mirroring(),
            mirrored,
        }
    }

    fn prg_addr(&self, base: u16) -> usize {
        let addr = if self.mirrored {
            base % 0x4000
        } else {
            base.wrapping_sub(0x8000)
        };
        addr as usize
    }
}

impl Memory for Mapper0 {
    fn read(&self, addr: Word) -> Byte {
        let addr: u16 = addr.into();
        match addr {
            0x0000..=0x1FFF => self.chr[addr as usize],
            0x8000..=0xFFFF => self.prg[self.prg_addr(addr)],
            _ => 0,
        }
        .into()
    }

    fn write(&mut self, addr: Word, value: Byte) {
        let addr: u16 = addr.into();
        if let 0x0000..=0x1FFF = addr {
            self.chr[addr as usize] = value.into()
        }
    }
}

impl Mapper for Mapper0 {
    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }
}
