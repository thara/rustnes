mod background;
mod register;
mod sprite;
mod vram_address;

use crate::interrupt::Interrupt;
use crate::types::{Byte, Memory, Word};

use background::{ATTRIBUTE_TABLE_FIRST, NAME_TABLE_FIRST, TILE_HEIGHT};
use register::{Controller, Mask, Register, Status};
use sprite::{Sprite, SpriteAttribute, OAM_SIZE, SPRITE_COUNT, SPRITE_LIMIT};
use vram_address::VRAMAddress;

const MAX_DOT: u16 = 340;
const MAX_LINE: u16 = 261;

const WIDTH: u16 = 256;

pub struct PPU {
    reg: Register,
    bus: Box<dyn Memory>,

    // Background registers
    name_table_entry: Byte,
    attr_table_entry: Byte,
    bg_temp_addr: VRAMAddress,

    // Background tiles
    tile: background::Tile,
    next_pattern: background::TilePattern,

    // Sprite OAM
    primary_oam: [u8; OAM_SIZE],
    secondary_oam: [u8; 32],
    sprites: [Sprite; SPRITE_LIMIT],
    sprite_zero_on_line: bool,

    // http://wiki.nesdev.com/w/index.php/PPU_registers#Ports
    internal_data_bus: u8,

    pub frames: u64,
    scan: Scan,
}

impl PPU {
    pub fn new(ppu_bus: Box<dyn Memory>) -> Self {
        Self {
            reg: Default::default(),
            bus: ppu_bus,
            name_table_entry: Default::default(),
            attr_table_entry: Default::default(),
            bg_temp_addr: Default::default(),
            tile: Default::default(),
            next_pattern: Default::default(),

            primary_oam: [0; OAM_SIZE],
            secondary_oam: [0; 32],
            sprites: [Default::default(); SPRITE_LIMIT],
            sprite_zero_on_line: false,
            internal_data_bus: 0,
            frames: 0,
            scan: Default::default(),
        }
    }

    pub fn reset(&mut self) {
        self.reg.reset();
        self.scan.clear();
        self.frames = 0;
    }

    pub fn current_line(&self) -> u16 {
        self.scan.line
    }

    pub fn step(&mut self) -> Option<Interrupt> {
        let mut interrupt = None;

        match (self.scan.line, self.scan.line == 261) {
            (0..=239, pre_rendered) => {
                // Visible or Pre Render
                let x = self.scan.dot.wrapping_sub(2);

                let bg = self.get_background_pixel(x);
                let sprite = self.get_sprite_pixel(x as i32, bg);

                if self.reg.rendering_enabled() {
                    self.fetch_background_pixel();
                    self.fetch_sprite_pixel();
                }

                if self.scan.line < MAX_LINE && x < WIDTH {
                    let _pixel = if self.reg.rendering_enabled() {
                        self.select_pixel(bg, sprite)
                    } else {
                        0
                    };
                    // TODO Render pixel
                }

                if pre_rendered {
                    if self.scan.dot == 1 {
                        self.reg.status.unset(
                            Status::VBLANK | Status::SPRITE_ZERO_HIT | Status::SPRITE_OVERFLOW,
                        )
                    }
                    if self.scan.dot == 341 && self.reg.rendering_enabled() && self.frames % 2 != 0
                    {
                        // Skip 0 cycle on visible frame
                        self.scan.skip();
                    }
                }
            }
            (240, _) => {
                // Post Render
            }
            (241, _) => {
                // Begin VBLANK
                if self.scan.dot == 1 {
                    self.reg.status.set(Status::VBLANK);
                    if self.reg.controller.is_set(Controller::NMI) {
                        interrupt = Some(Interrupt::NMI);
                    }
                }
            }
            _ => {}
        }

        if let ScanUpdate::Frame = self.scan.next_dot() {
            self.frames += 1;
        }

        interrupt
    }

    fn select_pixel(&self, bg: background::Pixel, sprite: sprite::Pixel) -> u16 {
        match (bg.enabled, sprite.enabled) {
            (false, false) => self.bus.read(0x3F00u16.into()).into(),
            (false, true) => sprite.color,
            (true, false) => bg.color,
            (true, true) => {
                if sprite.behide_background {
                    bg.color
                } else {
                    sprite.color
                }
            }
        }
    }
}

// background
impl PPU {
    fn fetch_background_pixel(&mut self) {
        match self.scan.dot {
            321 => {
                // No reload shift
                self.bg_temp_addr =
                    (NAME_TABLE_FIRST | self.reg.v.name_table_address_index().into()).into();
            }
            1..=255 | 322..=336 => match self.scan.dot % 8 {
                1 => {
                    // Fetch nametable byte : step 1
                    self.bg_temp_addr =
                        (NAME_TABLE_FIRST | self.reg.v.name_table_address_index().into()).into();
                    self.tile.reload(self.next_pattern, self.attr_table_entry);
                }
                2 => {
                    // Fetch nametable byte : step 2
                    self.name_table_entry = self.bus.read(self.bg_temp_addr.into());
                }
                3 => {
                    // Fetch attribute table byte : step 1
                    self.bg_temp_addr = (ATTRIBUTE_TABLE_FIRST
                        | self.reg.v.attribute_address_index().into())
                    .into();
                }
                4 => {
                    // Fetch attribute table byte : step 2
                    self.attr_table_entry = self.bus.read(self.bg_temp_addr.into());
                    // select area
                    if self.reg.v.coarse_x_scroll().nth(1) == 1 {
                        self.attr_table_entry >>= 2
                    }
                    if self.reg.v.coarse_y_scroll().nth(1) == 1 {
                        self.attr_table_entry >>= 4
                    }
                }
                5 => {
                    // Fetch tile bitmap low byte : step 1
                    let base: Word = if self.reg.controller.is_set(Controller::BG_TABLE_ADDR) {
                        0x1000u16
                    } else {
                        0x0000u16
                    }
                    .into();
                    let index = self.name_table_entry * TILE_HEIGHT * 2;
                    self.bg_temp_addr = (base + index + self.reg.v.fine_y_scroll()).into();
                }
                6 => {
                    // Fetch tile bitmap low byte : step 2
                    self.next_pattern.low = self.bus.read(self.bg_temp_addr.into()).into();
                }
                7 => {
                    // Fetch tile bitmap high byte : step 1
                    self.bg_temp_addr += TILE_HEIGHT.into();
                }
                0 => {
                    // Fetch tile bitmap high byte : step 2
                    self.next_pattern.high = self.bus.read(self.bg_temp_addr.into()).into();
                    if self.reg.rendering_enabled() {
                        self.reg.incr_coarse_x();
                    }
                }
                _ => {}
            },
            256 => {
                self.next_pattern.high = self.bus.read(self.bg_temp_addr.into()).into();
                if self.reg.rendering_enabled() {
                    self.reg.incr_y();
                }
            }
            257 => {
                self.tile.reload(self.next_pattern, self.attr_table_entry);
                if self.reg.rendering_enabled() {
                    self.reg.copy_x();
                }
            }
            280..=304 => {
                if self.scan.line == 261 && self.reg.rendering_enabled() {
                    self.reg.copy_y();
                }
            }
            // Unused name table fetches
            337 | 339 => {
                self.bg_temp_addr =
                    (NAME_TABLE_FIRST | self.reg.v.name_table_address_index().into()).into();
            }
            338 | 340 => {
                self.name_table_entry = self.bus.read(self.bg_temp_addr.into());
            }
            _ => {}
        }
    }

    fn get_background_pixel(&mut self, x: u16) -> background::Pixel {
        let (pixel, pallete) = self.tile.pixel_pallete(self.reg.fine_x.into());

        if (1 <= self.scan.dot && self.scan.dot <= 256)
            || (321 <= self.scan.dot && self.scan.dot <= 336)
        {
            self.tile.shift();
        }

        if self.reg.is_enabled_background(x) {
            background::Pixel {
                enabled: <Word as Into<u16>>::into(pixel) != 0,
                color: self.bus.read(pallete * 4 + pixel + 0x3F00).into(),
            }
        } else {
            background::Pixel::ZERO
        }
    }
}

// sprite
impl PPU {
    fn fetch_sprite_pixel(&mut self) {
        match self.scan.dot {
            //TODO more cycle accumelated
            0 => {
                for e in self.secondary_oam.iter_mut() {
                    *e = 0;
                }
                self.sprite_zero_on_line = false;
                // the sprite evaluation phase
                let sprite_size = if self.reg.controller.is_set(Controller::SPRITE_SIZE) {
                    16
                } else {
                    8
                };

                let mut iter = self.secondary_oam.iter_mut();

                let mut n = 0;
                for i in 0..SPRITE_COUNT {
                    let first = i * 4;
                    let y = self.primary_oam[first];

                    if let Some(p) = iter.next() {
                        let row = self.scan.line.wrapping_sub(self.primary_oam[first] as u16);
                        if row < sprite_size {
                            if n == 0 {
                                self.sprite_zero_on_line = true;
                            }
                            *p = y;
                            *iter.next().unwrap() = self.primary_oam[first + 1];
                            *iter.next().unwrap() = self.primary_oam[first + 2];
                            *iter.next().unwrap() = self.primary_oam[first + 3];
                            n += 1;
                        }
                    }
                }
                if SPRITE_LIMIT <= n && self.reg.rendering_enabled() {
                    self.reg.status.set(Status::SPRITE_OVERFLOW);
                }
            }
            257..=320 => {
                // the sprite fetch phase
                let i = (self.scan.dot.wrapping_sub(257)) / 8;
                let n = i.wrapping_mul(4) as usize;
                self.sprites[i as usize] = Sprite {
                    y: self.secondary_oam[n],
                    tile_index: self.secondary_oam[n + 1],
                    attr: self.secondary_oam[n + 1].into(),
                    x: self.secondary_oam[n + 1],
                };
            }
            _ => {}
        }
    }

    fn get_sprite_pixel(&mut self, x: i32, bg: background::Pixel) -> sprite::Pixel {
        if !self.reg.is_enabled_sprite(x) {
            return sprite::Pixel::ZERO;
        }

        let y = self.scan.line;
        for (i, sprite) in self.sprites.iter().enumerate() {
            if !sprite.valid() {
                break;
            }
            if (sprite.x as i32) < x - 7 && x < sprite.x as i32 {
                continue;
            }
            let mut row = sprite.row(y, self.reg.sprite_size());
            let col = sprite.col(x as u16);
            let mut tile_idx = sprite.tile_index as u16;

            let base = if self.reg.controller.sprite_8x16_pixels() {
                tile_idx &= 0xFE;
                if 7 < row {
                    tile_idx += 1;
                    row -= 8;
                }
                tile_idx & 1
            } else {
                self.reg.controller.base_sprite_table_addr()
            };

            let tile_addr = base + tile_idx * 16 + row;
            let low = self.bus.read(tile_addr.into());
            let high = self.bus.read((tile_addr + 8).into());

            let pixel = low.nth(col) + (high.nth(col) << 1);
            if pixel == 0 {
                // transparent
                continue;
            }

            if i == 0
                && self.sprite_zero_on_line
                && !self.reg.status.is_set(Status::SPRITE_ZERO_HIT)
                && sprite.x != 0xFF
                && x < 0xFF
                && bg.enabled
            {
                self.reg.status.set(Status::SPRITE_ZERO_HIT);
            }

            let addr = 0x3F10 + sprite.attr.pallete() as u16 * 4 + pixel as u16;
            return sprite::Pixel {
                enabled: pixel != 0,
                color: self.bus.read(addr.into()).into(),
                behide_background: sprite.attr.is_set(SpriteAttribute::BEHIND_BACKGROUND),
            };
        }

        sprite::Pixel::ZERO
    }
}

// register access from CPU
impl PPU {
    pub fn read_register(&mut self, addr: u16) -> Byte {
        let result = match addr {
            0x2002 => {
                let result = self.reg.read_status() | (self.internal_data_bus & 0b11111);
                if self.scan.line == 241 && self.scan.dot < 2 {
                    result & !0x80
                } else {
                    result
                }
            }
            0x2004 => {
                // https://wiki.nesdev.com/w/index.php/PPU_sprite_evaluation
                if self.scan.line < 240 && 1 <= self.scan.dot && self.scan.dot <= 64 {
                    // during sprite evaluation
                    0xFF
                } else {
                    self.primary_oam[self.reg.object_attribute_memory_address]
                }
                .into()
            }
            0x2007 => {
                let v: u16 = self.reg.v.into();
                let result = if v <= 0x3EFFu16 {
                    let data = self.reg.data;
                    self.reg.data = self.bus.read(self.reg.v.into());
                    data
                } else {
                    self.bus.read(self.reg.v.into())
                };
                self.reg.incr_v();
                result
            }
            _ => 0x00.into(),
        };

        self.internal_data_bus = result.into();
        result
    }

    pub fn write_register(&mut self, addr: u16, value: Byte) {
        match addr {
            0x2000 => self.reg.write_controller(value),
            0x2001 => self.reg.mask = Mask::new(value),
            0x2003 => {
                let addr: u16 = value.into();
                self.reg.object_attribute_memory_address = addr.into();
            }
            0x2004 => {
                self.primary_oam[self.reg.object_attribute_memory_address] = value.into();
                self.reg.object_attribute_memory_address =
                    self.reg.object_attribute_memory_address.wrapping_add(1);
            }
            0x2005 => self.reg.write_scroll(value),
            0x2006 => self.reg.write_vram_address(value),
            0x2007 => {
                self.bus.write(self.reg.v.into(), value);
                self.reg.incr_v();
            }
            _ => {}
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
struct Scan {
    dot: u16,
    line: u16,
}

impl Scan {
    fn clear(&mut self) {
        self.dot = 0;
        self.line = 0;
    }

    fn skip(&mut self) {
        self.dot += 1;
    }

    fn next_dot(&mut self) -> ScanUpdate {
        self.dot = self.dot.wrapping_add(1);
        if MAX_DOT <= self.dot {
            self.dot %= MAX_DOT;

            self.line += 1;
            if MAX_LINE < self.line {
                self.line = 0;
                ScanUpdate::Frame
            } else {
                ScanUpdate::Line
            }
        } else {
            ScanUpdate::Dot
        }
    }
}

enum ScanUpdate {
    Dot,
    Line,
    Frame,
}
