pub mod client;
pub mod message;
pub mod server;

use crate::schema::Color;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub _id: String,
    pub addr: Option<SocketAddr>,
    pub color: Color,
}

#[cfg(test)]
mod test {}
