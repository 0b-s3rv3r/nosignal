use serde::{Deserialize, Serialize};

pub struct Room {
    pub room_id: String,
    pub room_address: String,
    pub password: Option<String>,
    pub host: User,
    pub guests: Vec<User>,
}

#[derive(Serialize, Deserialize)]
pub struct RoomData {
    pub room_id: String,
    pub room_address: String,
    pub password: Option<String>,
    pub host_id: u32,
    pub guests_ids: Vec<u32>,
}

impl Room {
    pub fn prepare_data(&self) -> RoomData {
        RoomData {
            room_id: self.room_id,
            room_address: self.room_address,
            password: self.password,
            host_id: self.host.id,
            guests_ids: self.guests.iter().map(|guest| guest.id).collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub address: String,
    pub color: Color,
}

#[derive(Serialize, Deserialize)]
pub struct Messsage {
    pub msg_id: u32,
    pub sender_id: u32,
    pub chatroom_id: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct Color(i32, i32, i32);
