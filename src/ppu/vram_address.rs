use std::ops;

use crate::types::{Byte, Word};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct VRAMAddress(Word);

// https://wiki.nesdev.com/w/index.php/PPU_scrolling#PPU_internal_registers
//
// yyy NN YYYYY XXXXX
// ||| || ||||| +++++-- coarse X scroll
// ||| || +++++-------- coarse Y scroll
// ||| ++-------------- nametable select
// +++----------------- fine Y scroll
impl VRAMAddress {
    fn coarse_x(&self) -> impl Into<u16> {
        self.0 & 0b11111
    }

    pub fn coarse_x_scroll(&self) -> Word {
        self.0 & 0b11111
    }

    fn coarse_y(&self) -> impl Into<u16> {
        self.0 & 0b11_11100000
    }

    pub fn coarse_y_scroll(&self) -> Word {
        self.0 & 0b11_11100000 >> 5
    }

    fn fine_y(&self) -> impl Into<u16> {
        self.0 & 0b1110000_00000000
    }

    pub fn fine_y_scroll(&self) -> Byte {
        ((self.0 & 0b1110000_00000000) >> 12).byte()
    }

    pub fn name_table_address_index(&self) -> impl Into<u16> {
        self.0 & 0b1111_11111111
    }

    fn name_table_select(&self) -> impl Into<u16> {
        self.0 & 0b1100_00000000
    }

    fn name_table_no(&self) -> impl Into<u16> {
        self.name_table_select().into() >> 10
    }
}

// Tile and attribute fetching
// https://wiki.nesdev.com/w/index.php/PPU_scrolling#Tile_and_attribute_fetching
//
// NN 1111 YYY XXX
// || |||| ||| +++-- high 3 bits of coarse X (x/4)
// || |||| +++------ high 3 bits of coarse Y (y/4)
// || ++++---------- attribute offset (960 bytes)
// ++--------------- nametable select
impl VRAMAddress {
    fn coarse_x_high(&self) -> impl Into<u16> {
        (self.0 >> 2) & 0b000111
    }

    fn coarse_y_high(&self) -> impl Into<u16> {
        (self.0 >> 4) & 0b111000
    }

    pub fn attribute_address_index(&self) -> impl Into<u16> {
        self.name_table_select().into() | self.coarse_y_high().into() | self.coarse_x_high().into()
    }
}

impl From<u16> for VRAMAddress {
    fn from(value: u16) -> Self {
        Self(Word::from(value))
    }
}

impl From<Word> for VRAMAddress {
    fn from(value: Word) -> Self {
        Self(value)
    }
}

impl From<VRAMAddress> for Word {
    fn from(value: VRAMAddress) -> Self {
        value.0
    }
}

impl ops::AddAssign<u16> for VRAMAddress {
    fn add_assign(&mut self, other: u16) {
        *self = Self(self.0 + other)
    }
}

impl ops::BitAnd<u16> for VRAMAddress {
    type Output = Self;

    fn bitand(self, rhs: u16) -> Self::Output {
        Self(self.0 & rhs)
    }
}

impl ops::BitAndAssign<u16> for VRAMAddress {
    fn bitand_assign(&mut self, rhs: u16) {
        *self = Self(self.0 & rhs)
    }
}

impl ops::BitOr for VRAMAddress {
    type Output = Self;

    fn bitor(self, Self(rhs): Self) -> Self::Output {
        Self(self.0 | rhs)
    }
}

impl ops::BitOr<u16> for VRAMAddress {
    type Output = Self;

    fn bitor(self, rhs: u16) -> Self::Output {
        Self(self.0 | rhs)
    }
}

impl ops::BitOr<Word> for VRAMAddress {
    type Output = Self;

    fn bitor(self, rhs: Word) -> Self::Output {
        Self(self.0 | rhs)
    }
}

impl ops::BitXorAssign<u16> for VRAMAddress {
    fn bitxor_assign(&mut self, rhs: u16) {
        *self = Self(self.0 ^ rhs)
    }
}
