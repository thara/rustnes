mod background;
mod register;
mod vram_address;

use crate::types::{Byte, Memory, Word};

use background::{ATTRIBUTE_TABLE_FIRST, NAME_TABLE_FIRST, TILE_HEIGHT};
use register::Register;
use vram_address::VRAMAddress;

const MAX_DOT: u16 = 340;
const MAX_LINE: u16 = 261;

pub struct PPU {
    register: Register,
    bus: Box<dyn Memory>,

    // Background registers
    name_table_entry: Byte,
    attr_table_entry: Byte,
    bg_temp_addr: VRAMAddress,

    // Background tiles
    tile: background::Tile,
    next_pattern: background::TilePattern,

    scan: Scan,
}

impl PPU {
    pub fn new(ppu_bus: Box<dyn Memory>) -> Self {
        Self {
            register: Default::default(),
            bus: ppu_bus,
            scan: Default::default(),
            name_table_entry: Default::default(),
            attr_table_entry: Default::default(),
            bg_temp_addr: Default::default(),
            tile: Default::default(),
            next_pattern: Default::default(),
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
                    (NAME_TABLE_FIRST | self.register.v.name_table_address_index().into()).into();
            }
            1..=255 | 322..=336 => match self.scan.dot % 8 {
                1 => {
                    // Fetch nametable byte : step 1
                    self.bg_temp_addr = (NAME_TABLE_FIRST
                        | self.register.v.name_table_address_index().into())
                    .into();
                    self.tile.reload(self.next_pattern, self.attr_table_entry);
                }
                2 => {
                    // Fetch nametable byte : step 2
                    self.name_table_entry = self.bus.read(self.bg_temp_addr.into());
                }
                3 => {
                    // Fetch attribute table byte : step 1
                    self.bg_temp_addr = (ATTRIBUTE_TABLE_FIRST
                        | self.register.v.attribute_address_index().into())
                    .into();
                }
                4 => {
                    // Fetch attribute table byte : step 2
                    self.attr_table_entry = self.bus.read(self.bg_temp_addr.into());
                    // select area
                    if self.register.v.coarse_x_scroll().nth(1) == 1 {
                        self.attr_table_entry >>= 2
                    }
                    if self.register.v.coarse_y_scroll().nth(1) == 1 {
                        self.attr_table_entry >>= 4
                    }
                }
                5 => {
                    // Fetch tile bitmap low byte : step 1
                    let base: Word = if self
                        .register
                        .controller
                        .is_set(register::Controller::BG_TABLE_ADDR)
                    {
                        0x1000u16
                    } else {
                        0x0000u16
                    }
                    .into();
                    let index = self.name_table_entry * TILE_HEIGHT * 2;
                    self.bg_temp_addr = (base + index + self.register.v.fine_y_scroll()).into();
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
                    if self.register.rendering_enabled() {
                        self.register.incr_coarse_x();
                    }
                }
                _ => {}
            },
            256 => {
                self.next_pattern.high = self.bus.read(self.bg_temp_addr.into()).into();
                if self.register.rendering_enabled() {
                    self.register.incr_y();
                }
            }
            257 => {
                self.tile.reload(self.next_pattern, self.attr_table_entry);
                if self.register.rendering_enabled() {
                    self.register.copy_x();
                }
            }
            280..=304 => {
                if self.scan.line == 261 && self.register.rendering_enabled() {
                    self.register.copy_y();
                }
            }
            // Unused name table fetches
            337 | 339 => {
                self.bg_temp_addr =
                    (NAME_TABLE_FIRST | self.register.v.name_table_address_index().into()).into();
            }
            338 | 340 => {
                self.name_table_entry = self.bus.read(self.bg_temp_addr.into());
            }
            _ => {}
        }
    }

    fn get_background_pixel(&mut self, x: u16) -> background::Pixel {
        let (pixel, pallete) = self.tile.pixel_pallete(self.register.fine_x.into());

        if (1 <= self.scan.dot && self.scan.dot <= 256)
            || (321 <= self.scan.dot && self.scan.dot <= 336)
        {
            self.tile.shift();
        }

        if self.register.is_enabled_background(x) {
            background::Pixel {
                enabled: <Word as Into<u16>>::into(pixel) != 0,
                color: self.bus.read(pallete * 4 + pixel + 0x3F00).into(),
            }
        } else {
            background::Pixel::ZERO
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

            let last = self.line;
            self.line += 1;
            if MAX_LINE < self.line {
                self.line = 0;
                ScanUpdate::Frame { last_line: last }
            } else {
                ScanUpdate::Line { last_line: last }
            }
        } else {
            ScanUpdate::Dot
        }
    }
}

enum ScanUpdate {
    Dot,
    Line { last_line: u16 },
    Frame { last_line: u16 },
}
