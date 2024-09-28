mod app;
mod db;
mod error;
mod network;
mod schema;
mod tui;
mod util;

use app::{get_command_request, run};
use db::DbRepo;
use log::{error, warn};
use network::server::ChatServer;
use network::{client::ChatClient, User};
use schema::{Color, Room};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{env, io};
use tokio::time::sleep;
use tui::chat_app::ChatApp;
use util::setup_logger;

#[tokio::main]
async fn main() -> io::Result<()> {
    let t = env::args().nth(1).unwrap();

    setup_logger(None).unwrap();

    let room = Room {
        _id: "firstroom".into(),
        addr: SocketAddr::from_str("127.0.0.1:12345").unwrap(),
        passwd: None,
        banned_addrs: vec![],
        is_owner: true,
    };

    let mut room2 = room.clone();
    room2.is_owner = false;

    let user = User {
        _id: "user1".into(),
        addr: None,
        color: Color::LightRed,
    };

    let user2 = User {
        _id: "user2".into(),
        addr: None,
        color: Color::LightGreen,
    };

    let room_ = room.clone();

    let db = Arc::new(Mutex::new(DbRepo::memory_init().unwrap()));

    match t.as_str() {
        "server" => {
            println!("Starting server...");
            let mut server = ChatServer::new(room, db).await.unwrap();
            server.run().await.unwrap();

            sleep(Duration::from_secs(1)).await;

            println!("Starting client...");
            let mut client = ChatClient::new(room_, user);
            if client.connect().await.is_err() {
                error!("Unable to connect");
                return Ok(());
            }
            sleep(Duration::from_secs(1)).await;
            if !client.is_ok() {
                warn!("Connection refused");
                return Ok(());
            }

            let mut app = ChatApp::new(client, false);
            app.run().await?;

            server.stop().await;
        }
        "client" => {
            let mut client = ChatClient::new(room2.clone(), user2);
            if client.connect().await.is_err() {
                warn!("Connection refused from server {}", room2._id);
                println!("Connection refused");
                return Ok(());
            }
            sleep(Duration::from_secs(1)).await;

            let mut app = ChatApp::new(client, false);
            app.run().await?;
        }
        _ => {}
    }

    Ok(())

    // run(get_command_request(), true).unwrap();
}
