use crate::types::{Byte, Word};

pub(super) const NAME_TABLE_FIRST: Word = Word::new(0x2000u16);
pub(super) const ATTRIBUTE_TABLE_FIRST: Word = Word::new(0x23C0u16);
pub(super) const TILE_HEIGHT: Byte = Byte::new(8);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub(super) struct Pixel {
    pub enabled: bool,
    pub color: u16,
}

impl Pixel {
    pub const ZERO: Self = Self {
        enabled: false,
        color: 0x00,
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub(super) struct Tile {
    pattern: TilePattern,
    attr: TileAttribute,
}

impl Tile {
    pub fn pixel_pallete(&self, x: u8) -> (Word, Word) {
        // http://wiki.nesdev.com/w/index.php/PPU_palettes#Memory_Map
        let p = 15u8.wrapping_sub(x);
        let pixel = (self.pattern.high.nth(p) << 1) | self.pattern.low.nth(p);

        let a = 15u8.wrapping_sub(x);
        let attr = (self.attr.high.nth(a) << 1) | self.attr.low.nth(a);

        (pixel.into(), attr.into())
    }

    pub fn shift(&mut self) {
        self.pattern.low <<= 1;
        self.pattern.high <<= 1;

        self.attr.low = (self.attr.low << 1) | if self.attr.low_latch { 1 } else { 0 };
        self.attr.high = (self.attr.high << 1) | if self.attr.high_latch { 1 } else { 0 };
    }

    pub fn reload(&mut self, next_ptn: TilePattern, next_attr: Byte) {
        self.pattern.low = (self.pattern.low & 0xFF00) | next_ptn.low;
        self.pattern.high = (self.pattern.high & 0xFF00) | next_ptn.high;
        self.attr.low_latch = next_attr.nth(0) == 1;
        self.attr.high_latch = next_attr.nth(1) == 1;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub(super) struct TilePattern {
    pub low: Word,
    pub high: Word,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
struct TileAttribute {
    low: Byte,
    high: Byte,

    // 1 quadrant of attrTableEntry
    low_latch: bool,
    high_latch: bool,
}
