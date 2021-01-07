pub const SPRITE_COUNT: usize = 64;
pub const SPRITE_LIMIT: usize = 8;
pub const OAM_SIZE: usize = 4 * SPRITE_COUNT;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Pixel {
    pub enabled: bool,
    pub color: u16,
    pub behide_background: bool,
}

impl Pixel {
    pub const ZERO: Self = Self {
        enabled: false,
        color: 0x00,
        behide_background: true,
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Sprite {
    // Y position of top
    pub y: u8,
    // Tile index number
    pub tile_index: u8,
    // Attributes
    pub attr: SpriteAttribute,
    // X position of left
    pub x: u8,
}

impl Sprite {
    pub fn valid(&self) -> bool {
        !(self.x == 0xFF && self.y == 0xFF && self.tile_index == 0xFF && self.attr.0 == 0xFF)
    }

    pub fn row(&self, line: u16, sprite_height: i8) -> u16 {
        let row = (line as u16).wrapping_sub(self.y as u16).wrapping_sub(1);
        if self.attr.is_set(SpriteAttribute::FLIP_VERTICALLY) {
            (sprite_height as u16).wrapping_sub(1).wrapping_sub(row)
        } else {
            row
        }
    }

    pub fn col(&self, x: u16) -> u8 {
        let col = 7u16.wrapping_sub(x.wrapping_sub(self.x as u16));
        if self.attr.is_set(SpriteAttribute::FLIP_HORIZONTALLY) {
            8u16.wrapping_sub(1).wrapping_sub(col) as u8
        } else {
            col as u8
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct SpriteAttribute(u8);

impl SpriteAttribute {
    const FLIP_VERTICALLY: Self = Self(1 << 7);
    const FLIP_HORIZONTALLY: Self = Self(1 << 6);
    // Priority
    pub const BEHIND_BACKGROUND: Self = Self(1 << 5);

    // Palette
    #[allow(dead_code)]
    const PALLETE_2: Self = Self(1 << 1);
    #[allow(dead_code)]
    const PALLETE_1: Self = Self(1);

    pub fn pallete(&self) -> u8 {
        self.0 & 0b11
    }

    pub fn is_set(&self, Self(v): Self) -> bool {
        self.0 & v == v
    }
}

impl From<u8> for SpriteAttribute {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
