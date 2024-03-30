use crate::schema::{self, Room};
use crate::db::DbRepo;
use crate::error::DbError;

use polodb_core::bson::doc;
use std::env;

pub enum CommandRequest {
    Version,
    Help,
    Create {
        room_id: String,
        has_password: bool,
    },
    Join {
        room_address: String,
        username: Option<String>,
    },
    Delete {
        room_id: String,
    },
    List,
    Set(AppOption),
    Invalid,
}

pub enum AppOption {
    RememberPassword
}

pub fn get_command_request() -> CommandRequest {
    let args: Vec<String> = env::args().collect();
    let len = args.len();

    if len < 2 {
        return CommandRequest::Invalid;
    }

    let command = &args[1];
    match command.as_str() {
        "version" => CommandRequest::Version,
        "help" => CommandRequest::Help,
        "create" => match len {
            3 => CommandRequest::Create {
                room_id: args[2].clone(),
                has_password: true,
            },
            4 if args[2] == "-n" => CommandRequest::Create {
                room_id: args[3].clone(),
                has_password: false,
            },
            _ => CommandRequest::Invalid,
        },
        "join" => match len {
            3 => CommandRequest::Join {
                room_address: args[2].clone(),
                username: None,
            },
            4 => CommandRequest::Join {
                room_address: args[2].clone(),
                username: Some(args[3].clone()),
            },
            _ => CommandRequest::Invalid,
        },
        "list" => CommandRequest::List,
        "del" => match len {
            3 => CommandRequest::Delete {
                room_id: args[2].clone(),
            },
            _ => CommandRequest::Invalid,
        },
        _ => CommandRequest::Invalid,
    }
}

pub struct App {
    db: DbRepo
}

impl App {
    pub fn run() {
        match get_command_request() {
            CommandRequest::Version => println!(env!("CARGO_PKG_VERSION")),
            CommandRequest::Help => Self::print_help(),
            CommandRequest::Create {
                room_id,
                has_password,
            } => App::create_room(&room_id, has_password),
            CommandRequest::Join {
                room_address,
                username,
            } => App::join_room(&room_address, &username),
            CommandRequest::Delete { room_id } => App::delete_room(&room_id),
            CommandRequest::List => App::list_rooms(),
            CommandRequest::Set(option) => App::set_app_option(option),
            CommandRequest::Invalid => println!("Invalid command! Type 'kioto help' for getting help"),
        }
    }

    fn print_help() {
        println!("Commands:\n
            kioto version - print version of kioto\n
            kioto help - list all commands\n
            kioto create <room_name> - create new room with password\n
                -n without password\n
            kioto join <room_id> - join room with last use or if was not set with random name and color\n
            kioto join <room_id> <username(color)> - join room with user specified username and color\n
            kioto list - list all rooms that you've already joined\n
            kioto del <room_id> - delete room");
    }

    fn create_room(&mut self, room_id: &str, password: bool) -> Result<(), DbError> {
        if let Some(_) = self.db.rooms().find_one(doc! {"room_id": room_id}).unwrap() {
            return Err(DbError::AlreadyExistingId);
        }

        let password = get_password();
            let local_user = get_local_user();
            let new_room = Room {
                room_id: room_id.to_owned(),
                room_address: generate_new_address(),
                password: password,
                host: get_local_user(),
                guests: vec![local_user],
            };
            if let Err(err) db[ROOM].insert_one(new_room.prepare_data()) {
                return err;
            }

            Ok()
    }

    fn delete_room(room_id: &str) {
match db[ROOM].delete_one(room_id) {
    
}
    }

    fn list_rooms() {
        let rooms = db[ROOM].find();
        rooms.iter().for_each(|room| println!("{}", room.room_id));
        }

    fn join_room(room_address: &str, guestname: &str) {
    
        }

        fn set_app_option(option: AppOption) {} 

}
