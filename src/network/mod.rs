pub mod client;
pub mod server;

use crate::schema::{Color, TextMessage};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::{net::SocketAddr, string::ToString};
use tokio_tungstenite::tungstenite::Message as ttMessage;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub msg_type: MessageType,
    pub passwd: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    User(UserMsg),
    UserReq(UserReqMsg),
    Server(ServerMsg),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserMsg {
    Normal { msg: TextMessage },
    UserJoined { user: User },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UserReqMsg {
    FetchMessagesReq,
    BanReq { addr: SocketAddr },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ServerMsg {
    AuthFailure,
    MessagesFetch { messages: Vec<TextMessage> },
    UserLeft { addr: SocketAddr },
    BanConfirm { addr: SocketAddr },
}

impl From<ttMessage> for Message {
    fn from(value: ttMessage) -> Self {
        from_str(&value.to_string()).unwrap()
    }
}

impl Message {
    pub fn to_ttmessage(&self) -> ttMessage {
        ttMessage::text(to_string(self).unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub _id: String,
    pub addr: Option<SocketAddr>,
    pub color: Color,
}

#[cfg(test)]
mod test {}
