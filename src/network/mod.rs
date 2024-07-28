pub mod client;
pub mod server;

use crate::schema::{Color, Message};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::{net::SocketAddr, string::ToString};
use tokio_tungstenite::tungstenite::Message as ttMessage;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChatMessage {
    Normal {
        msg: Message,
        passwd: Option<String>,
    },
    Ban {
        addr: SocketAddr,
        passwd: Option<String>,
    },
    UserJoined {
        user: User,
        passwd: Option<String>,
    },
    UserLeft {
        user_id: String,
    },
    FetchMessagesReq {
        passwd: Option<String>,
    },
    FetchMessages {
        messages: Vec<Message>,
    },
    ServerShutdown,
    AuthFailure_,
}

impl From<ttMessage> for ChatMessage {
    fn from(value: ttMessage) -> Self {
        from_str(&value.to_string()).unwrap()
    }
}

impl ToString for ChatMessage {
    fn to_string(&self) -> String {
        to_string(self).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub addr: Option<SocketAddr>,
    pub color: Color,
}

#[cfg(test)]
mod test {}
