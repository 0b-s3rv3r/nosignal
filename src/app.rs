use crate::db::DbRepo;
use crate::error::DbError;
use crate::schema::{AppOpt, AppOption, Color, Room, RoomData, User};
use crate::util::{create_env_dir, get_passwd, get_unique_id};
use polodb_core::{bson::doc, Collection};
use strum::IntoEnumIterator;

use std::env;
use std::str::FromStr;

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
    Set(AppOpt),
    Invalid,
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
                room_id: args[2].to_owned(),
                has_password: true,
            },
            4 if args[2] == "-n" => CommandRequest::Create {
                room_id: args[3].to_owned(),
                has_password: false,
            },
            _ => CommandRequest::Invalid,
        },
        "join" => match len {
            3 => CommandRequest::Join {
                room_address: args[2].to_owned(),
                username: None,
            },
            4 => CommandRequest::Join {
                room_address: args[2].to_owned(),
                username: Some(args[3].to_owned()),
            },
            _ => CommandRequest::Invalid,
        },
        "list" => CommandRequest::List,
        "del" => match len {
            3 => CommandRequest::Delete {
                room_id: args[2].to_owned(),
            },
            _ => CommandRequest::Invalid,
        },
        "set" => match len {
            3 => CommandRequest::Set(AppOpt::from_str(&args[2].to_owned()).unwrap()),
            _ => CommandRequest::Invalid,
        },

        _ => CommandRequest::Invalid,
    }
}

pub struct App {
    pub(crate) db: DbRepo,
    local_usr: User,
}

impl App {
    pub fn init() -> App {
        let db = DbRepo::init(&create_env_dir("kioto").unwrap());
        App::db_init(db)
    }

    pub(crate) fn mem_init() -> App {
        let db = DbRepo::memory_init();
        App::db_init(db)
    }

    fn db_init(db: DbRepo) -> App {
        App::insert_app_options(&db.options);

        if let Some(local_usr) = db.user_local_data.find_one(None).unwrap() {
            App {
                db: db,
                local_usr: local_usr,
            }
        } else {
            App {
                db: db,
                local_usr: User {
                    address: "placholder".to_owned(),
                    username: "user".to_owned() + &get_unique_id(),
                    color: Color(0, 0, 0),
                },
            }
        }
    }

    fn insert_app_options(opt_db: &Collection<AppOption>) {
        opt_db.insert_many(App::create_app_option_vec()).unwrap();
    }

    fn create_app_option_vec() -> Vec<AppOption> {
        AppOpt::iter()
            .map(|opt| AppOption {
                option: opt,
                enabled: false,
            })
            .collect()
    }

    pub fn run(&self, runopt: CommandRequest) {
        match runopt {
            CommandRequest::Version => println!(env!("CARGO_PKG_VERSION")),
            CommandRequest::Help => App::print_help(),
            CommandRequest::Create {
                room_id,
                has_password,
            } => {
                if let Err(DbError::AlreadyExistingId) = self.create_room(&room_id, has_password) {
                    panic!("Room with this id already exists! Try something new!");
                }
            }
            CommandRequest::Join {
                room_address,
                username,
            } => {
                if let Some(user) = username {
                    self.join_room(&room_address, &user)
                } else {
                    self.join_room(&room_address, "")
                }
            }
            CommandRequest::Delete { room_id } => self.delete_room(&room_id),
            CommandRequest::List => self.list_rooms(),
            CommandRequest::Set(option) => self.set_app_option(option),
            CommandRequest::Invalid => {
                println!("Invalid command! Type 'kioto help' for getting help")
            }
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

    fn create_room(&self, room_id: &str, password: bool) -> Result<(), DbError> {
        if let Some(_) = self.db.rooms.find_one(doc! {"room_id": room_id}).unwrap() {
            return Err(DbError::AlreadyExistingId);
        }

        let new_room = RoomData {
            room_id: room_id.to_owned(),
            room_address: "new_address_placeholder".to_owned(),
            password: if password { Some(get_passwd()) } else { None },
            locked_addresses: vec![],
            are_you_host: true,
        };

        self.db.rooms.insert_one(&new_room).unwrap();

        Ok(())
    }

    fn delete_room(&self, room_id: &str) {
        self.db.rooms.delete_one(doc! {"room_id": room_id}).unwrap();
    }

    fn list_rooms(&self) {
        let rooms = self.db.rooms.find(None).unwrap();
        rooms.for_each(|room| println!("{}", room.unwrap().room_id));
    }

    fn join_room(&self, room_address: &str, guestname: &str) {
        todo!()
    }

    fn set_app_option(&self, option: AppOpt) {
        let current_state = self
            .db
            .options
            .find_one(doc! {"option": option.to_string()})
            .unwrap()
            .unwrap()
            .enabled;

        self.db
            .options
            .update_one(
                doc! {"option": option.to_string()},
                doc! {
                    "$set": {
                        "enabled": (!current_state)
                    }
                },
            )
            .unwrap();
    }
}
