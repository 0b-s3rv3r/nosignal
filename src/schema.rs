use std::net::SocketAddr;
use tokio::net::TcpStream;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

use crate::network::{RoomServer, User};

impl RoomServer {
    pub async fn serialize(&self) -> RoomData {
        let self_ = &self.inner.lock().await.data;
        RoomData {
            room_id: self_.room_id.clone(),
            socket_addr: self_.socket_addr,
            password: self_.password.clone(),
            locked_addrs: self_.locked_addrs.clone(),
            is_owner: self_.is_owner,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct RoomData {
    pub room_id: String,
    pub socket_addr: SocketAddr,
    pub password: Option<String>,
    pub locked_addrs: Vec<SocketAddr>,
    pub is_owner: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    pub user_id: String,
    pub addr: SocketAddr,
    pub color: Color,
}

impl User {
    pub fn serialize(&self) -> UserData {
        UserData {
            addr: self.addr.peer_addr().unwrap(),
            user_id: self.user_id.clone(),
            color: self.color,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Message {
    pub msg_id: u32,
    pub sender_id: String,
    pub chatroom_id: String,
    pub content: String,
    pub timestamp: std::time::SystemTime,
}

#[derive(Serialize, Deserialize)]
pub struct Color(pub i32, pub i32, pub i32);

#[derive(Serialize, Deserialize, EnumString, EnumIter, Display, PartialEq, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AppOpt {
    #[strum(serialize = "remember_passwords")]
    RememberPasswords,
    #[strum(serialize = "light_mode")]
    LightMode,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct AppOption {
    pub option: AppOpt,
    pub enabled: bool,
}
