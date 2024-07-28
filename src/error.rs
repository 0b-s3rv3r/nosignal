use polodb_core::Error as pdbError;
use std::io::Error as ioError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    PdbError(pdbError),
    #[error("{0}")]
    IoError(ioError),
    #[error("This id already exists.")]
    AlreadyExistingId,
    #[error("Data not found in database.")]
    DataNotFound,
    #[error("Invalid password.")]
    InvalidPassword,
    #[error("No such room")]
    NotExistingId,
    #[error("There is no any room yet")]
    NoAnyRoom,
    #[error("Invalid command.")]
    InvalidCommand,
}
