use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, SinkExt, StreamExt};
use log::debug;
use std::{
    collections::HashMap,
    env,
    io::{self},
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::error::Error;
use tokio_tungstenite::tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type Rx = UnboundedReceiver<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

pub struct ChatServer {
    peer_map: PeerMap,
    listener: TcpListener,
}

impl ChatServer {
    pub async fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        debug!("Listening on {addr}");

        Ok(Self {
            peer_map: PeerMap::new(Mutex::new(HashMap::new())),
            listener,
        })
    }

    pub async fn run(&self) -> io::Result<()> {
        while let Ok((stream, addr)) = self.listener.accept().await {
            tokio::spawn(ChatServer::handle_conection(
                self.peer_map.clone(),
                stream,
                addr,
            ));
        }

        Ok(())
    }

    async fn handle_conection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
        let ws_stream = tokio_tungstenite::accept_async(raw_stream).await.unwrap();
        debug!("Connection established with {addr}");

        let (tx, rx) = unbounded();
        peer_map.lock().unwrap().insert(addr, tx);

        let (outgoing, incoming) = ws_stream.split();

        let broadcast_incoming = incoming.try_for_each(|msg| {
            debug!("{} from {}", msg.to_text().unwrap(), addr);

            let peers = peer_map.lock().unwrap();
            let broadcast_recipients = peers
                .iter()
                .filter(|(peer_addr, _)| peer_addr != &&addr)
                .map(|(_, ws_sink)| ws_sink);

            for recp in broadcast_recipients {
                recp.unbounded_send(msg.clone()).unwrap();
                debug!("{msg}");
            }

            future::ok(())
        });

        let receive_from_others = rx.map(Ok).forward(outgoing);

        pin_mut!(broadcast_incoming, receive_from_others);
        future::select(broadcast_incoming, receive_from_others).await;

        peer_map.lock().unwrap().remove(&addr);

        debug!("{} disconnected", &addr);
    }
}

pub struct Client {
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

impl Client {
    pub async fn connect(url: &str) -> Result<Self, Error> {
        let (ws_stream, _) = connect_async(url).await?;
        let (write, read) = ws_stream.split();

        let (tx, mut rx) = mpsc::channel::<Message>(100);
        let (tx_in, rx_in) = mpsc::channel::<Message>(100);

        // Task for reading from WebSocket and sending to receiver channel
        task::spawn(async move {
            let mut read = read;
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(msg) => {
                        if tx_in.send(msg).await.is_err() {
                            eprintln!("Receiver dropped");
                            return;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading message: {}", e);
                        return;
                    }
                }
            }
        });

        // Task for sending messages from the sender channel to WebSocket
        task::spawn(async move {
            let mut write = write;
            while let Some(msg) = rx.recv().await {
                if let Err(e) = write.send(msg).await {
                    eprintln!("Error sending message: {}", e);
                    return;
                }
            }
        });

        Ok(Client {
            sender: tx,
            receiver: rx_in,
        })
    }

    pub async fn send(&self, msg: Message) -> Result<(), mpsc::error::SendError<Message>> {
        self.sender.send(msg).await
    }

    pub async fn receive(&mut self) -> Option<Message> {
        if self.receiver.is_empty() {
            return None;
        }
        self.receiver.recv().await
    }
}

// pub struct User {
//     pub user_id: String,
//     pub color: Color,
//     pub sender: Option<UnboundedSender<Message>>,
// }
//
// async fn get_public_addr() -> Result<SocketAddr, reqwest::Error> {
//     let response = reqwest::get("https://api.ipify.org").await?.text().await?;
//
//     Ok(SocketAddr::from_str(&response).unwrap())
// }
