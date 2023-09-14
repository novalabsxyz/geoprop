use thiserror::Error;

#[derive(Error, Debug)]
pub enum NasademError {
    #[error("")]
    Io(#[from] std::io::Error),

    #[error("invalid HGT name, {0}")]
    HgtName(std::path::PathBuf),

    #[error("invalid HGT file len, {0}")]
    HgtLen(u64),
}
