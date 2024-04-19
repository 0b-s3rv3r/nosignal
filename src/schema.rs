use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

pub struct Room {
    pub room_id: String,
    pub room_addr: String,
    pub password: Option<String>,
    pub guests: Vec<Client>,
    pub locked_addressed: Vec<String>,
    pub are_you_host: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct RoomData {
    pub room_id: String,
    pub room_address: String,
    pub password: Option<String>,
    pub locked_addresses: Vec<String>,
    pub are_you_host: bool,
}

impl Room {
    pub fn prepare_data(&self) -> RoomData {
        RoomData {
            room_id: self.room_id.clone(),
            room_address: self.room_address.clone(),
            password: self.password.clone(),
            locked_addresses: self.locked_addressed.clone(),
            are_you_host: self.are_you_host.clone(),
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
