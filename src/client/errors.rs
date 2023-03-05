use thiserror::Error;

#[derive(Debug, Error)]
pub enum HandshakeError {
    #[error("IOError: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Invalid Server Protocol Version: {0}")]
    InvalidServerVersion(u8),
}
