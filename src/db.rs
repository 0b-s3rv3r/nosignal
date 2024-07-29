use crate::schema::{LocalData, Room, TextMessage};
// use bson::doc;
use polodb_core::{Collection, Database, Result as pdbResult};
// use serde::{Deserialize, Serialize};
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

    pub(crate) fn memory_init() -> pdbResult<Self> {
        let db = Database::open_memory()?;

        Ok(DbRepo {
            rooms: db.collection::<Room>("rooms"),
            messages: db.collection::<TextMessage>("messages"),
            local_data: db.collection::<LocalData>("local_data"),
            _db: db,
        })
    }
}

// pub struct Store<T: Serialize + for<'b> Deserialize<'b>>(Collection<T>);
//
// impl<T: Serialize + for<'b> Deserialize<'b>> Store<T> {
//     pub fn init(db: &Database, name: &str) -> Self {
//         Self(db.collection::<T>(name))
//     }
//
//     pub fn get_one(&self, id: Option<String>) /* -> pdbResult<Option<T>> */ {
//         let result = self.0.find(None).unwrap();
//
//         // if let Some(id_) = id {
//         // } else {
//        // }
//     }
//
//     pub fn get_many(&self, id: Option<String>) -> pdbResult<Option<Vec<T>>> {
//         if let Some(id_) = id {
//             let result = self.0.find(doc! {"id": id_})?.;
//             Ok(())
//         } else {
//             Ok(self.0.find(None)?)
//         }
//     }
//
//     pub fn insert(&self, lcl_data: &LocalData) -> pdbResult<()> {
//         Ok(self.0.insert_one(lcl_data)?)
//     }
// }
