pub enum DbError {
    AlreadyExistingId,
}

pub enum NetError {
    PubAddrFetchFailure,
}

pub enum CommandError {
    InvalidIpv4,
}
