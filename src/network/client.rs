use super::{Message, MessageType, ServerMsg, User, UserMsg, UserReqMsg};
use crate::schema::Room;
use futures_util::{SinkExt, StreamExt};
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as TtError, Message as TtMessage},
};

#[derive(Debug)]
pub struct ChatClient {
    event_loop_handle: Option<JoinHandle<()>>,
    transceiver: Option<Sender<TtMessage>>,
    in_receiver: Option<Receiver<TtMessage>>,
    pub user: User,
    pub room: Room,
    pub users: HashMap<SocketAddr, User>,
}

impl ChatClient {
    pub fn new(room: Room, user: User) -> Self {
        Self {
            event_loop_handle: None,
            transceiver: None,
            in_receiver: None,
            user,
            room,
            users: HashMap::new(),
        }
    }

    pub async fn connect(&mut self) -> Result<(), TtError> {
        let (ws_stream, _) = connect_async(format!("ws://{}/", self.room.addr)).await?;
        let (write, read) = ws_stream.split();

        let (tx, mut rx) = mpsc::channel::<TtMessage>(100);
        let (tx_in, rx_in) = mpsc::channel::<TtMessage>(100);

        tx.send(
            Message {
                msg_type: MessageType::User(UserMsg::UserJoined {
                    user: self.user.clone(),
                }),
                passwd: self.room.passwd.clone(),
            }
            .to_ttmessage(),
        )
        .await
        .unwrap();

        tx.send(
            Message {
                msg_type: MessageType::UserReq(UserReqMsg::FetchMessagesReq),
                passwd: self.room.passwd.clone(),
            }
            .to_ttmessage(),
        )
        .await
        .unwrap();

        self.transceiver = Some(tx);
        self.in_receiver = Some(rx_in);

        let joinhandle = tokio::spawn(async move {
            tokio::spawn(async move {
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

            tokio::spawn(async move {
                let mut write = write;
                while let Some(msg) = rx.recv().await {
                    if let Err(e) = write.send(msg).await {
                        eprintln!("Error sending message: {}", e);
                        return;
                    }
                }
            });
        });

        self.event_loop_handle = Some(joinhandle);

        Ok(())
    }

    pub fn close_connection(&mut self) {
        self.users.retain(|_, user| *user == self.user);

        if let Some(handle) = &self.event_loop_handle {
            handle.abort();
        }
    }

    pub async fn send_msg(&self, msg: Message) -> Result<(), SendError<TtMessage>> {
        if let Some(transceiver) = &self.transceiver {
            transceiver.send(msg.to_ttmessage()).await?
        }
        Ok(())
    }

    pub async fn recv_msg(&mut self) -> Option<MessageType> {
        if let Some(ref mut receiver) = self.in_receiver {
            if receiver.is_empty() {
                return None;
            }

            let msg = Message::from(receiver.recv().await.unwrap()).msg_type;

            match msg {
                MessageType::User(ref user_msg) => match user_msg {
                    UserMsg::UserJoined { user } => {
                        if let Some(addr) = user.addr {
                            if user._id == self.user._id {
                                self.user.addr = Some(addr);
                            }
                            self.users.insert(addr, user.clone());
                        }
                    }
                    _ => (),
                },
                MessageType::Server(ref server_msg) => match server_msg {
                    ServerMsg::UserLeft { addr } => {
                        self.users.remove(addr);
                    }
                    ServerMsg::BanConfirm { addr } => {
                        self.users.remove(addr);
                    }
                    _ => (),
                },
                _ => (),
            }

            return Some(msg);
        }
        None
    }
}
