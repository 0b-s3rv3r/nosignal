use crate::db::DbRepo;
use crate::error::{CommandError, DbError};
use crate::schema::{Color, LocalData, Room};
use crate::util::{create_env_dir, get_passwd, get_unique_id};

use clap::{Arg, ArgMatches, Command};
use polodb_core::bson::doc;

use std::env;
use std::str::FromStr;

pub fn run() {
    let cmd_req = get_command_request().unwrap();

    let mut db = db_init(true);

    match cmd_req {
        CommandRequest::Create {
            room_id,
            ip,
            has_password,
        } => {
            // if let Err(DbError::AlreadyExistingId) = create_room(&room_id, has_password) {
            //     panic!("Room with this id already exists! Try something new!");
            // }
        }
        CommandRequest::Join {
            room_address,
            username,
            color,
        } => {
            if let Some(user) = username {
                // join_room(&room_address, false)
            } else {
                // join_room(&room_address, false)
            }
        }
        CommandRequest::Delete { room_id } => delete_room(&mut db, &room_id),
        CommandRequest::List => list_rooms(&db),
        CommandRequest::Set { option, value } => set_app_option(&mut db, &option, &value),
        CommandRequest::Invalid => {
            println!("Invalid command! Type 'kioto help' for getting help")
        }
    }
}

pub fn db_init(open_memory: bool) -> DbRepo {
    if open_memory {
        return DbRepo::memory_init();
    }

    let db = DbRepo::init(&create_env_dir("kioto").unwrap());

    if db.local_data.count_documents().unwrap() == 0 {
        db.local_data
            .insert_one(LocalData {
                addr: "127.0.0.1:12345".into(),
                username: get_unique_id(),
                color: Color::White,
                remember_passwords: false,
                light_mode: false,
            })
            .unwrap();
    }

    db
}

fn create_room(db: &mut DbRepo, room_id: &str, passwd: Option<String>) -> Result<(), DbError> {
    if let Some(_) = db.rooms.find_one(doc! {"id": room_id}).unwrap() {
        return Err(DbError::AlreadyExistingId);
    }

    let new_room = Room {
        id: room_id.into(),
        addr: "".into(),
        passwd,
        banned_addrs: vec![],
        is_owner: false,
    };

    db.rooms.insert_one(&new_room).unwrap();

    Ok(())
}

fn delete_room(db: &mut DbRepo, room_id: &str) {
    db.rooms.delete_one(doc! {"room_id": room_id}).unwrap();
}

fn list_rooms(db: &DbRepo) {
    let rooms = db.rooms.find(None).unwrap();
    rooms.for_each(|room| println!("{}", room.unwrap().id));
}

fn join_room(room_ip: &str, room_id: &str, is_host: bool) {
    todo!()
}

fn set_app_option(db: &mut DbRepo, option: &str, value: &str) {
    let current_state = db
        .local_data
        .find_one(doc! {"option": option.to_string()})
        .unwrap()
        .unwrap();

    db.local_data
        .update_one(
            doc! {"option": option.to_string()},
            doc! {"value": value.to_string()},
        )
        .unwrap();
}

#[derive(Debug)]
pub enum CommandRequest {
    Create {
        room_id: String,
        ip: Option<String>,
        has_password: bool,
    },
    Join {
        room_address: String,
        username: Option<String>,
        color: Option<Color>,
    },
    Delete {
        room_id: String,
    },
    List,
    Set {
        option: String,
        value: String,
    },
    Invalid,
}

fn get_command_request() -> Result<CommandRequest, CommandError> {
    match config_clap().subcommand() {
        Some(("create", create_matches)) => {
            let room_id = create_matches
                .get_one::<String>("room_id")
                .unwrap()
                .to_owned();

            let room_ip = if let Some(room_ip) = create_matches.get_one::<String>("room_ip") {
                Some(room_ip)
            } else {
                None
            };

            let has_password = create_matches.get_flag("password");
            Ok(CommandRequest::Create {
                room_id,
                ip: room_ip.cloned(),
                has_password,
            })
        }
        Some(("join", join_matches)) => {
            let room_address = join_matches
                .get_one::<String>("room_address")
                .unwrap()
                .to_owned();

            let username = if let Some(username) = join_matches.get_one::<String>("username") {
                Some(username)
            } else {
                None
            };

            let color = if let Some(color) = join_matches.get_one::<String>("color") {
                Some(Color::from_str(color).unwrap())
            } else {
                None
            };

            Ok(CommandRequest::Join {
                room_address,
                username: username.cloned(),
                color,
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
            let value_str = set_matches.get_one::<String>("value").unwrap();
            Ok(CommandRequest::Set {
                option: option_str.to_string(),
                value: value_str.to_string(),
            })
        }
        _ => Ok(CommandRequest::Invalid),
    }
}

fn config_clap() -> ArgMatches {
    Command::new("kioto")
        .about("Yet another tui chat.")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
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
                .arg(Arg::new("room_id").required(true))
                .arg(Arg::new("room_ip").required(false)),
        )
        .subcommand(
            Command::new("join")
                .long_flag("join")
                .short_flag('j')
                .about("Joins a room")
                .arg(Arg::new("room_address").required(true))
                .arg(Arg::new("username").required(false))
                .arg(Arg::new("color").required(false)),
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
                .arg(Arg::new("option").required(true))
                .arg(Arg::new("value").required(true)),
        )
        .get_matches()
}
