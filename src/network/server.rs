use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use log::debug;
use std::{
    collections::HashMap,
    io::{self},
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async, tungstenite::protocol::Message, tungstenite::Error as TtError,
};

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[derive(Debug)]
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

    async fn handle_conection(
        peer_map: PeerMap,
        raw_stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<(), TtError> {
        let ws_stream = accept_async(raw_stream).await?;
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

        Ok(())
    }
}
