use crate::{
    db::DbRepo,
    error::AppError,
    schema::{Color, LocalData, Room},
    util::{create_env_dir, get_unique_id, hash_passwd, passwd_input, setup_logger},
};
use clap::{Arg, ArgMatches, Command};
use crossterm::style::Stylize;
use polodb_core::{bson::doc, Result as pdbResult};
use std::{
    any::Any,
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
    str::FromStr,
};

pub fn run(cmd_req: CommandRequest, open_memory: bool) -> Result<(), AppError> {
    let path = create_env_dir("kioto")?;

    let log_path = path.join("errors.log");
    setup_logger(Some(&log_path)).expect(format!("{}", "Failed to set up logger.".red()).as_str());

    let mut db = if open_memory {
        db_init(None)?
    } else {
        db_init(Some(&path))?
    };

    run_option(cmd_req, &mut db)?;

    Ok(())
}

fn run_option(cmd_req: CommandRequest, db: &DbRepo) -> Result<(), AppError> {
    match cmd_req {
        CommandRequest::Create {
            room_id,
            ip,
            password,
        } => create_room(db, &room_id, ip, password)?,
        CommandRequest::Join {
            id_or_address,
            username,
            color,
        } => join_room(id_or_address, username, color)?,
        CommandRequest::Delete { room_id } => delete_room(&db, &room_id)?,
        CommandRequest::List => list_rooms_and_local_data(&db)?,
        CommandRequest::Set { option, value } => set_local_data(db, &option, &value)?,
        CommandRequest::Invalid => return Err(AppError::InvalidCommand),
    }

    Ok(())
}

pub fn db_init(db_path: Option<&Path>) -> pdbResult<DbRepo> {
    let db = if let Some(path) = db_path {
        DbRepo::init(path)?
    } else {
        DbRepo::memory_init()?
    };

    if db.local_data.count_documents()? == 0 {
        db.local_data.insert_one(&LocalData {
            id: 0,
            user_id: get_unique_id(),
            room_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12345),
            color: Color::White,
            light_mode: false,
        })?;
    }

    Ok(db)
}

fn create_room(
    db: &DbRepo,
    room_id: &str,
    room_ip: Option<String>,
    password: bool,
) -> Result<(), AppError> {
    if db.rooms.find_one(doc! {"_id": room_id})?.is_some() {
        return Err(AppError::AlreadyExistingId);
    }

    let addr = match room_ip {
        Some(ip) => SocketAddr::from_str(&ip).unwrap(),
        None => {
            db.local_data
                .find_one(None)?
                .ok_or(AppError::DataNotFound)?
                .room_addr
        }
    };

    let passwd = if password { Some(passwd_input()) } else { None };

    db.rooms.insert_one(&Room {
        _id: room_id.into(),
        addr,
        passwd,
        banned_addrs: vec![],
        is_owner: true,
    })?;

    Ok(())
}

fn delete_room(db: &DbRepo, room_id: &str) -> Result<(), AppError> {
    if let Some(room) = db.rooms.find_one(doc! {"_id": room_id})? {
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

    db.rooms.delete_one(doc! {"_id": room_id})?;
    Ok(())
}

fn list_rooms_and_local_data(db: &DbRepo) -> Result<(), AppError> {
    let local_data = db.rooms.find_one(None)?.ok_or(AppError::DataNotFound)?;

    println!("{:#?}", local_data);

    let mut rooms = db.rooms.find(None)?;
    if !rooms.any(|el| {
        let room = el.unwrap();
        println!("{}: {}", room._id, room.addr.to_string());
        true
    }) {
        return Err(AppError::NoAnyRoom);
    }

    Ok(())
}

fn join_room(
    id_or_addr: IdOrAddr,
    username: Option<String>,
    color: Option<Color>,
) -> Result<(), AppError> {
    todo!("if there is no such id then join, then store info temporary")
}

fn set_local_data(db: &DbRepo, option: &str, value: &str) -> Result<(), AppError> {
    db.local_data.update_one(
        doc! {"id": 0},
        doc! {"$set": doc! {
            option: value
        }},
    )?;

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
    use std::{net::SocketAddr, str::FromStr};

    use crate::app::{db_init, run_option};

    use super::{Color, CommandRequest, LocalData, Room};
    use polodb_core::bson::doc;

    #[test]
    fn new_room_creation() {
        let db = db_init(None).unwrap();

        let room_with_custom_values = Room {
            _id: "someroom".into(),
            addr: SocketAddr::from_str("127.0.0.1:8080".into()).unwrap(),
            passwd: None,
            banned_addrs: vec![],
            is_owner: true,
        };

        run_option(
            CommandRequest::Create {
                room_id: room_with_custom_values._id.clone(),
                ip: Some(room_with_custom_values.addr.to_string()),
                password: false,
            },
            &db,
        )
        .unwrap();

        assert_eq!(
            db.rooms
                .find_one(doc! {"_id": "someroom"})
                .unwrap()
                .unwrap(),
            room_with_custom_values
        );

        let room_with_default_values = Room {
            _id: "anotheroom".into(),
            addr: SocketAddr::from_str("127.0.0.1:12345").unwrap(),
            passwd: None,
            banned_addrs: vec![],
            is_owner: true,
        };

        run_option(
            CommandRequest::Create {
                room_id: room_with_default_values._id.clone(),
                ip: None,
                password: false,
            },
            &db,
        )
        .unwrap();

        assert_eq!(
            db.rooms
                .find_one(doc! {"_id": "anotheroom"})
                .unwrap()
                .unwrap(),
            room_with_default_values
        );
    }

    #[test]
    fn room_deletion() {
        let db = db_init(None).unwrap();

        let room = Room {
            _id: "someroom".into(),
            addr: SocketAddr::from_str("127.0.0.1:8080").unwrap(),
            passwd: None,
            banned_addrs: vec![],
            is_owner: true,
        };

        run_option(
            CommandRequest::Create {
                room_id: room._id.clone(),
                ip: Some(room.addr.to_string()),
                password: false,
            },
            &db,
        )
        .unwrap();

        assert_eq!(
            db.rooms
                .find_one(doc! {"_id": "someroom"})
                .unwrap()
                .unwrap(),
            room
        );

        run_option(
            CommandRequest::Delete {
                room_id: "someroom".into(),
            },
            &db,
        )
        .unwrap();

        assert_eq!(db.rooms.find_one(doc! {"_id": "someroom"}).unwrap(), None);
    }

    #[test]
    fn local_data_default_init() {
        let db = db_init(None).unwrap();

        let local_data = LocalData {
            id: 0,
            user_id: "*".into(),
            room_addr: SocketAddr::from_str("127.0.0.1:12345").unwrap(),
            color: Color::White,
            light_mode: false,
        };

        let local_data_from_db = db.local_data.find_one(None).unwrap().unwrap();

        assert_eq!(local_data_from_db.room_addr, local_data.room_addr);
        assert_eq!(local_data_from_db.color, local_data.color);
        assert_eq!(local_data_from_db.light_mode, local_data.light_mode);
    }

    #[test]
    fn local_data_setting() {
        let db = db_init(None).unwrap();

        run_option(
            CommandRequest::Set {
                option: "user_id".to_owned(),
                value: "someuser".to_owned(),
            },
            &db,
        )
        .unwrap();

        assert_eq!(
            db.local_data
                .find_one(doc! {"id": 0})
                .unwrap()
                .unwrap()
                .user_id,
            "someuser"
        );
    }

    #[test]
    fn room_joining() {}
}
