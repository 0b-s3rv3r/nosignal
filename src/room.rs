pub struct ChatRoom {
    room_id: String,
    address: String,
    password: Option<String>,
    host: User,
    guests: Vec<User>,
}

impl ChatRoom {
    pub fn add_message(content: String, sender: User) {}
    pub fn add_guest(guest: User) {}
    pub fn remove_guest(guest: User) {}
}

pub struct User {
    username: String,
    color: String,
    address: String
}

pub struct Message {
    id: u32,
    sender: User,
    content: String
}