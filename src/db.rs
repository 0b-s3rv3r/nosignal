use crate::schema::{LocalData, RoomHeader, ServerRoom, TextMessage};
use polodb_core::{Collection, Database, Result as pdbResult};
use std::path::Path;

pub struct DbRepo {
    pub server_rooms: Collection<ServerRoom>,
    pub room_headers: Collection<RoomHeader>,
    pub messages: Collection<TextMessage>,
    pub local_data: Collection<LocalData>,
    _db: Database,
}

impl DbRepo {
    pub fn init(filepath: &Path) -> pdbResult<Self> {
        let db = Database::open_file(filepath)?;

        Ok(DbRepo {
            server_rooms: db.collection::<ServerRoom>("server_rooms"),
            room_headers: db.collection::<RoomHeader>("room_headers"),
            messages: db.collection::<TextMessage>("messages"),
            local_data: db.collection::<LocalData>("local_data"),
            _db: db,
        })
    }

    pub fn memory_init() -> pdbResult<Self> {
        let db = Database::open_memory()?;

        Ok(DbRepo {
            server_rooms: db.collection::<ServerRoom>("server_rooms"),
            room_headers: db.collection::<RoomHeader>("room_headers"),
            messages: db.collection::<TextMessage>("messages"),
            local_data: db.collection::<LocalData>("local_data"),
            _db: db,
        })
    }
}
