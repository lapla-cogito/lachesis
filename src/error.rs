#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Scheduler already initialized")]
    AlreadyInitialized,

    #[error("Scheduler not initialized")]
    NotInitialized,

    #[error("Thread not found: {0}")]
    ThreadNotFound(u64),

    #[error("Deadlock detected")]
    Deadlock,

    #[error("Lock acquisition failed")]
    LockFailed,

    #[error("Invalid stack size: {size}. Minimum size is {min} bytes")]
    InvalidStackSize { size: usize, min: usize },

    #[error("Thread spawn failed")]
    SpawnFailed,

    #[error("System resource error: {0}")]
    SystemResource(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

impl Error {
    pub fn is_recoverable(&self) -> bool {
        match self {
            Error::AlreadyInitialized
            | Error::NotInitialized
            | Error::InvalidStackSize { .. }
            | Error::Configuration(_) => false,
            Error::ThreadNotFound(_)
            | Error::Deadlock
            | Error::LockFailed
            | Error::SpawnFailed
            | Error::SystemResource(_) => true,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
