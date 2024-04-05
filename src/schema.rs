use serde::{Deserialize, Serialize};
use strum::EnumString;

pub struct Room {
    pub room_id: String,
    pub room_address: String,
    pub password: Option<String>,
    pub guests: Vec<User>,
    pub locked_addressed: Vec<String>,
    pub are_you_host: bool,
}

#[derive(Serialize, Deserialize)]
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
            room_id: self.room_id,
            room_address: self.room_address,
            password: self.password,
            locked_addresses: self.locked_addressed,
            are_you_host: self.are_you_host,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub address: String,
    pub username: String,
    pub color: Color,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub msg_id: u32,
    pub sender_address: String,
    pub chatroom_id: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct Color(pub i32, pub i32, pub i32);

#[derive(Serialize, Deserialize, EnumString)]
pub enum AppOption {
    RememberPasswords(bool),
    LightMode(bool),
}
