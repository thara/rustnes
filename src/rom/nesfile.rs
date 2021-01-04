use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, SeekFrom};
use std::path::Path;

use anyhow::{Context, Result};
use thiserror::Error;

use crate::types::Mirroring;

pub struct NESFile {
    header: NESFileHeader,
    row_data: Vec<u8>,
}

impl NESFile {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<NESFile> {
        let f = File::open(path.as_ref()).with_context(|| {
            format!(
                "Failed to open ROM file: {}",
                path.as_ref().to_str().unwrap_or("unknown")
            )
        })?;
        let mut b = BufReader::new(f);
        let mut header_bytes = [0; NESFileHeader::SIZE];

        b.read_exact(&mut header_bytes)?;
        let header = NESFileHeader::parse(&header_bytes);
        if !header.valid() {
            return Err(From::from(NESFileError::InvalidHeader));
        }

        b.seek(SeekFrom::Start(0))?;

        let mut row_data = Vec::new();
        b.read_to_end(&mut row_data)?;

        Ok(Self { header, row_data })
    }

    fn read_bytes(&self, first: usize, count: usize) -> (Vec<u8>, usize) {
        let last = first + count;
        (self.row_data[first..last].to_vec(), last)
    }

    pub(super) fn read_prg_rom(&self, first: usize, rom_size: usize) -> (Vec<u8>, usize) {
        self.read_bytes(first, self.header.prg_size_of_unit * rom_size)
    }

    pub(super) fn read_chr_rom(&self, first: usize, rom_size: usize) -> Option<(Vec<u8>, usize)> {
        if self.header.chr_size_of_unit == 0 {
            None // Use CHA RAM
        } else {
            Some(self.read_bytes(first, self.header.chr_size_of_unit * rom_size))
        }
    }

    pub(super) fn mirroring(&self) -> Mirroring {
        if self.header.flags6 & 1 == 0 {
            Mirroring::Horizontal()
        } else {
            Mirroring::Vertical()
        }
    }

    pub(super) fn mapper_no(&self) -> u8 {
        (self.header.flags7 & 0b11110000) + (self.header.flags6 >> 4)
    }
}

pub struct NESFileHeader {
    magic: [u8; 4],
    prg_size_of_unit: usize,
    chr_size_of_unit: usize,
    flags6: u8,
    flags7: u8,
    _flags8: u8,
    _flags9: u8,
    _flags10: u8,
    padding: [u8; 5],
}

impl NESFileHeader {
    const MAGIC_NUMBER: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
    const PADDING: [u8; 5] = [0; 5];
    pub const SIZE: usize = 16;

    fn parse(bytes: &[u8; Self::SIZE]) -> Self {
        NESFileHeader {
            magic: bytes[0..4].try_into().unwrap(),
            prg_size_of_unit: bytes[4] as usize,
            chr_size_of_unit: bytes[5] as usize,
            flags6: bytes[6],
            flags7: bytes[7],
            _flags8: bytes[8],
            _flags9: bytes[9],
            _flags10: bytes[10],
            padding: bytes[11..].try_into().unwrap(),
        }
    }

    fn valid(&self) -> bool {
        self.magic == Self::MAGIC_NUMBER && self.padding == Self::PADDING
    }
}

#[derive(Debug, Error)]
enum NESFileError {
    #[error("The ROM file has invalid header")]
    InvalidHeader,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_header() {
        let data = [
            0x4E, 0x45, 0x53, 0x1A, 0x93, 0x34, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];

        let header = NESFileHeader::parse(&data);

        assert!(header.valid());
        assert_eq!(header.prg_size_of_unit, 0x93);
        assert_eq!(header.chr_size_of_unit, 0x34);
        assert_eq!(header.flags6, 0xF1);
        assert_eq!(header.flags7, 0xF2);
        assert_eq!(header._flags8, 0xF3);
        assert_eq!(header._flags9, 0xF4);
        assert_eq!(header._flags10, 0xF5);
    }

    #[test]
    fn invalid_header() {
        let data = [
            0x4E, 0x45, 0x53, 0x1B, 0x93, 0x34, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];

        let header = NESFileHeader::parse(&data);

        assert!(!header.valid());
    }

    #[test]
    fn load_sample_rom() {
        use std::path::Path;

        let path = Path::new(file!()).parent().unwrap().join("sample.nes");
        let result = NESFile::open(path);
        assert!(result.is_ok());

        let nesfile = result.unwrap();
        assert!(nesfile.header.valid());
    }
}
