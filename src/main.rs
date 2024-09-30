mod app;
mod db;
mod error;
mod network;
mod schema;
mod tui;
mod util;

use app::{get_command_request, run};
use error::AppError;
use log::error;

#[tokio::main]
async fn main() {
    if let Err(err) = run(get_command_request()).await {
        match err {
            AppError::PdbError(err) => error!("{}", err),
            // match err {
            //     polodb_core::Error::UnexpectedIdType(_, _)
            //     | polodb_core::Error::BsonErr(_)
            //     | polodb_core::Error::BsonDeErr(_) => println!("Invalid arguments!"),
            //     _ => error!("{}", err),
            // },
            AppError::IoError(err) => error!("{}", err),
            AppError::AlreadyExistingId => println!("{}", err),
            AppError::DataNotFound => println!("{}", err),
            AppError::AuthFailure => println!("{}", err),
            AppError::ConnectionRefused => println!("{}", err),
            AppError::NotExistingId => println!("{}", err),
            AppError::InvalidArgument => println!("{}", err),
            AppError::InvalidCommand => println!("{}", err),
        }
        std::process::exit(1);
    }
    std::process::exit(0);
}

// use db::DbRepo;
// use log::{error, warn};
// use network::server::ChatServer;
// use network::{client::ChatClient, User};
// use schema::{Color, ServerRoom};
// use std::net::SocketAddr;
// use std::str::FromStr;
// use std::sync::{Arc, Mutex};
// use std::time::Duration;
// use std::{env, io};
// use tokio::time::sleep;
// use tui::chat_app::ChatApp;
// use util::setup_logger;
//

// #[tokio::main]
// async fn main() -> io::Result<()> {
//     let t = env::args().nth(1).unwrap();
//
//     setup_logger(None).unwrap();
//
//     let room = ServerRoom {
//         _id: "firstroom".into(),
//         addr: SocketAddr::from_str("127.0.0.1:12345").unwrap(),
//         passwd: None,
//         banned_addrs: vec![],
//     };
//
//     let room_header = room.room_header();
//
//     let user = User {
//         id: "user1".into(),
//         addr: None,
//         color: Color::LightRed,
//     };
//
//     let user2 = User {
//         id: "user2".into(),
//         addr: None,
//         color: Color::LightGreen,
//     };
//
//     let db = Arc::new(Mutex::new(DbRepo::memory_init().unwrap()));
//
//     match t.as_str() {
//         "server" => {
//             let mut server = ChatServer::new(room, db).await.unwrap();
//             server.run().await.unwrap();
//             sleep(Duration::from_secs(1)).await;
//
//             let mut client = ChatClient::new(room_header, user);
//             if client.connect().await.is_err() {
//                 error!("Unable to connect");
//                 return Ok(());
//             }
//             sleep(Duration::from_secs(1)).await;
//             if !client.is_ok() {
//                 warn!("Connection refused");
//                 return Ok(());
//             }
//
//             let mut app = ChatApp::new(client, false);
//             app.run().await.unwrap();
//
//             server.stop().await;
//         }
//         "client" => {
//             let mut client = ChatClient::new(room_header.clone(), user2);
//             if client.connect().await.is_err() {
//                 warn!("Connection refused from server {}", room_header._id);
//                 println!("Connection refused");
//                 return Ok(());
//             }
//             sleep(Duration::from_secs(1)).await;
//
//             let mut app = ChatApp::new(client, false);
//             app.run().await.unwrap();
//         }
//         _ => {}
//     }
//
//     Ok(())
// }
