use nasadem::NasademError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TerrainError {
    #[error("missing required parameters")]
    Builder,

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("no height files in {0}")]
    Path(PathBuf),

    #[error("{0}")]
    Nasadem(#[from] NasademError),
}
