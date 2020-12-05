use crate::migration::errors::*;
use crate::migration::runner::RunnerError;

#[derive(Debug)]
pub enum IOError {
    IO(std::io::Error),
    Blocking(tokio::task::JoinError),
}

#[derive(Debug)]
pub enum Error {
    DoesNotExist,
    ReferencedModelIsNotInDB,
    InternalError(quaint::error::Error),
    IO(IOError),
    RunnerError(RunnerError),
    MigrationError(MigrationError),
    Other(String),
}

pub type DBResult<T> = std::result::Result<T, Error>;

impl std::convert::From<MigrationError> for Error {
    fn from(err: MigrationError) -> Self {
        Error::MigrationError(err)
    }
}

impl std::convert::From<RunnerError> for Error {
    fn from(err: RunnerError) -> Self {
        Error::RunnerError(err)
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO(IOError::IO(err))
    }
}

impl std::convert::From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Error::IO(IOError::Blocking(err))
    }
}

impl std::convert::From<quaint::error::Error> for Error {
    fn from(err: quaint::error::Error) -> Self {
        Error::InternalError(err)
    }
}
