use super::User;
use crate::schema::TextMessage;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::net::SocketAddr;
use tokio_tungstenite::tungstenite::Message as TtMessage;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Message {
    pub msg_type: MessageType,
    pub passwd: Option<String>,
}

impl From<(MessageType, Option<String>)> for Message {
    fn from(value: (MessageType, Option<String>)) -> Self {
        let (msg_type, passwd) = value;
        Self { msg_type, passwd }
    }
}

impl From<(UserMsg, Option<String>)> for Message {
    fn from(value: (UserMsg, Option<String>)) -> Self {
        let (user_msg, passwd) = value;
        Self {
            msg_type: MessageType::User(user_msg),
            passwd,
        }
    }
}

impl From<ServerMsg> for Message {
    fn from(value: ServerMsg) -> Self {
        Self {
            msg_type: MessageType::Server(value),
            passwd: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum MessageType {
    User(UserMsg),
    Server(ServerMsg),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum UserMsg {
    Normal { msg: TextMessage },
    UserJoined { user: User },
    Auth,
    SyncReq,
    BanReq { addr: SocketAddr },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ServerMsg {
    AuthFailure,
    AuthReq {
        passwd_required: bool,
    },
    Sync {
        user_addr: SocketAddr,
        room_id: String,
        messages: Vec<TextMessage>,
        users: Vec<User>,
    },
    UserLeft {
        addr: SocketAddr,
    },
    BanConfirm {
        addr: SocketAddr,
    },
    ServerShutdown,
}

impl From<TtMessage> for Message {
    fn from(value: TtMessage) -> Self {
        from_str(&value.to_string()).unwrap()
    }
}

impl TryFrom<&TtMessage> for Message {
    type Error = ();

    fn try_from(value: &TtMessage) -> Result<Self, Self::Error> {
        if let Ok(val) = from_str(&value.to_string()) {
            val
        } else {
            Err(())
        }
    }
}

impl Message {
    pub fn to_ttmessage(&self) -> TtMessage {
        TtMessage::text(to_string(self).unwrap())
    }
}
