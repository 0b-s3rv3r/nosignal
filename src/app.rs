use crate::{
    db::DbRepo,
    error::AppError,
    schema::{Color, LocalData, Room},
    util::{create_env_dir, get_unique_id, hash_passwd, passwd_input, setup_logger},
};
use clap::{Arg, ArgMatches, Command};
use crossterm::style::Stylize;
use log::error;
use polodb_core::{bson::doc, Result as pdbResult};
use std::{env, net::Ipv4Addr, path::Path, str::FromStr};

pub fn run(cmd_req: CommandRequest, open_memory: bool) -> i32 {
    let path = match create_env_dir("kioto") {
        Ok(path) => path,
        Err(err) => {
            error!("{}", err.to_string());
            println!("{}", "Failed to init env.".red());
            return 1;
        }
    };

    let log_path = path.join("output.log");
    setup_logger(&log_path).unwrap();

    let mut db = match db_init(open_memory, &path) {
        Ok(db) => db,
        Err(err) => {
            error!("{}", err.to_string());
            println!("{}", "Failed to init db.".red());
            return 1;
        }
    };

    match cmd_req {
        CommandRequest::Create {
            room_id,
            ip,
            password,
        } => match create_room(&mut db, &room_id, ip, password) {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        },
        CommandRequest::Join {
            id_or_address,
            username,
            color,
        } => match id_or_address {
            IdOrAddr::Id(id) => match join_room(None, Some(&id), username, color) {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            },
            IdOrAddr::Addr(addr) => match join_room(Some(&addr), None, username, color) {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            },
        },
        CommandRequest::Delete { room_id } => match delete_room(&mut db, &room_id) {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        },
        CommandRequest::List => match list_rooms_and_local_data(&db) {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        },
        CommandRequest::Set { option, value } => match set_local_data(&mut db, &option, &value) {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        },
        CommandRequest::Invalid => {
            println!(
                "{}",
                "Invalid command! Type 'kioto help' for getting help".yellow()
            );
        }
    }

    0
}

pub fn db_init(open_memory: bool, db_path: &Path) -> pdbResult<DbRepo> {
    if open_memory {
        return Ok(DbRepo::memory_init()?);
    }

    let db = DbRepo::init(db_path)?;

    if db.local_data.count_documents()? == 0 {
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

    Ok(db)
}

fn create_room(
    db: &mut DbRepo,
    room_id: &str,
    room_ip: Option<String>,
    password: bool,
) -> Result<(), AppError> {
    if db
        .rooms
        .find_one(doc! {"id": room_id})
        .map_err(|e| AppError::PdbError(e))?
        .is_some()
    {
        return Err(AppError::AlreadyExistingId);
    }

    let addr = match room_ip {
        Some(ip) => ip,
        None => {
            db.local_data
                .find_one(None)
                .map_err(|e| AppError::PdbError(e))?
                .ok_or(AppError::DataNotFound)?
                .addr
        }
    };

    let passwd = if password { Some(passwd_input()) } else { None };

    db.rooms
        .insert_one(&Room {
            id: room_id.into(),
            addr,
            passwd,
            banned_addrs: vec![],
            is_owner: false,
        })
        .map_err(|e| AppError::PdbError(e))?;

    Ok(())
}

fn delete_room(db: &mut DbRepo, room_id: &str) -> Result<(), AppError> {
    if let Some(room) = db
        .rooms
        .find_one(doc! {"room_id": room_id})
        .map_err(|e| AppError::PdbError(e))?
    {
        if room.is_owner {
            if let Some(passwd) = room.passwd {
                hash_passwd(&passwd);
                if passwd_input() != passwd {
                    return Err(AppError::InvalidPassword);
                }
            }
        }
    } else {
        return Err(AppError::NotExistingId);
    }

    db.rooms
        .delete_one(doc! {"room_id": room_id})
        .map_err(|e| AppError::PdbError(e))?;
    Ok(())
}

fn list_rooms_and_local_data(db: &DbRepo) -> Result<(), AppError> {
    let local_data = db
        .rooms
        .find_one(None)
        .map_err(|e| AppError::PdbError(e))?
        .ok_or(AppError::DataNotFound)?;

    println!("{:?}", local_data);

    let mut rooms = db.rooms.find(None).map_err(|e| AppError::PdbError(e))?;
    if !rooms.any(|el| {
        let room = el.unwrap();
        println!("{}: {}", room.id, room.addr);
        true
    }) {
        return Err(AppError::NoAnyRoom);
    }

    Ok(())
}

fn join_room(
    room_ip: Option<&str>,
    room_id: Option<&str>,
    username: Option<String>,
    color: Option<Color>,
) -> Result<(), AppError> {
    todo!("if there is no such id then join, but store info temporary")
}

fn set_local_data(db: &mut DbRepo, option: &str, value: &str) -> Result<(), AppError> {
    db.local_data
        .update_one(
            doc! {"option": option.to_string()},
            doc! {"value": value.to_string()},
        )
        .map_err(|e| AppError::PdbError(e))?;

    Ok(())
}

#[derive(Debug)]
pub enum IdOrAddr {
    Id(String),
    Addr(String),
}

#[derive(Debug)]
pub enum CommandRequest {
    Create {
        room_id: String,
        ip: Option<String>,
        password: bool,
    },
    Join {
        id_or_address: IdOrAddr,
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

pub fn get_command_request() -> CommandRequest {
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

            let password = create_matches.get_flag("password");
            CommandRequest::Create {
                room_id,
                ip: room_ip.cloned(),
                password,
            }
        }
        Some(("join", join_matches)) => {
            let id_or_addr_ = join_matches
                .get_one::<String>("id_or_addr")
                .unwrap()
                .to_owned();

            let id_or_addr = if Ipv4Addr::from_str(&id_or_addr_).is_ok() {
                IdOrAddr::Addr(id_or_addr_)
            } else {
                IdOrAddr::Id(id_or_addr_)
            };

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

            CommandRequest::Join {
                id_or_address: id_or_addr,
                username: username.cloned(),
                color,
            }
        }
        Some(("delete", delete_matches)) => {
            let room_id = delete_matches
                .get_one::<String>("room_id")
                .unwrap()
                .to_owned();
            CommandRequest::Delete { room_id }
        }
        Some(("list", _)) => CommandRequest::List,
        Some(("set", set_matches)) => {
            let option_str = set_matches.get_one::<String>("option").unwrap();
            let value_str = set_matches.get_one::<String>("value").unwrap();
            CommandRequest::Set {
                option: option_str.to_string(),
                value: value_str.to_string(),
            }
        }
        _ => CommandRequest::Invalid,
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
                .arg(Arg::new("id_or_addr").required(true))
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

#[cfg(test)]
mod test {
    use super::{run, DbRepo};
    use polodb_core::bson::doc;

    #[test]
    fn new_room_creation() {}

    #[test]
    fn room_deletion() {}

    #[test]
    fn local_data_update() {}

    #[test]
    fn data_listing() {}

    #[test]
    fn room_joining() {}
}
