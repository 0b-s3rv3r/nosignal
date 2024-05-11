use futures_util::{future, FutureExt, StreamExt, TryStreamExt};
use log::debug;
use ratatui::style::Style;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{io, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::tungstenite::handshake::server::Request as SRequest;
use tokio_tungstenite::tungstenite::handshake::server::Response as SResponse;
use tokio_tungstenite::{accept_async, tungstenite::Error as wsError};
use tokio_tungstenite::{accept_hdr_async, connect_async};

use crate::schema::{Color, Message, RoomData, RoomStyle, UserData};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnRequest {
    pub user: UserData,
    pub passwd: Option<String>,
}

pub struct RoomServer {
    pub listener: TcpListener,
    pub inner: Arc<Mutex<ServerInner>>,
}

pub struct ServerInner {
    pub peers: HashMap<SocketAddr, User>,
    pub msgs: Vec<Message>,
    pub data: RoomData,
}

impl RoomServer {
    pub async fn init(data: RoomData) -> Result<Self, io::Error> {
        Ok(Self {
            listener: TcpListener::bind(&data.socket_addr).await?,
            inner: Arc::new(Mutex::new(ServerInner {
                peers: HashMap::new(),
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
        let inner_data = inner.lock().await;

        let peer_addr = stream.peer_addr()?;
        debug!("Incoming connection from: {}", peer_addr);

        {
            if inner_data.data.locked_addrs.contains(&peer_addr) {
                debug!("Connection has been rejected");
                return;
            }
        }

        let new_user: User;
        let callback = |req: &SRequest, mut response: SResponse| {
            let conn_req = serde_json::from_str::<ConnRequest>(req).unwrap();

            if let Some(passwd) = inner_data.data.password {
                if let Some(req_passwd) = conn_req.passwd {
                    if passwd != req_passwd {
                        todo!()
                    }
                } else {
                    todo!()
                }
            }

            new_user = User {
                user_id: conn_req.user.user_id,
                color: conn_req.user.color,
                sender: None,
            };

            Ok(response)
        };

        let ws_stream = accept_hdr_async(stream, callback).await?;
        debug!("Connection established");

        let (tx, rx) = unbounded_channel();
        new_user.sender = Some(tx);
        inner_data.peers.insert(peer_addr, new_user);

        let (write, read) = ws_stream.split();

        let broadcast_incoming = read.try_for_each(|msg| {
            let broadcast_recipients = inner_data
                .peers
                .into_iter()
                .filter(|(addr, _)| addr != &peer_addr)
                .map(|(_, ws_sink)| ws_sink);

            for recp in broadcast_recipients {
                recp
            }

            future::ok(())
        });

        Ok(())
    }

    pub async fn block_addr(&mut self, addr: SocketAddr) {
        self.inner.lock().await.data.locked_addrs.push(addr);
    }
}

pub struct RoomClient {
    data: Arc<Mutex<RoomData>>,
    guests: Arc<Mutex<Vec<User>>>,
    pub msgs: Arc<Mutex<Vec<Message>>>,
    pub style: RoomStyle,
}

impl RoomClient {
    pub async fn conn(room: RoomData) -> Result<Self, io::Error> {
        let (ws_stream, _) = connect_async(&room.socket_addr).await?;
        let (mut read, mut write) = ws_stream.split();
        Ok(Self {
            data: Arc::new(Mutex::new(room)),
            guests: Arc::new(Mutex::new(vec![])),
            msgs: Arc::new(Mutex::new(vec![])),
            style: RoomStyle {
                bg: room.style.bg,
                fg: room.style.fg,
            },
        })
    }

    pub async fn run(&mut self) {
        loop {
            // if let Some(msg)
        }
    }

    pub async fn send_msg(&mut self, msg: &Message) {
        let json_msg = serde_json::to_string(&msg)?;
    }

    pub async fn receive_msg(&self) {}
}

pub struct User {
    pub user_id: String,
    pub color: Color,
    pub sender: Option<UnboundedSender<Message>>,
}

async fn get_public_addr() -> Result<SocketAddr, reqwest::Error> {
    let response = reqwest::get("https://api.ipify.org").await?.text().await?;

    Ok(SocketAddr::from_str(&response).unwrap())
}
