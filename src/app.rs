use crate::db::DbRepo;
use crate::error::{CommandError, DbError};
use crate::schema::{AppOpt, AppOption, Color, RoomData, UserData};
use crate::util::{create_env_dir, get_passwd, get_unique_id};

use clap::{Arg, ArgAction, ArgMatches, Command};
use log::error;
use polodb_core::{bson::doc, Collection};
use strum::IntoEnumIterator;

use std::env;
use std::net::SocketAddr;
use std::str::FromStr;

pub enum CommandRequest {
    Init,
    Deinit,
    Addr {
        addr: SocketAddr,
    },
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

pub fn get_command_request() -> Result<CommandRequest, CommandError> {
    match set_clap_commands().subcommand() {
        Some(("addr", addr_matches)) => {
            let arg_addr = addr_matches.get_one::<String>("ipv4").unwrap();
            let addr = SocketAddr::from_str(&arg_addr).map_err(|_| CommandError::InvalidIpv4)?;

            Ok(CommandRequest::Addr { addr: addr })
        }
        Some(("create", create_matches)) => {
            let room_id = create_matches
                .get_one::<String>("room_id")
                .unwrap()
                .to_owned();
            let has_password = create_matches.get_flag("password");
            Ok(CommandRequest::Create {
                room_id,
                has_password,
            })
        }
        Some(("join", join_matches)) => {
            let room_address = join_matches.get_one::<String>("room_address").unwrap();
            let username = join_matches.get_one::<String>("username");
            Ok(CommandRequest::Join {
                room_address: room_address.to_owned(),
                username: username.cloned(),
            })
        }
        Some(("delete", delete_matches)) => {
            let room_id = delete_matches
                .get_one::<String>("room_id")
                .unwrap()
                .to_owned();
            Ok(CommandRequest::Delete { room_id })
        }
        Some(("list", _)) => Ok(CommandRequest::List),
        Some(("set", set_matches)) => {
            let option_str = set_matches.get_one::<String>("option").unwrap();
            let option = AppOpt::from_str(option_str).unwrap();
            Ok(CommandRequest::Set(option))
        }
        _ => Ok(CommandRequest::Invalid),
    }
}

pub fn set_clap_commands() -> ArgMatches {
    Command::new("kioto")
        .about("Yet another tui chat.")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("addr")
                .long_flag("addr")
                .short_flag('a')
                .about("Specify custom IPv4")
                .arg(Arg::new("ipv4").required(true)),
        )
        .subcommand(
            Command::new("create")
                .long_flag("create")
                .short_flag('c')
                .about("Creates a new room")
                .arg(
                    Arg::new("password")
                        .long("password")
                        .short('p')
                        .num_args(0)
                        .required(false),
                )
                .arg(Arg::new("room_id").required(true)),
        )
        .subcommand(
            Command::new("join")
                .long_flag("join")
                .short_flag('j')
                .about("Joins a room")
                .arg(Arg::new("room_address").required(true))
                .arg(Arg::new("username")),
        )
        .subcommand(
            Command::new("delete")
                .long_flag("delete")
                .short_flag('d')
                .about("Deletes a room")
                .arg(Arg::new("room_id").required(true)),
        )
        .subcommand(
            Command::new("list")
                .about("Lists all rooms")
                .long_flag("list")
                .short_flag('l'),
        )
        .subcommand(
            Command::new("set")
                .long_flag("set")
                .short_flag('s')
                .about("Sets an application option")
                .arg(Arg::new("option").required(true)),
        )
        .get_matches()
}

pub struct App {
    pub(crate) db: DbRepo,
    local_usr: UserData,
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
        if db.options.count_documents().unwrap() == 0 {
            App::insert_app_options(&db.options);
        }

        if let Some(local_usr) = db.user_local_data.find_one(None).unwrap() {
            App {
                db: db,
                local_usr: local_usr,
            }
        } else {
            App {
                db: db,
                local_usr: UserData {
                    user_id: "user".to_owned() + &get_unique_id(),
                    color: Color(0, 0, 0),
                    addr: SocketAddr::from_str("placeholder").unwrap(),
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

    pub fn run(&mut self, runopt: CommandRequest) {
        match runopt {
            CommandRequest::Addr { addr } => {
                self.local_usr.addr = addr;
                self.db
                    .user_local_data
                    .update_one(
                        doc! {"username": self.local_usr.user_id },
                        doc! {"addr": bson::to_bson(&self.local_usr.addr).unwrap() },
                    )
                    .unwrap();
            }
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
            socket_addr: SocketAddr::from_str("new_address_placeholder").unwrap(),
            password: if password { Some(get_passwd()) } else { None },
            locked_addrs: vec![],
            is_owner: true,
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
