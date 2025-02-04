use std::array::TryFromSliceError;
use std::io;
use std::num::ParseIntError;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("ERR - Io: {0}")]
    Io(#[from] io::Error),

    #[error("ERR - Invalid arguments: {0}")]
    InvalidArgs(String),

    #[error("ERR - Parse: {0}")]
    Parse(#[from] ParseIntError),

    #[error("ERR - TryFromSlice: {0}")]
    TryFromSlice(#[from] TryFromSliceError),

    #[error("ERR - Http: {0}")]
    Http(#[from] reqwest::Error),

    #[error("ERR - Other: {0}")]
    Other(#[from] anyhow::Error),
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Other(anyhow::anyhow!("{value}"))
    }
}
