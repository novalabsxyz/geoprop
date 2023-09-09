use thiserror::Error;

#[derive(Error, Debug)]
pub enum HError {
    #[error("")]
    Io(#[from] std::io::Error),
}
