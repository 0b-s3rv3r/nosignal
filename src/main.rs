mod app;
mod db;
mod error;
mod network;
mod schema;
mod tui;
mod util;

use db::DbRepo;
// use app::{get_command_request, run};
use network::server::ChatServer;
use network::{client::ChatClient, User};
use schema::{Color, Room};
use std::sync::{Arc, Mutex};
use std::{env, io};
use tui::chat_app::ChatApp;

#[tokio::main]
async fn main() -> io::Result<()> {
    let t = env::args().nth(1).unwrap();

    let room = Room {
        id: "firstroom".into(),
        addr: "127.0.0.1:12345".into(),
        passwd: None,
        banned_addrs: vec![],
        is_owner: true,
    };

    let user = User {
        id: "user1".into(),
        addr: None,
        color: Color::LightRed,
    };

    let user2 = User {
        id: "user2".into(),
        addr: None,
        color: Color::LightGreen,
    };

    let room_ = room.clone();

    let db = Arc::new(Mutex::new(DbRepo::memory_init().unwrap()));

    match t.as_str() {
        "server" => {
            println!("Starting server...");
            tokio::spawn(async move {
                let server = ChatServer::new(db, room).await.unwrap();
                server.run().await.unwrap();
            });

            let client = ChatClient::connect(room_, &user, true).await.unwrap();

            let mut app = ChatApp::new(client);
            app.run().await?;
        }
        "client" => {
            let client = ChatClient::connect(room, &user2, false).await.unwrap();

            let mut app = ChatApp::new(client);
            app.run().await?;
        }
        _ => {}
    }

    Ok(())

    // run(get_command_request(), true).unwrap();
}
