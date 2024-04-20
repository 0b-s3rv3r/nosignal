pub enum DbError {
    AlreadyExistingId,
}

pub enum NetError {
    PubAddrFetchFailure,
    ListenerBindingFailure,
}

pub enum CommandError {
    InvalidIpv4,
}
