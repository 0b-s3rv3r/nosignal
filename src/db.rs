use std::path::Path;

use polodb_core::{Collection, Database};

use crate::schema::{AppOption, Message, RoomData, UserData};

pub struct DbRepo {
    pub rooms: Collection<RoomData>,
    pub messages: Collection<Message>,
    pub options: Collection<AppOption>,
    pub user_local_data: Collection<UserData>,
    _db: Database,
}

impl DbRepo {
    pub fn init(filepath: &Path) -> Self {
        let db = Database::open_file(filepath).unwrap();

        DbRepo {
            rooms: db.collection::<RoomData>("rooms"),
            messages: db.collection::<Message>("messages"),
            options: db.collection::<AppOption>("options"),
            user_local_data: db.collection::<UserData>("local_data"),
            _db: db,
        }
    }

    pub(crate) fn memory_init() -> Self {
        let db = Database::open_memory().unwrap();

        DbRepo {
            rooms: db.collection::<RoomData>("rooms"),
            messages: db.collection::<Message>("messages"),
            options: db.collection::<AppOption>("options"),
            user_local_data: db.collection::<UserData>("local_data"),
            _db: db,
        }
    }
}
