use std::io::Error as IoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] IoError),

    #[error("data parse error: {0}")]
    BinRwError(#[from] binrw::Error),

    #[error(transparent)]
    LibBmfwError(#[from] libbmfw::Error),
}
