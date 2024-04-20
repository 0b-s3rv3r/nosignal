use std::{net::Ipv4Addr, str::FromStr};

use futures_util::TryFutureExt;
use reqwest::dns::Resolve;
use tokio::net::TcpListener;

use crate::error::NetError;
use crate::schema::{Message, Room, User};

struct ChatServer {
    listener: TcpListener,
    current_room: Room,
}

impl ChatServer {
    pub async fn new(room: Room) -> Result<Self, NetError> {
        Ok(Self {
            listener: TcpListener::bind(&room.socket_addr)
                .await
                .map_err(|_| NetError::ListenerBindingFailure)?,
            current_room: room,
        })
    }

    pub fn run() {}

    fn handle_client() {}
}

struct ChatClient {
    listener: TcpListener,
    current_room: Room,
}

async fn get_public_addr() -> Result<Ipv4Addr, NetError> {
    let response = reqwest::get("https://api.ipify.org")
        .await
        .map_err(|_| NetError::PubAddrFetchFailure)?
        .text()
        .await
        .map_err(|_| NetError::PubAddrFetchFailure)?;

    Ok(Ipv4Addr::from_str(&response).map_err(|_| NetError::PubAddrFetchFailure)?)
}
