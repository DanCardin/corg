use thiserror::Error;

#[derive(Error, Debug)]
pub enum CorgError {
    #[error("No code blocks detected")]
    NoBlocksDetected,

    #[error("{0}")]
    IOError(#[from] std::io::Error),

    #[error("Error occured during block execution: {0}")]
    BlockExecutionError(String),

    #[error("Generated dutput did not match the existing content")]
    CheckFailed((String, String)),
}
