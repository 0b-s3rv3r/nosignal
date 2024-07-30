use super::{Message, MessageType, ServerMsg};
use crate::{db::DbRepo, schema::Room, util::hash_passwd};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use log::debug;
use std::{
    collections::HashMap,
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async, tungstenite::protocol::Message as ttMessage, tungstenite::Error as TtError,
};

type Tx = UnboundedSender<ttMessage>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

// #[derive(Debug)]
pub struct ChatServer {
    peer_map: PeerMap,
    listener: TcpListener,
    room: Room,
    db: Arc<Mutex<DbRepo>>,
}

impl ChatServer {
    pub async fn new(db: Arc<Mutex<DbRepo>>, room: Room) -> io::Result<Self> {
        let listener = TcpListener::bind(&room.addr).await?;
        debug!("Listening on {}", room.addr);

        Ok(Self {
            peer_map: PeerMap::new(Mutex::new(HashMap::new())),
            listener,
            room,
            db,
        })
    }

    pub async fn run(&self) -> io::Result<()> {
        while let Ok((stream, addr)) = self.listener.accept().await {
            tokio::spawn(Self::handle_conection(
                self.peer_map.clone(),
                stream,
                addr,
                self.room.clone(),
                self.db.clone(),
            ));
        }

        Ok(())
    }

    async fn handle_conection(
        peer_map: PeerMap,
        raw_stream: TcpStream,
        addr: SocketAddr,
        room: Room,
        db: Arc<Mutex<DbRepo>>,
    ) -> Result<(), TtError> {
        let ws_stream = accept_async(raw_stream).await?;
        debug!("Connection established with {addr}");

        let (tx, rx) = unbounded();
        peer_map.lock().unwrap().insert(addr, tx);

        let (outgoing, incoming) = ws_stream.split();

        let mut room = room;
        let broadcast_incoming = incoming.try_for_each(|msg| {
            debug!("{} from {}", msg.to_text().unwrap(), addr);

            Self::handle_message(
                &Message::from(msg),
                peer_map.clone(),
                addr,
                &mut room,
                db.clone(),
            );

            future::ok(())
        });

        let receive_from_others = rx.map(Ok).forward(outgoing);

        pin_mut!(broadcast_incoming, receive_from_others);
        future::select(broadcast_incoming, receive_from_others).await;

        peer_map.lock().unwrap().remove(&addr);
        debug!("{} disconnected", &addr);

        Ok(())
    }

    fn send_to_all(msg: Message, peer_map: PeerMap, self_addr: SocketAddr) {
        let peers = peer_map.lock().unwrap();
        let broadcast_recipients = peers
            .iter()
            .filter(|(peer_addr, _)| peer_addr != &&self_addr)
            .map(|(_, ws_sink)| ws_sink);

        for recp in broadcast_recipients {
            recp.unbounded_send(msg.to_ttmessage()).unwrap();
        }
    }

    fn send_to_one(msg: Message, peer_map: PeerMap, addr: SocketAddr) {
        let peers = peer_map.lock().unwrap();
        let recp = peers.get(&addr).unwrap();
        recp.unbounded_send(msg.to_ttmessage()).unwrap();
    }

    fn handle_message(
        msg: &Message,
        peer_map: PeerMap,
        self_addr: SocketAddr,
        room: &mut Room,
        db: Arc<Mutex<DbRepo>>,
    ) {
        if let Some(passwd) = &room.passwd {
            if let Some(msg_passwd) = &msg.passwd {
                if msg_passwd != passwd {
                    Self::send_to_one(
                        Message {
                            msg_type: MessageType::Server(ServerMsg::AuthFailure),
                            passwd: None,
                        },
                        peer_map,
                        self_addr,
                    )
                }
            }
        }

        match msg.msg_type {
            MessageType::User(_) => todo!(),
            MessageType::UserReq(_) => todo!(),
            _ => (),
        }
    }
}
