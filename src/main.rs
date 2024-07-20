mod app;
mod db;
mod error;
mod network;
mod schema;
mod tui;
mod util;

// use app::{get_command_request, run};
use network::client::ChatClient;
use network::server::ChatServer;
use std::{env, io};
use tui::chat_app::ChatApp;

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

    // run(get_command_request(), true)
}
