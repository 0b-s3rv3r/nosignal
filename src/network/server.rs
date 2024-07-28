use super::ChatMessage;
use crate::{schema::Room, util::hash_passwd};
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
    room: Room,
}

impl ChatServer {
    pub async fn new(room: Room) -> io::Result<Self> {
        let listener = TcpListener::bind(&room.addr).await?;
        debug!("Listening on {}", room.addr);

        Ok(Self {
            peer_map: PeerMap::new(Mutex::new(HashMap::new())),
            listener,
            room,
        })
    }

    pub async fn run(&self) -> io::Result<()> {
        while let Ok((stream, addr)) = self.listener.accept().await {
            tokio::spawn(Self::handle_conection(
                self.peer_map.clone(),
                stream,
                addr,
                self.room.clone(),
            ));
        }

        Ok(())
    }

    async fn handle_conection(
        peer_map: PeerMap,
        raw_stream: TcpStream,
        addr: SocketAddr,
        room: Room,
    ) -> Result<(), TtError> {
        let ws_stream = accept_async(raw_stream).await?;
        debug!("Connection established with {addr}");

        let (tx, rx) = unbounded();
        peer_map.lock().unwrap().insert(addr, tx);

        let (outgoing, incoming) = ws_stream.split();

        let mut room = room;
        let broadcast_incoming = incoming.try_for_each(|msg| {
            debug!("{} from {}", msg.to_text().unwrap(), addr);

            Self::handle_message(&ChatMessage::from(msg), peer_map.clone(), addr, &mut room);

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
            recp.unbounded_send(msg.clone()).unwrap();
            debug!("{msg}");
        }
    }

    fn send_to_one(msg: Message, peer_map: PeerMap, addr: SocketAddr) {
        let peers = peer_map.lock().unwrap();
        let recp = peers.get(&addr).unwrap();
        recp.unbounded_send(msg.clone()).unwrap();
    }

    fn handle_message(
        msg: &ChatMessage,
        peer_map: PeerMap,
        self_addr: SocketAddr,
        room: &mut Room,
    ) {
        match msg {
            ChatMessage::Normal { msg: _, passwd } => {
                if let Some(passwd_) = passwd {
                    if !Self::check_password(passwd_, room) {
                        Self::send_to_one(
                            Message::Text(ChatMessage::AuthFailure_.to_string()),
                            peer_map,
                            self_addr,
                        );
                        return;
                    }
                }

                Self::send_to_all(Message::Text(msg.to_string()), peer_map, self_addr);
            }
            ChatMessage::Ban { addr, passwd } => {
                if let Some(passwd_) = passwd {
                    if !Self::check_password(passwd_, room) {
                        Self::send_to_one(
                            Message::Text(ChatMessage::AuthFailure_.to_string()),
                            peer_map,
                            self_addr,
                        );
                        return ();
                    }
                }
                room.banned_addrs.push(*addr);
                peer_map.lock().unwrap().remove(&addr);
                Self::send_to_all(Message::Text(msg.to_string()), peer_map, self_addr);
            }
            ChatMessage::UserJoined { user: _, passwd } => {
                if let Some(passwd_) = passwd {
                    if !Self::check_password(passwd_, room) {
                        Self::send_to_one(
                            Message::Text(ChatMessage::AuthFailure_.to_string()),
                            peer_map,
                            self_addr,
                        );
                        return;
                    }
                }

                Self::send_to_all(Message::Text(msg.to_string()), peer_map, self_addr);
            }
            ChatMessage::UserLeft { .. } => {
                Self::send_to_all(Message::Text(msg.to_string()), peer_map, self_addr)
            }
            ChatMessage::ServerShutdown => {
                Self::send_to_all(Message::Text(msg.to_string()), peer_map, self_addr);
            }
            ChatMessage::FetchMessagesReq { passwd } => {
                if let Some(passwd_) = passwd {
                    if !Self::check_password(passwd_, room) {
                        Self::send_to_one(
                            Message::Text(ChatMessage::AuthFailure_.to_string()),
                            peer_map,
                            self_addr,
                        );
                        return;
                    }
                }
                todo!("add fetching from db")
            }
            _ => (),
        }
    }

    fn check_password(passwd: &str, room: &Room) -> bool {
        if let Some(room_pass) = &room.passwd {
            return hash_passwd(passwd) == *room_pass;
        }

        true
    }
}
