use crate::{
    db::DbRepo,
    error::AppError,
    network::{
        client::ChatClient,
        message::{Message, MessageType, ServerMsg, UserMsg},
        server::ChatServer,
        User,
    },
    schema::{Color, LocalData, RoomHeader, ServerRoom},
    tui::chat_app::ChatApp,
    util::{create_env_dir, get_unique_id, passwd_input, setup_logger},
};
use clap::{Arg, ArgMatches, Command};
use crossterm::style::Stylize;
use polodb_core::{bson::doc, Result as pdbResult};
use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::Path,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::sleep;
use tokio_util::either::Either;

pub async fn run(cmd_req: CommandRequest) -> Result<(), AppError> {
    let path = create_env_dir("kioto")?;

    let log_path = path.join("errors.log");
    setup_logger(Some(&log_path)).expect(format!("{}", "Failed to set up logger.".red()).as_str());

    let db_path = path.join("db.db");
    let db = Arc::new(Mutex::new(db_init(Some(&db_path))?));
    run_option(cmd_req, db).await?;
    Ok(())
}

async fn run_option(cmd_req: CommandRequest, db: Arc<Mutex<DbRepo>>) -> Result<(), AppError> {
    match cmd_req {
        CommandRequest::Create { room_id, password } => {
            create_room(&db.lock().unwrap(), &room_id, password)?
        }
        CommandRequest::Join { id_or_address } => join_room(id_or_address, db).await?,
        CommandRequest::Delete { room_id } => delete_room(&db.lock().unwrap(), &room_id)?,
        CommandRequest::List => list_rooms_and_local_data(&db.lock().unwrap())?,
        CommandRequest::Set { option, value } => {
            set_local_data(&db.lock().unwrap(), &option, &value)?
        }
        CommandRequest::Invalid => return Err(AppError::InvalidCommand),
    }
    Ok(())
}

pub fn db_init(db_path: Option<&Path>) -> pdbResult<DbRepo> {
    let db = DbRepo::init(db_path)?;

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

fn create_room(db: &DbRepo, room_id: &str, password: bool) -> Result<(), AppError> {
    if db.server_rooms.find_one(doc! {"_id": room_id})?.is_some() {
        return Err(AppError::AlreadyExistingId);
    }

    let addr = db
        .local_data
        .find_one(doc! {})?
        .ok_or(AppError::DataNotFound)?
        .room_addr;
    let passwd = if password { Some(passwd_input()) } else { None };

    db.server_rooms.insert_one(&ServerRoom {
        _id: room_id.into(),
        addr,
        passwd,
        banned_addrs: vec![],
    })?;
    Ok(())
}

fn delete_room(db: &DbRepo, room_id: &str) -> Result<(), AppError> {
    if db
        .server_rooms
        .delete_one(doc! {"_id": room_id})?
        .deleted_count
        > 0
    {
        return Ok(());
    }

    if db
        .messages
        .delete_many(doc! {"room_id": room_id})?
        .deleted_count
        > 0
    {
        return Ok(());
    }

    Err(AppError::NotExistingId)
}

fn list_rooms_and_local_data(db: &DbRepo) -> Result<(), AppError> {
    let local_data = db
        .local_data
        .find_one(doc! {})?
        .ok_or(AppError::DataNotFound)?;
    let local_data_print = format!(
        "Config:\n user_id: {}\n room_addr: {}\n color: {}\n light_mode: {}",
        local_data.user_id,
        local_data.room_addr.to_string(),
        local_data.color.to_string(),
        local_data.light_mode.to_string(),
    );
    println!("{}", local_data_print);

    println!("\nRooms:");
    let server_rooms = db.server_rooms.find(doc! {})?;
    let room_headers = db.room_headers.find(doc! {})?;
    server_rooms.for_each(|el| {
        let room = el.unwrap();
        println!(" {}: {} [host]", room._id, room.addr.to_string());
    });
    room_headers.for_each(|el| {
        let room = el.unwrap();
        println!(" {}: {} [guest]", room._id, room.addr.to_string());
    });
    Ok(())
}

async fn join_room(id_or_addr: IdOrAddr, db: Arc<Mutex<DbRepo>>) -> Result<(), AppError> {
    let local_data = db
        .lock()
        .unwrap()
        .local_data
        .find_one(doc! {})
        .unwrap()
        .unwrap();
    let user = User {
        id: local_data.user_id,
        addr: None,
        color: local_data.color,
    };

    match id_or_addr {
        IdOrAddr::Id(room_id) => match find_room(&db.lock().unwrap(), &room_id)? {
            Either::Left(server_room) => {
                let room_header = server_room.room_header();

                let mut server = ChatServer::new(server_room, db.clone()).await.unwrap();
                server.run().await.unwrap();
                sleep(Duration::from_secs(1)).await;

                println!("server joined");

                let mut client = ChatClient::new(room_header, user);
                if client.connect().await.is_err() {
                    return Err(AppError::ConnectionRefused);
                }
                println!("client joining");
                sleep(Duration::from_secs(1)).await;
                if !client.is_ok() {
                    return Err(AppError::ConnectionRefused);
                }

                println!("client joined");

                let mut app = ChatApp::new(client, false);
                app.run().await?;

                server.stop().await;
            }
            Either::Right(room_header) => {
                let mut client = ChatClient::new(room_header, user.clone());
                if client.connect().await.is_err() {
                    return Err(AppError::ConnectionRefused);
                }
                sleep(Duration::from_secs(1)).await;
                while let Some(MessageType::Server(ServerMsg::AuthReq { passwd_required })) =
                    client.recv_msg().await
                {
                    if passwd_required {
                        client
                            .send_msg(Message::from((UserMsg::Auth, Some(passwd_input()))))
                            .await
                            .unwrap();
                    }
                }
                if !client.is_ok() {
                    return Err(AppError::ConnectionRefused);
                }

                client
                    .send_msg(Message::from((
                        UserMsg::SyncReq,
                        client.room.lock().unwrap().passwd.clone(),
                    )))
                    .await
                    .unwrap();

                client
                    .send_msg(Message::from((
                        UserMsg::UserJoined { user: user.clone() },
                        client.room.lock().unwrap().passwd.clone(),
                    )))
                    .await
                    .unwrap();

                let mut app = ChatApp::new(client, false);
                app.run().await?;
            }
        },
        IdOrAddr::Addr(addr) => {
            let room_header = RoomHeader {
                _id: String::from(""),
                addr: SocketAddr::from_str(&addr).unwrap(),
                passwd: None,
            };

            let mut client = ChatClient::new(room_header, user.clone());
            if client.connect().await.is_err() {
                return Err(AppError::ConnectionRefused);
            }
            sleep(Duration::from_secs(1)).await;
            while let Some(MessageType::Server(ServerMsg::AuthReq { passwd_required })) =
                client.recv_msg().await
            {
                if db
                    .lock()
                    .unwrap()
                    .room_headers
                    .find_one(doc! {"addr": addr.clone()})?
                    .is_none()
                {
                    db.lock()
                        .unwrap()
                        .room_headers
                        .insert_one(client.room.lock().unwrap().clone())?;
                }

                if passwd_required {
                    client
                        .send_msg(Message::from((UserMsg::Auth, Some(passwd_input()))))
                        .await
                        .unwrap();
                }
            }
            if !client.is_ok() {
                return Err(AppError::ConnectionRefused);
            }

            client
                .send_msg(Message::from((
                    UserMsg::SyncReq,
                    client.room.lock().unwrap().passwd.clone(),
                )))
                .await
                .unwrap();

            client
                .send_msg(Message::from((
                    UserMsg::UserJoined { user: user.clone() },
                    client.room.lock().unwrap().passwd.clone(),
                )))
                .await
                .unwrap();

            let mut app = ChatApp::new(client, false);
            app.run().await?;
        }
    }
    Ok(())
}

fn set_local_data(db: &DbRepo, option: &str, value: &str) -> Result<(), AppError> {
    match option {
        "user_id" => {
            db.local_data.update_one(
                doc! {"id": 0},
                doc! {"$set": doc! {
                    option: value
                }},
            )?;
        }
        "room_addr" => {
            if SocketAddr::from_str(value).is_err() {
                return Err(AppError::InvalidArgument);
            }
            db.local_data.update_one(
                doc! {"id": 0},
                doc! {"$set": doc! {
                    option: value
                }},
            )?;
        }
        "color" => {
            if Color::from_str(value).is_err() {
                return Err(AppError::InvalidArgument);
            }
            db.local_data.update_one(
                doc! {"id": 0},
                doc! {"$set": doc! {
                    option: value
                }},
            )?;
        }
        "light_mode" => {
            if let Ok(state) = bool::from_str(value) {
                db.local_data.update_one(
                    doc! {"id": 0},
                    doc! {"$set": doc! {
                        option: state
                    }},
                )?;
            } else {
                return Err(AppError::InvalidArgument);
            }
        }
        _ => return Err(AppError::InvalidArgument),
    }

    Ok(())
}

fn find_room(db: &DbRepo, room_id: &str) -> Result<Either<ServerRoom, RoomHeader>, AppError> {
    if let Some(room) = db.server_rooms.find_one(doc! {
       "_id": room_id
    })? {
        return Ok(Either::Left(room));
    }

    if let Some(room) = db.room_headers.find_one(doc! {
       "_id": room_id
    })? {
        return Ok(Either::Right(room));
    }

    Err(AppError::NotExistingId)
}

#[derive(Debug)]
pub enum IdOrAddr {
    Id(String),
    Addr(String),
}

#[derive(Debug)]
pub enum CommandRequest {
    Create { room_id: String, password: bool },
    Join { id_or_address: IdOrAddr },
    Delete { room_id: String },
    Set { option: String, value: String },
    List,
    Invalid,
}

pub fn get_command_request() -> CommandRequest {
    match config_clap().subcommand() {
        Some(("create", create_matches)) => {
            let room_id = create_matches
                .get_one::<String>("room_id")
                .unwrap()
                .to_owned();

            let password = create_matches.get_flag("password");
            CommandRequest::Create { room_id, password }
        }
        Some(("join", join_matches)) => {
            let id_or_addr_ = join_matches
                .get_one::<String>("id_or_addr")
                .unwrap()
                .to_owned();

            let id_or_addr = if SocketAddr::from_str(&id_or_addr_).is_ok() {
                IdOrAddr::Addr(id_or_addr_)
            } else {
                IdOrAddr::Id(id_or_addr_)
            };

            CommandRequest::Join {
                id_or_address: id_or_addr,
            }
        }
        Some(("delete", delete_matches)) => {
            let room_id = delete_matches
                .get_one::<String>("room_id")
                .unwrap()
                .to_owned();
            CommandRequest::Delete { room_id }
        }
        Some(("set", set_matches)) => {
            let option_str = set_matches.get_one::<String>("option").unwrap();
            let value_str = set_matches.get_one::<String>("value").unwrap();
            CommandRequest::Set {
                option: option_str.to_string(),
                value: value_str.to_string(),
            }
        }
        Some(("list", _)) => CommandRequest::List,
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
                .arg(Arg::new("room_id").required(true)),
        )
        .subcommand(
            Command::new("join")
                .long_flag("join")
                .short_flag('j')
                .about("Joins a room")
                .arg(Arg::new("id_or_addr").required(true)),
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
    use super::{Color, CommandRequest, LocalData};
    use crate::{
        app::{db_init, run_option},
        schema::ServerRoom,
    };
    use polodb_core::bson::doc;
    use std::{
        net::SocketAddr,
        str::FromStr,
        sync::{Arc, Mutex},
    };

    #[tokio::test]
    async fn new_room_creation() {
        let db = Arc::new(Mutex::new(db_init(None).unwrap()));

        let room_with_custom_values = ServerRoom {
            _id: "someroom".into(),
            addr: SocketAddr::from_str("127.0.0.1:12345".into()).unwrap(),
            passwd: None,
            banned_addrs: vec![],
        };

        run_option(
            CommandRequest::Create {
                room_id: room_with_custom_values._id.clone(),
                password: false,
            },
            db.clone(),
        )
        .await
        .unwrap();

        assert_eq!(
            db.lock()
                .unwrap()
                .server_rooms
                .find_one(doc! {"_id": "someroom"})
                .unwrap()
                .unwrap(),
            room_with_custom_values
        );

        let room_with_default_values = ServerRoom {
            _id: "anotheroom".into(),
            addr: SocketAddr::from_str("127.0.0.1:12345").unwrap(),
            passwd: None,
            banned_addrs: vec![],
        };

        run_option(
            CommandRequest::Create {
                room_id: room_with_default_values._id.clone(),
                password: false,
            },
            db.clone(),
        )
        .await
        .unwrap();

        assert_eq!(
            db.lock()
                .unwrap()
                .server_rooms
                .find_one(doc! {"_id": "anotheroom"})
                .unwrap()
                .unwrap(),
            room_with_default_values
        );
    }

    #[tokio::test]
    async fn room_deletion() {
        let db = Arc::new(Mutex::new(db_init(None).unwrap()));

        let room = ServerRoom {
            _id: "someroom".into(),
            addr: SocketAddr::from_str("127.0.0.1:12345").unwrap(),
            passwd: None,
            banned_addrs: vec![],
        };

        run_option(
            CommandRequest::Create {
                room_id: room._id.clone(),
                password: false,
            },
            db.clone(),
        )
        .await
        .unwrap();

        assert_eq!(
            db.lock()
                .unwrap()
                .server_rooms
                .find_one(doc! {"_id": "someroom"})
                .unwrap()
                .unwrap(),
            room
        );

        run_option(
            CommandRequest::Delete {
                room_id: "someroom".into(),
            },
            db.clone(),
        )
        .await
        .unwrap();

        assert_eq!(
            db.lock()
                .unwrap()
                .server_rooms
                .find_one(doc! {"_id": "someroom"})
                .unwrap(),
            None
        );
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

    #[tokio::test]
    async fn local_data_setting() {
        let db = Arc::new(Mutex::new(db_init(None).unwrap()));

        run_option(
            CommandRequest::Set {
                option: "user_id".to_owned(),
                value: "someuser".to_owned(),
            },
            db.clone(),
        )
        .await
        .unwrap();

        assert_eq!(
            db.lock()
                .unwrap()
                .local_data
                .find_one(doc! {"_id": 0})
                .unwrap()
                .unwrap()
                .user_id,
            "someuser"
        );
    }
}
