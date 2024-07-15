// mod app;
// mod db;
// mod error;
// mod schema;
// mod test;
mod tui;
// mod util;
mod network;

use network::client::ChatClient;
use network::server::ChatServer;
use tui::chat_app::ChatApp;

use std::{env, io};

#[tokio::main]
async fn main() -> io::Result<()> {
    let t = env::args().nth(1).unwrap();
    match t.as_str() {
        "server" => {
            println!("Starting server...");
            tokio::spawn(async move {
                let server = ChatServer::new("192.168.0.157:12345").await.unwrap();
                server.run().await.unwrap();
            });

            let client = ChatClient::connect("ws://192.168.0.157:12345/")
                .await
                .unwrap();

            let mut app = ChatApp::new(client);
            app.run().await?;
        }
        "client" => {
            let client = ChatClient::connect("ws://192.168.0.157:12345/")
                .await
                .unwrap();

            let mut app = ChatApp::new(client);
            app.run().await?;
        }
        _ => {}
    }

    Ok(())
}
