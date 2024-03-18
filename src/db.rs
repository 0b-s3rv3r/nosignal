use rusqlite::*;
use crate::room::{ChatRoom, Message};

const FILEPATH: &str = " ";

pub struct ChatDb {}

impl ChatDb {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_chatroom(room_id: String) {}
    pub fn get_all_chatrooms() {}
    pub fn add_chatroom(room: ChatRoom) {}
    pub fn update_chatroom(room: ChatRoom) {}
    pub fn delete_chatroom(room: ChatRoom) {}
}
