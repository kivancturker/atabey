use std::io;
use std::path::PathBuf;

/// Why a ledger write failed. Writing never panics.
#[derive(Debug, thiserror::Error)]
pub enum DefterError {
    /// Creating a folder or writing the ledger file failed.
    #[error("ledger I/O failed for {}: {source}", path.display())]
    Io {
        /// The folder or file involved.
        path: PathBuf,
        /// The underlying error.
        source: io::Error,
    },
    /// The entry could not be encoded as JSON.
    #[error("ledger entry could not be serialized: {0}")]
    Serialize(#[from] serde_json::Error),
}

/// Why the app log could not be set up.
#[derive(Debug, thiserror::Error)]
pub enum LogError {
    /// The log folder or the rolling file could not be created.
    #[error("app log setup failed for {}: {reason}", path.display())]
    Setup {
        /// The log folder.
        path: PathBuf,
        /// What went wrong.
        reason: String,
    },
    /// A global `tracing` subscriber is already installed.
    #[error("a global tracing subscriber is already installed")]
    AlreadyInstalled,
}
