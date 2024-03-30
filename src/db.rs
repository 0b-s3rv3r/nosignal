use std::path::Path;

use polodb_core::{Collection, Database};

use crate::schema::{Messsage, RoomData};

pub struct DbRepo {
    rooms: Collection<RoomData>,
    messages: Collection<Messsage>,
}

impl DbRepo {
    pub fn init(filepath: &Path) -> Self {
        let db = Database::open_file(filepath).unwrap();

        DbRepo {
            rooms: db.collection("rooms"),
            messages: db.collection("messages"),
        }
    }

    pub(crate) fn memory_init() -> Self {
        let db = Database::open_memory().unwrap();

        DbRepo {
            rooms: db.collection("rooms"),
            messages: db.collection("messages"),
        }
    }

    pub fn rooms<'a>(&self) -> &'a Collection<RoomData> {
        &self.rooms
    }

    pub fn messages<'a>(&self) -> &'a Collection<Messsage> {
        &self.messages
    }
}
