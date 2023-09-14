use nasadem::HError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TerrainError {
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Nasadem(#[from] HError),
}
