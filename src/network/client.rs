use super::{
    message::{Message, MessageType, ServerMsg, UserMsg},
    User,
};
use crate::schema::RoomHeader;
use futures_util::{SinkExt, StreamExt};
use log::{error, warn};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::{self, error::SendError, Receiver, Sender};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as TtError, Message as TtMessage},
};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct ChatClient {
    pub room: Arc<Mutex<RoomHeader>>,
    pub user: Arc<Mutex<User>>,
    transceiver: Option<Sender<TtMessage>>,
    in_receiver: Option<Receiver<Message>>,
    finisher: CancellationToken,
}

impl ChatClient {
    pub fn new(room: RoomHeader, user: User) -> Self {
        Self {
            room: Arc::new(Mutex::new(room)),
            user: Arc::new(Mutex::new(user)),
            transceiver: None,
            in_receiver: None,
            finisher: CancellationToken::new(),
        }
    }

    pub async fn connect(&mut self) -> Result<(), TtError> {
        let (ws_stream, _) =
            connect_async(format!("ws://{}/", self.room.lock().unwrap().addr)).await?;
        let (mut write, mut read) = ws_stream.split();

        let (tx, mut rx) = mpsc::channel::<TtMessage>(100);
        let (tx_in, rx_in) = mpsc::channel::<Message>(100);

        self.transceiver = Some(tx);
        self.in_receiver = Some(rx_in);

        let rcancel_token = self.finisher.child_token();
        let wcancel_token = self.finisher.child_token();

        let shared_room = self.room.clone();
        let shared_user = self.user.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if rcancel_token.is_cancelled() {
                    break;
                }
                match msg {
                    Ok(msg) => {
                        let deserialize_result = Message::try_from(msg);
                        if let Ok(deserialized_msg) = &deserialize_result {
                            if let MessageType::Server(ServerMsg::Sync {
                                room_id, user_addr, ..
                            }) = &deserialized_msg.msg_type
                            {
                                shared_room.lock().unwrap()._id = room_id.clone();
                                shared_user.lock().unwrap().addr = Some(*user_addr);
                            }

                            if let Err(err) = tx_in.send(deserialized_msg.clone()).await {
                                rcancel_token.cancel();
                                error!("Receiver dropped: {}", err);
                            }
                        } else {
                            warn!("Failed to desrialize from TtMessage in client");
                        }
                    }
                    Err(e) => {
                        rcancel_token.cancel();
                        warn!("Error reading message {}", e);
                    }
                }
            }
        });

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if wcancel_token.is_cancelled() {
                    break;
                }
                if let Err(e) = write.send(msg).await {
                    match e {
                        TtError::ConnectionClosed | TtError::AlreadyClosed => {
                            wcancel_token.cancel();
                            warn!("Unable to connect with server");
                        }
                        _ => {
                            wcancel_token.cancel();
                            warn!("Error sending message: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.finisher.cancel();
    }

    pub fn is_ok(&self) -> bool {
        !self.finisher.is_cancelled()
    }

    pub fn set_passwd(&mut self, passwd: &str) {
        self.room.lock().unwrap().passwd = Some(passwd.to_string());
    }

    pub async fn send_msg(&self, msg: impl Into<MessageType>) -> Result<(), SendError<TtMessage>> {
        if let Some(transceiver) = &self.transceiver {
            transceiver
                .send(
                    Message::from((msg.into(), self.room.lock().unwrap().passwd.clone()))
                        .to_ttmessage(),
                )
                .await?
        }
        Ok(())
    }

    pub async fn recv_msg(&mut self) -> Option<MessageType> {
        if !self.is_ok() {
            return None;
        }
        if let Some(ref mut receiver) = self.in_receiver {
            if receiver.is_empty() {
                return None;
            }

            let msg_type = receiver.recv().await?.msg_type;
            if let MessageType::Server(server_msg) = &msg_type {
                match server_msg {
                    ServerMsg::AuthFailure => {
                        self.disconnect();
                    }
                    ServerMsg::BanConfirm { addr } => {
                        if *addr == self.user.lock().unwrap().addr.unwrap() {
                            self.disconnect();
                        }
                    }
                    ServerMsg::ServerShutdown => {
                        self.disconnect();
                    }
                    _ => {}
                }
            }

            return Some(msg_type);
        }
        None
    }

    pub async fn ban(&self, addr: &SocketAddr) -> Result<(), SendError<TtMessage>> {
        self.send_msg(UserMsg::BanReq { addr: *addr }).await
    }
}
