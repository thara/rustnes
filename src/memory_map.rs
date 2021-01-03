use crate::types::{Byte, Memory, Word};

pub struct CPUBus {
    wram: [u8; 0x2000],
}

impl CPUBus {
    pub fn new() -> CPUBus {
        Self { wram: [0; 0x2000] }
    }
}

impl Memory for CPUBus {
    fn read(&self, addr: Word) -> Byte {
        let addr: u16 = addr.into();
        match addr {
            0x0000..=0x1FFF => self.wram[addr as usize],
            _ => 0x00,
        }
        .into()
    }

    fn write(&mut self, addr: Word, value: Byte) {
        let addr: u16 = addr.into();
        match addr {
            0x0000..=0x1FFF => self.wram[addr as usize] = value.into(),
            _ => {}
        }
    }
}

#[cfg(test)]
pub mod test_util {
    use crate::types::{Byte, Memory, Word};

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
}
