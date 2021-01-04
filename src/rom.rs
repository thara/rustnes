mod nesfile;

mod mapper_0;

use crate::types::{Memory, Mirroring};

use std::path::Path;

use anyhow::Result;
use thiserror::Error;

pub trait Mapper: Memory {
    fn mirroring(&self) -> Mirroring;
}

pub struct ROM {
    pub mapper: Box<dyn Mapper>,
}

impl ROM {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let f = nesfile::NESFile::open(path)?;
        let mapper_no = f.mapper_no();
        let mapper = if mapper_no == 0 {
            Ok(mapper_0::Mapper0::new(f))
        } else {
            Err(MapperError::UnsupportedMapper(f.mapper_no()))
        }?;
        Ok(Self {
            mapper: Box::new(mapper),
        })
    }
}

#[derive(Debug, Error)]
enum MapperError {
    #[error("Mapper no {0} does not supported")]
    UnsupportedMapper(u8),
}
