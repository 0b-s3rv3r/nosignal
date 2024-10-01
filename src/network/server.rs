use super::{
    message::{Message, MessageType, ServerMsg, UserMsg, UserReqMsg},
    User,
};
use crate::{
    db::DbRepo,
    schema::{ServerRoom, TextMessage},
};
use bson::doc;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, StreamExt, TryStreamExt};
use log::{error, warn};
use std::{
    collections::HashMap,
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};
use tokio::{
    net::{TcpListener, TcpStream},
    time::sleep,
};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error as TtError, Message as TtMessage},
};
use tokio_util::sync::CancellationToken;

type Tx = UnboundedSender<TtMessage>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, (Tx, Option<User>)>>>;
type Db = Arc<Mutex<DbRepo>>;

pub struct ChatServer {
    pub(super) room: Arc<Mutex<ServerRoom>>,
    peer_map: PeerMap,
    db: Db,
    finisher: CancellationToken,
}

impl ChatServer {
    pub async fn new(room: ServerRoom, db: Db) -> io::Result<Self> {
        Ok(Self {
            peer_map: PeerMap::new(Mutex::new(HashMap::new())),
            room: Arc::new(Mutex::new(room)),
            db,
            finisher: CancellationToken::new(),
        })
    }

    pub async fn run(&mut self) -> io::Result<()> {
        let peer_map = self.peer_map.clone();
        let room = self.room.clone();
        let db = self.db.clone();
        let addr = self.room.lock().unwrap().addr;
        let cloned_token = self.finisher.clone();

        tokio::spawn(async move {
            let cloned_token = cloned_token.clone();
            let cloned_token_ = cloned_token.clone();

            let accepting_task = tokio::spawn(async move {
                let cloned_token_ = cloned_token_.clone();
                let listener_result = TcpListener::bind(&addr).await;
                if let Err(err) = &listener_result {
                    error!("{}", err);
                }
                let listener = listener_result.unwrap();

                while !cloned_token_.is_cancelled() {
                    tokio::select! {
                        accept_result = listener.accept() => {
                            match accept_result {
                                Ok((stream, addr)) => {
                                    let banned = room.clone().lock().unwrap().banned_addrs.iter().any(|&banned| banned != addr );

                                    if !banned {
                                        tokio::spawn(Self::handle_conection(
                                            peer_map.clone(),
                                            stream,
                                            addr,
                                            room.clone(),
                                            db.clone(),
                                            cloned_token_.clone(),
                                        ));
                                    }                                 }
                                Err(e) => {
                                    warn!("Failed to accept connection: {}", e);
                                }
                            }
                        }
                        _ = cloned_token_.cancelled() => {
                            break;
                        }
                    }
                }
            });

            pin_mut!(accepting_task);
            tokio::select! {
                _ = accepting_task => {},
                _ = cloned_token.cancelled() => {}
            };
        });

        Ok(())
    }

    pub async fn stop(&mut self) {
        Self::send_to_all(
            Message::from((ServerMsg::ServerShutdown, None)),
            self.peer_map.clone(),
            None,
        );
        sleep(Duration::from_millis(500)).await;
        self.finisher.cancel();
    }

    async fn handle_conection(
        peer_map: PeerMap,
        stream: TcpStream,
        addr: SocketAddr,
        room: Arc<Mutex<ServerRoom>>,
        db: Db,
        finisher: CancellationToken,
    ) -> Result<(), TtError> {
        let ws_stream = accept_async(stream).await?;

        let (tx, rx) = unbounded();
        peer_map.lock().unwrap().insert(addr, (tx, None));

        let (outgoing, incoming) = ws_stream.split();

        let room_id = room.lock().unwrap().id.clone();
        let passwd = room.lock().unwrap().passwd.clone();
        Self::send_to_one(
            Message::from((
                ServerMsg::Auth {
                    user_addr: addr,
                    room_id,
                    passwd,
                },
                None,
            )),
            peer_map.clone(),
            &addr,
        );

        let broadcast_incoming = incoming.try_for_each(|msg| {
            Self::handle_message(
                Message::from(msg),
                peer_map.clone(),
                addr,
                room.clone(),
                db.clone(),
            );

            if let Some(_) = room
                .lock()
                .unwrap()
                .banned_addrs
                .iter()
                .find(|&&a| a == addr)
            {
                return future::err(TtError::ConnectionClosed);
            }

            future::ok(())
        });

        let receive_from_others = rx.map(Ok).forward(outgoing);

        pin_mut!(broadcast_incoming, receive_from_others);
        tokio::select! {
            _ = broadcast_incoming => {},
            _ = receive_from_others => {},
            _ = finisher.cancelled() => return Ok(()),
        };

        peer_map.lock().unwrap().remove(&addr);

        Self::send_to_all(
            Message {
                msg_type: MessageType::Server(ServerMsg::UserLeft { addr }),
                passwd: None,
            },
            peer_map,
            None,
        );

        Ok(())
    }

    fn send_to_all(msg: Message, peer_map: PeerMap, except_addr: Option<&SocketAddr>) {
        let peers = peer_map.lock().unwrap();

        let broadcast_recipients = peers
            .iter()
            .filter(|(&peer_addr, _)| {
                if let Some(addr) = except_addr {
                    peer_addr != *addr
                } else {
                    true
                }
            })
            .map(|(_, ws_sink)| ws_sink);

        for (recp, _) in broadcast_recipients {
            if let Err(err) = recp.unbounded_send(msg.to_ttmessage()) {
                warn!("{}", err);
            }
        }
    }

    fn send_to_one(msg: Message, peer_map: PeerMap, addr: &SocketAddr) {
        let recp = {
            let peers = peer_map.lock().unwrap();
            peers.get(&addr).unwrap().0.clone()
        };
        if let Err(err) = recp.unbounded_send(msg.to_ttmessage()) {
            warn!("{}", err);
        }
    }

    fn handle_message(
        msg: Message,
        peer_map: PeerMap,
        addr: SocketAddr,
        room: Arc<Mutex<ServerRoom>>,
        db: Arc<Mutex<DbRepo>>,
    ) {
        if let Some(passwd) = &room.lock().unwrap().passwd {
            if let Some(msg_passwd) = &msg.passwd {
                if msg_passwd != passwd {
                    Self::send_to_one(
                        Message {
                            msg_type: MessageType::Server(ServerMsg::AuthFailure),
                            passwd: None,
                        },
                        peer_map.clone(),
                        &addr,
                    );
                    // peer_map.lock().unwrap().remove(&addr);
                    return;
                }
            } else {
                Self::send_to_one(
                    Message {
                        msg_type: MessageType::Server(ServerMsg::AuthFailure),
                        passwd: None,
                    },
                    peer_map.clone(),
                    &addr,
                );
                // peer_map.lock().unwrap().remove(&addr);
                return;
            }
        }

        match msg.msg_type {
            MessageType::User(user_msg) => match user_msg {
                UserMsg::Normal { msg: mut text_msg } => {
                    text_msg.timestamp = Some(SystemTime::now());
                    Self::send_to_all(
                        Message::from((
                            UserMsg::Normal {
                                msg: text_msg.clone(),
                            },
                            None,
                        )),
                        peer_map.clone(),
                        Some(&addr),
                    );
                    db.lock().unwrap().messages.insert_one(text_msg).unwrap();
                }
                UserMsg::UserJoined { user } => {
                    let mut updated_user = user;
                    updated_user.addr = Some(addr);
                    Self::send_to_all(
                        Message::from((
                            UserMsg::UserJoined {
                                user: updated_user.clone(),
                            },
                            room.lock().unwrap().passwd.clone(),
                        )),
                        peer_map.clone(),
                        None,
                    );
                    peer_map.lock().unwrap().get_mut(&addr).unwrap().1 = Some(updated_user);
                }
            },
            MessageType::UserReq(user_req) => match user_req {
                UserReqMsg::SyncReq => {
                    let messages = db
                        .lock()
                        .unwrap()
                        .messages
                        .find(doc! {"room_id": &room.lock().unwrap().id})
                        .unwrap()
                        .into_iter()
                        .map(|msg| msg.unwrap())
                        .collect::<Vec<TextMessage>>();

                    let users = if peer_map.lock().unwrap().is_empty() {
                        vec![]
                    } else {
                        peer_map
                            .clone()
                            .lock()
                            .unwrap()
                            .iter()
                            .filter_map(|(_, (_, user))| {
                                if user.is_some() {
                                    return Some(user.clone().unwrap());
                                }
                                None
                            })
                            .collect::<Vec<User>>()
                            .clone()
                    };

                    Self::send_to_one(
                        Message::from((
                            ServerMsg::Sync { messages, users },
                            room.lock().unwrap().passwd.clone(),
                        )),
                        peer_map.clone(),
                        &addr,
                    );
                }
                UserReqMsg::BanReq { addr: banned_addr } => {
                    Self::send_to_all(
                        Message::from((
                            ServerMsg::BanConfirm { addr: banned_addr },
                            room.lock().unwrap().passwd.clone(),
                        )),
                        peer_map.clone(),
                        None,
                    );

                    room.lock().unwrap().banned_addrs.push(banned_addr);
                    peer_map.lock().unwrap().remove(&banned_addr);
                    let result = db.lock().unwrap().server_rooms.update_one(
                        doc! {"id": room.lock().unwrap().id.clone()},
                        doc! {"$set": doc! {
                            "banned_addrs": room
                                .lock()
                                .unwrap()
                                .banned_addrs
                                .iter()
                                .map(|sa| sa.to_string())
                                .collect::<Vec<String>>()
                        }},
                    );
                    if let Err(err) = result {
                        error!("{}", err);
                    }
                }
            },
            _ => {}
        }
    }
}
