// mod app;
// mod db;
// mod error;
mod network;
// mod schema;
// mod test;
mod ui;
// mod util;

use std::io;

use network::{ChatServer, Client};
use ui::App;

use std::env;
use tokio;

#[tokio::main]
async fn main() -> io::Result<()> {
    let t = env::args().nth(1).unwrap();
    match t.as_str() {
        "server" => {
            println!("Starting server...");
            tokio::spawn(async move {
                let server = ChatServer::new("127.0.0.1:12345").await.unwrap();
                server.run().await.unwrap();
            });

            let client = Client::connect("ws://127.0.0.1:12345/").await.unwrap();

            let mut app = App::new(client);
            app.run().await?;
        }
        "client" => {
            let client = Client::connect("ws://127.0.0.1:12345/").await.unwrap();

            let mut app = App::new(client);
            app.run().await?;
        }
        _ => {}
    }

    Ok(())
}
