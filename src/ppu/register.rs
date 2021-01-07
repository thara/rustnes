use crate::types::{Byte, Word};
use std::ops;

use super::vram_address::VRAMAddress;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub(super) struct Register {
    // PPUCTRL
    pub(super) controller: Controller,
    // PPUMASK
    pub(super) mask: Mask,
    // PPUSTATUS
    pub(super) status: Status,
    // PPUDATA
    pub(super) data: Byte,
    // OAMADDR
    pub(super) object_attribute_memory_address: usize,

    // current VRAM address
    pub(super) v: VRAMAddress,
    // temporary VRAM address
    t: VRAMAddress,
    // Fine X scroll
    pub fine_x: Byte,
    write_toggle: bool,
}

impl Register {
    pub fn reset(&mut self) {
        self.controller = Controller(0);
        self.mask = Mask(0);
        self.status = Status(0);
        self.data = 0x00.into();
    }

    pub fn sprite_size(&self) -> i8 {
        if self.controller.is_set(Controller::SPRITE_SIZE) {
            16
        } else {
            8
        }
    }

    pub fn rendering_enabled(&self) -> bool {
        self.mask.is_set(Mask::SPRITE) || self.mask.is_set(Mask::BACKGROUND)
    }

    pub fn is_enabled_background(&self, x: u16) -> bool {
        self.mask.is_set(Mask::BACKGROUND) && !(x < 8 && self.mask.is_set(Mask::BACKGROUND_LEFT))
    }

    pub fn is_enabled_sprite(&self, x: i32) -> bool {
        self.mask.is_set(Mask::SPRITE) && !(x < 8 && self.mask.is_set(Mask::SPRITE_LEFT))
    }

    pub fn incr_v(&mut self) {
        self.v += self.controller.vram_increment()
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#.242000_write
    pub fn write_controller(&mut self, value: impl Into<u8>) {
        self.controller = Controller(value.into());
        // t: ...BA.. ........ = d: ......BA
        self.t = (self.t & !0b0001100_00000000) | (self.controller.name_table_select() << 10)
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#.242002_read
    pub fn read_status(&mut self) -> Byte {
        let s = self.status;
        self.status.unset(Status::VBLANK);
        self.write_toggle = false;
        s.0.into()
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#.242005_first_write_.28w_is_0.29
    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#.242005_second_write_.28w_is_1.29
    pub fn write_scroll(&mut self, position: impl Into<u8>) {
        let p = position.into();
        if !self.write_toggle {
            // first write
            // t: ....... ...HGFED = d: HGFED...
            // x:              CBA = d: .....CBA
            self.t = (self.t & !0b0000000_00011111) | ((p as u16 & 0b11111000) >> 3);
            self.fine_x = Byte::from(p & 0b111);
            self.write_toggle = true;
        } else {
            // second write
            // t: CBA..HG FED..... = d: HGFEDCBA
            self.t = (self.t & !0b1110011_11100000)
                | ((p as u16 & 0b111) << 12)
                | ((p as u16 & 0b11111000) << 2);
            self.write_toggle = false
        }
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#.242006_first_write_.28w_is_0.29
    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#.242006_second_write_.28w_is_1.29
    pub fn write_vram_address(&mut self, addr: impl Into<u8>) {
        let d = addr.into();
        if !self.write_toggle {
            // first write
            // t: .FEDCBA ........ = d: ..FEDCBA
            // t: X...... ........ = 0
            self.t = (self.t & !0b0111111_00000000) | ((d as u16 & 0b111111) << 8);
            self.write_toggle = true
        } else {
            // second write
            // t: ....... HGFEDCBA = d: HGFEDCBA
            // v                   = t
            self.t = (self.t & !0b0000000_11111111) | d as u16;
            self.v = self.t.into();
            self.write_toggle = false
        }
    }

    pub fn incr_coarse_x(&mut self) {
        if self.v.coarse_x_scroll() == 31u16.into() {
            self.v &= !0b11111; // coarse X = 0
            self.v ^= 0x0400; // switch horizontal nametable
        } else {
            self.v += 1;
        }
    }

    pub fn incr_y(&mut self) {
        if self.v.fine_y_scroll() < 7.into() {
            self.v += 0x1000;
        } else {
            self.v &= !0x7000; // fine Y = 0

            let mut y: u16 = self.v.coarse_y_scroll().into();
            if y == 29 {
                y = 0;
                self.v ^= 0x0800; // switch vertical nametable
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }

            self.v = (self.v & !0x03E0) | (y << 5);
        }
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#At_dot_257_of_each_scanline
    pub fn copy_x(&mut self) {
        // v: ....F.. ...EDCBA = t: ....F.. ...EDCBA
        self.v = (self.v & !0b100_00011111) | (self.t & 0b100_00011111)
    }

    // http://wiki.nesdev.com/w/index.php/PPU_scrolling#During_dots_280_to_304_of_the_pre-render_scanline_.28end_of_vblank.29
    pub fn copy_y(&mut self) {
        // v: IHGF.ED CBA..... = t: IHGF.ED CBA.....
        self.v = (self.v & !0b1111011_11100000) | (self.t & 0b1111011_11100000)
    }

    #[allow(dead_code)]
    fn background_pattern_table_addr_base(&self) -> impl Into<u16> {
        if self.controller.is_set(Controller::BG_TABLE_ADDR) {
            0x1000u16
        } else {
            0x0000u16
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Controller(u8);

impl Controller {
    // NMI
    pub const NMI: Self = Self(1 << 7);
    // PPU master/slave (0: master, 1: slave)
    #[allow(dead_code)]
    pub const SLAVE: Self = Self(1 << 6);
    // Sprite size
    pub const SPRITE_SIZE: Self = Self(1 << 5);
    // Background pattern table address
    pub const BG_TABLE_ADDR: Self = Self(1 << 4);
    // Sprite pattern table address for 8x8 sprites
    pub const SPRITE_TABLE_ADDR: Self = Self(1 << 3);
    // VRAM address increment
    pub const VRAM_ADDR_INCR: Self = Self(1 << 2);
    // Base nametable address
    #[allow(dead_code)]
    pub const NAME_TABLE_ADDR_HIGH: Self = Self(1 << 1);
    #[allow(dead_code)]
    pub const NAME_TABLE_ADDR_LOW: Self = Self(1 << 0);

    pub fn is_set(&self, Self(v): Self) -> bool {
        self.0 & v == v
    }

    fn name_table_select(&self) -> Word {
        (self.0 & 0b11).into()
    }

    #[allow(dead_code)]
    fn bg_pattern_table_addr_base(&self) -> Word {
        if self.is_set(Self::BG_TABLE_ADDR) {
            0x1000u16
        } else {
            0x0000u16
        }
        .into()
    }

    #[allow(dead_code)]
    pub fn base_name_table_addr(&self) -> Word {
        match self.name_table_select().into() {
            0u16 => 0x2000u16,
            1u16 => 0x2400u16,
            2u16 => 0x2800u16,
            3u16 => 0x2C00u16,
            _ => panic!("unrecognized name table select"),
        }
        .into()
    }

    pub fn base_sprite_table_addr(&self) -> u16 {
        if self.is_set(Self::SPRITE_TABLE_ADDR) {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn sprite_8x16_pixels(&self) -> bool {
        self.is_set(Self::SPRITE_SIZE)
    }

    fn vram_increment(&self) -> u16 {
        if self.is_set(Self::VRAM_ADDR_INCR) {
            32u16
        } else {
            1u16
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Mask(u8);

impl Mask {
    // Emphasize blue
    #[allow(dead_code)]
    const BLUE: Self = Self(1 << 7);
    // Emphasize green
    #[allow(dead_code)]
    const GREEN: Self = Self(1 << 6);
    // Emphasize red
    #[allow(dead_code)]
    const RED: Self = Self(1 << 5);
    // Show sprite
    const SPRITE: Self = Self(1 << 4);
    // Show background
    const BACKGROUND: Self = Self(1 << 3);
    // Show sprite in leftmost 8 pixels
    const SPRITE_LEFT: Self = Self(1 << 2);
    // Show background in leftmost 8 pixels
    const BACKGROUND_LEFT: Self = Self(1 << 1);
    // Greyscale
    #[allow(dead_code)]
    const GREYSCALE: Self = Self(1);

    pub fn new(v: impl Into<u8>) -> Self {
        Self(v.into())
    }

    pub fn is_set(&self, Self(v): Self) -> bool {
        self.0 & v == v
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Status(u8);

impl Status {
    // In vblank?
    pub const VBLANK: Self = Self(1 << 7);
    // Sprite 0 Hit
    pub const SPRITE_ZERO_HIT: Self = Self(1 << 6);
    // Sprite overflow
    pub const SPRITE_OVERFLOW: Self = Self(1 << 5);

    pub fn is_set(&self, Self(v): Self) -> bool {
        self.0 & v == v
    }

    pub fn set(&mut self, s: Self) {
        self.0 |= s.0
    }

    pub fn unset(&mut self, Self(s): Self) {
        self.0 &= !s
    }
}

impl ops::BitOr for Status {
    type Output = Self;

    fn bitor(self, Self(rhs): Self) -> Self::Output {
        Self(self.0 | rhs)
    }
}
