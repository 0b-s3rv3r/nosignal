use mongodb::{options::ClientOptions, Client, Collection};

use crate::room::{ChatRoom, Message};

const FILEPATH: &str = " ";

pub struct ChatDb {
    db_collection: Collection<Message>,
}

impl ChatDb {
    pub fn new() -> Self {
        let mut client_options = ClientOptions::parse("mongodb://localhost:27017").unwrap();

        client_options.app_name = Some("gigachat".to_string());
        client_options.db_path = Some(FILEPATH.to_string());

        let client = Client::with_options(client_options).unwrap();

        let db = client.database("gigachatdb");

        ChatDb {
            db_collection: db.collection("messages"),
        }
    }

    pub fn get_chatroom(room_id: String) {}
    pub fn get_all_chatrooms() {}
    pub fn add_chatroom(room: ChatRoom) {}
    pub fn update_chatroom(room: ChatRoom) {}
    pub fn delete_chatroom(room: ChatRoom) {}
}
