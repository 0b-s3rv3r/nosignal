#[cfg(test)]
mod test {

    use polodb_core::bson::doc;

    use crate::{
        app::{App, CommandRequest},
        schema::{AppOpt, AppOption, RoomData},
    };

    #[test]
    fn check_rooms_db() {
        let app = App::mem_init();
        app.run(CommandRequest::Create {
            room_id: "some_room".to_owned(),
            has_password: false,
        });

        let room = RoomData {
            room_id: "some_room".to_owned(),
            room_address: "new_address_placeholder".to_owned(),
            password: None,
            locked_addresses: vec![],
            are_you_host: true,
        };

        let room_from_db = app.db.rooms.find_one(None).unwrap().unwrap();
        assert!(room_from_db == room);
    }

    #[test]
    fn check_options_db() {
        let option = AppOption {
            option: AppOpt::LightMode,
            enabled: true,
        };
        let app = App::mem_init();
        app.run(CommandRequest::Set(option.option.clone()));
        let option_from_db = app
            .db
            .options
            .find_one(doc! {"option": option.option.to_string()})
            .unwrap()
            .unwrap();
        assert_eq!(option_from_db, option);
    }
}
