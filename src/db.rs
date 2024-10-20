use crate::schema::{Config, RoomHeader, ServerRoom, TextMessage};
use polodb_core::{Collection, Database, Result as pdbResult};
use std::path::Path;

pub struct DbRepo {
    pub server_rooms: Collection<ServerRoom>,
    pub room_headers: Collection<RoomHeader>,
    pub messages: Collection<TextMessage>,
    pub local_data: Collection<Config>,
    _db: Database,
}

impl DbRepo {
    pub fn new(filepath: &Path) -> pdbResult<Self> {
        let db = Database::open_path(filepath)?;

        Ok(DbRepo {
            local_data: db.collection::<Config>("config"),
            messages: db.collection::<TextMessage>("messages"),
            room_headers: db.collection::<RoomHeader>("room_headers"),
            server_rooms: db.collection::<ServerRoom>("server_rooms"),
            _db: db,
        })
    }
}
