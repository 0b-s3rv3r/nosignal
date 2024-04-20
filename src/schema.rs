use std::net::{Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

pub struct Room {
    pub room_id: String,
    pub socket_addr: SocketAddr,
    pub password: Option<String>,
    pub guests: Vec<User>,
    pub locked_addressed: Vec<String>,
    pub is_owner: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct RoomData {
    pub room_id: String,
    pub socket_addr: SocketAddr,
    pub password: Option<String>,
    pub locked_addresses: Vec<String>,
    pub is_owner: bool,
}

impl Room {
    pub fn prepare_data(&self) -> RoomData {
        RoomData {
            room_id: self.room_id.clone(),
            socket_addr: self.socket_addr.clone(),
            password: self.password.clone(),
            locked_addresses: self.locked_addressed.clone(),
            is_owner: self.is_owner.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub addr: Ipv4Addr,
    pub username: String,
    pub color: Color,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub msg_id: u32,
    pub sender_address: String,
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
