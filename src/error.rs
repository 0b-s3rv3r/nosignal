#[derive(Debug)]
pub enum DbError {
    AlreadyExistingId,
}

#[derive(Debug)]
pub enum CommandError {
    InvalidIpv4,
}

#[derive(Debug)]
pub enum AppError {
    EnvCreationFailure,
    DbFailure,
}
