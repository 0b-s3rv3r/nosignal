use crate::schema::{LocalData, Room, TextMessage};
use bson::doc;
use log::error;
use polodb_core::{Collection, Database, Result as pdbResult};
use std::path::Path;

pub struct DbRepo {
    pub rooms: Collection<Room>,
    pub messages: Collection<TextMessage>,
    pub local_data: Collection<LocalData>,
    _db: Database,
}

impl DbRepo {
    pub fn init(filepath: &Path) -> pdbResult<Self> {
        let db = Database::open_file(filepath)?;

        Ok(DbRepo {
            rooms: db.collection::<Room>("rooms"),
            messages: db.collection::<TextMessage>("messages"),
            local_data: db.collection::<LocalData>("local_data"),
            _db: db,
        })
    }

    pub fn memory_init() -> pdbResult<Self> {
        let db = Database::open_memory()?;

        Ok(DbRepo {
            rooms: db.collection::<Room>("rooms"),
            messages: db.collection::<TextMessage>("messages"),
            local_data: db.collection::<LocalData>("local_data"),
            _db: db,
        })
    }

    pub fn room_update(&self, room: &Room) -> Result<(), polodb_core::Error> {
        self.rooms.update_one(
            doc! {
                "_id": room._id.clone()
            },
            doc! {
                "$set": doc! {
                    "banned_addrs": room.banned_addrs.iter().map(|sa| sa.to_string()).collect::<Vec<String>>(),
                }
            },
        )?;
        Ok(())
    }
}
