use terrain::TerrainError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PropahError {
    #[error("missing required parameter '{0}'")]
    Builder(&'static str),

    #[error("{0}")]
    Terrain(#[from] TerrainError),
}
