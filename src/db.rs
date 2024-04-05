use std::path::Path;

use polodb_core::{Collection, Database};

use crate::schema::{AppOpt, Message, RoomData, User};

pub struct DbRepo {
    pub rooms: Collection<RoomData>,
    pub messages: Collection<Message>,
    pub options: Collection<AppOpt>,
    pub user_local_data: Collection<User>,
}

impl DbRepo {
    pub fn init(filepath: &Path) -> Self {
        let db = Database::open_file(filepath).unwrap();

        DbRepo {
            rooms: db.collection::<RoomData>("rooms"),
            messages: db.collection::<Message>("messages"),
            options: db.collection::<AppOpt>("options"),
            user_local_data: db.collection::<User>("local_data"),
        }
    }

    pub(crate) fn memory_init() -> Self {
        let db = Database::open_memory().unwrap();

        DbRepo {
            rooms: db.collection::<RoomData>("rooms"),
            messages: db.collection::<Message>("messages"),
            options: db.collection::<AppOpt>("options"),
            user_local_data: db.collection::<User>("local_data"),
        }
    }
}
