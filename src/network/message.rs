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

impl Message {
    pub fn new(msg_type: MessageType, passwd: Option<String>) -> Self {
        Self { msg_type, passwd }
    }
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

impl From<(UserReqMsg, Option<String>)> for Message {
    fn from(value: (UserReqMsg, Option<String>)) -> Self {
        let (user_req, passwd) = value;
        Self {
            msg_type: MessageType::UserReq(user_req),
            passwd,
        }
    }
}

impl From<(ServerMsg, Option<String>)> for Message {
    fn from(value: (ServerMsg, Option<String>)) -> Self {
        let (server_msg, passwd) = value;
        Self {
            msg_type: MessageType::Server(server_msg),
            passwd,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum MessageType {
    User(UserMsg),
    UserReq(UserReqMsg),
    Server(ServerMsg),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum UserMsg {
    Normal { msg: TextMessage },
    UserJoined { user: User },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum UserReqMsg {
    SyncReq,
    BanReq { addr: SocketAddr },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ServerMsg {
    AuthFailure,
    Sync {
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

impl Message {
    pub fn to_ttmessage(&self) -> TtMessage {
        TtMessage::text(to_string(self).unwrap())
    }
}
