use futures_util::TryStreamExt;
use futures_util::{future, FutureExt, StreamExt};
use log::debug;
use std::io;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Error as wsError;

use crate::schema::{Color, Message, RoomData};

pub struct RoomServer {
    pub listener: TcpListener,
    pub inner: Arc<Mutex<ServerInner>>,
}

pub struct ServerInner {
    pub guests: Vec<User>,
    pub msgs: Vec<Message>,
    pub data: RoomData,
}

impl RoomServer {
    pub async fn init(data: RoomData) -> Result<Self, io::Error> {
        Ok(Self {
            listener: TcpListener::bind(&data.socket_addr).await?,
            inner: Arc::new(Mutex::new(ServerInner {
                guests: vec![],
                msgs: vec![],
                data: data,
            })),
        })
    }

    pub async fn run(&self) -> Result<(), io::Error> {
        loop {
            if let Ok((stream, _)) = self.listener.accept().await {
                let server_inner = self.inner.clone();
                tokio::spawn(async move { RoomServer::handle_conn(stream, server_inner) });
            }
        }
    }

    async fn handle_conn(stream: TcpStream, inner: Arc<Mutex<ServerInner>>) -> Result<(), wsError> {
        debug!("Incoming connection from: {}", stream.peer_addr().unwrap());
        let ws_stream = accept_async(stream).await?;
        debug!("Connection established");

        let (write, read) = ws_stream.split();

        read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
            .forward(write)
            .await?;
        debug!("Message forwarded");

        Ok(())
    }

    pub async fn block_addr(&mut self, addr: SocketAddr) {
        self.inner.lock().await.data.locked_addrs.push(addr);
    }
}

struct RoomClient {
    stream: TcpStream,
    guests: Vec<Arc<Mutex<User>>>,
    msgs: Vec<Arc<Mutex<Message>>>,
    data: Arc<Mutex<RoomData>>,
}

impl RoomClient {
    pub async fn conn(room: RoomData) -> Result<Self, io::Error> {
        Ok(Self {
            stream: TcpStream::connect(&room.socket_addr).await?,
            guests: vec![],
            msgs: vec![],
            data: Arc::new(Mutex::new(room)),
        })
    }

    pub async fn send_msg(&self, msg: &Message) {}
    pub async fn receive_msg(&self, msg: &Message) {}
}

pub struct User {
    pub user_id: String,
    pub color: Color,
    pub addr: TcpStream,
}

async fn get_public_addr() -> Result<SocketAddr, reqwest::Error> {
    let response = reqwest::get("https://api.ipify.org").await?.text().await?;

    Ok(SocketAddr::from_str(&response).unwrap())
}
