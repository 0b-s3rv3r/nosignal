use super::{Message, MessageType, ServerMsg, UserMsg, UserReqMsg};
use crate::{
    db::DbRepo,
    schema::{Room, TextMessage},
};
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use std::{
    collections::HashMap,
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::{
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error as TtError, Message as TtMessage},
};

type Tx = UnboundedSender<TtMessage>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

pub struct ChatServer {
    room: Arc<Mutex<Room>>,
    peer_map: PeerMap,
    event_loop_handle: Option<JoinHandle<()>>,
    db: Arc<Mutex<DbRepo>>,
}

impl ChatServer {
    pub async fn new(room: Room, db: Arc<Mutex<DbRepo>>) -> io::Result<Self> {
        Ok(Self {
            peer_map: PeerMap::new(Mutex::new(HashMap::new())),
            room: Arc::new(Mutex::new(room)),
            event_loop_handle: None,
            db,
        })
    }

    pub async fn run(&mut self) -> io::Result<()> {
        let peer_map = self.peer_map.clone();
        let room = self.room.clone();
        let db = self.db.clone();
        let addr = self.room.lock().unwrap().addr;

        let joinhandle = tokio::spawn(async move {
            let listener = TcpListener::bind(&addr).await.unwrap();

            while let Ok((stream, addr)) = listener.accept().await {
                tokio::spawn(Self::handle_conection(
                    peer_map.clone(),
                    stream,
                    addr,
                    room.clone(),
                    db.clone(),
                ));
            }
        });

        self.event_loop_handle = Some(joinhandle);

        Ok(())
    }

    pub fn stop(&self) {
        if let Some(joinhandle) = &self.event_loop_handle {
            joinhandle.abort();
        }
    }

    async fn handle_conection(
        peer_map: PeerMap,
        stream: TcpStream,
        addr: SocketAddr,
        room: Arc<Mutex<Room>>,
        db: Arc<Mutex<DbRepo>>,
    ) -> Result<(), TtError> {
        let ws_stream = accept_async(stream).await?;

        let (tx, rx) = unbounded();
        peer_map.lock().unwrap().insert(addr, tx);

        let (outgoing, incoming) = ws_stream.split();

        let broadcast_incoming = incoming.try_for_each(|msg| {
            Self::handle_message(
                Message::from(msg),
                peer_map.clone(),
                addr,
                room.clone(),
                db.clone(),
            );

            future::ok(())
        });

        let receive_from_others = rx.map(Ok).forward(outgoing);

        pin_mut!(broadcast_incoming, receive_from_others);
        future::select(broadcast_incoming, receive_from_others).await;

        peer_map.lock().unwrap().remove(&addr);

        Ok(())
    }

    fn send_to_all(msg: Message, peer_map: PeerMap, except_addr: Option<SocketAddr>) {
        let peers = peer_map.lock().unwrap();

        let broadcast_recipients = peers
            .iter()
            .filter(|(peer_addr, _)| {
                if let Some(addr) = except_addr {
                    peer_addr != &&addr
                } else {
                    true
                }
            })
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
        msg: Message,
        peer_map: PeerMap,
        addr: SocketAddr,
        room: Arc<Mutex<Room>>,
        db: Arc<Mutex<DbRepo>>,
    ) {
        let room = room.lock().unwrap();

        if let Some(passwd) = &room.passwd {
            if let Some(msg_passwd) = &msg.passwd {
                if msg_passwd != passwd {
                    Self::send_to_one(
                        Message {
                            msg_type: MessageType::Server(ServerMsg::AuthFailure),
                            passwd: None,
                        },
                        peer_map.clone(),
                        addr,
                    );
                }
            }
        }

        match &msg.msg_type {
            MessageType::User(user_msg) => match user_msg {
                UserMsg::Normal { msg: text_msg, .. } => {
                    Self::send_to_all(msg.clone(), peer_map.clone(), Some(addr));
                    db.lock().unwrap().messages.insert_one(text_msg).unwrap();
                }
                UserMsg::UserJoined { user } => {
                    let mut updated_user = user.clone();
                    updated_user.addr = Some(addr);
                    Self::send_to_all(
                        Message {
                            msg_type: MessageType::User(UserMsg::UserJoined { user: updated_user }),
                            passwd: room.passwd.clone(),
                        },
                        peer_map,
                        None,
                    );
                }
            },
            MessageType::UserReq(user_req) => match user_req {
                UserReqMsg::FetchMessagesReq => {
                    let messages = db
                        .lock()
                        .unwrap()
                        .messages
                        .find(None)
                        .unwrap()
                        .into_iter()
                        .map(|msg| msg.unwrap())
                        .collect::<Vec<TextMessage>>();

                    Self::send_to_one(
                        Message {
                            msg_type: MessageType::Server(ServerMsg::MessagesFetch { messages }),
                            passwd: None,
                        },
                        peer_map,
                        addr,
                    );
                }
                UserReqMsg::BanReq { addr } => todo!(),
            },
            _ => (),
        }
    }
}
