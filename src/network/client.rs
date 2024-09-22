use super::{
    message::{Message, MessageType, ServerMsg, UserMsg, UserReqMsg},
    User,
};
use crate::schema::Room;
use futures_util::{SinkExt, StreamExt};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, RwLock},
};
use tokio::sync::mpsc::{self, error::SendError, Receiver, Sender};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as TtError, Message as TtMessage},
};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct ChatClient {
    pub room: Arc<Mutex<Room>>,
    pub user: User,
    transceiver: Option<Sender<TtMessage>>,
    in_receiver: Option<Receiver<TtMessage>>,
    finisher: CancellationToken,
    pub is_ok: Arc<RwLock<bool>>,
}

impl ChatClient {
    pub fn new(room: Room, user: User) -> Self {
        Self {
            room: Arc::new(Mutex::new(room)),
            user,
            transceiver: None,
            in_receiver: None,
            finisher: CancellationToken::new(),
            is_ok: Arc::new(RwLock::new(true)),
        }
    }

    pub async fn connect(&mut self) -> Result<(), TtError> {
        let (ws_stream, _) =
            connect_async(format!("ws://{}/", self.room.lock().unwrap().addr)).await?;
        let (mut write, mut read) = ws_stream.split();

        let (tx, mut rx) = mpsc::channel::<TtMessage>(100);
        let (tx_in, rx_in) = mpsc::channel::<TtMessage>(100);

        tx.send(
            Message::from((
                UserMsg::UserJoined {
                    user: self.user.clone(),
                },
                self.room.lock().unwrap().passwd.clone(),
            ))
            .to_ttmessage(),
        )
        .await
        .unwrap();

        self.transceiver = Some(tx);
        self.in_receiver = Some(rx_in);

        let rcancel_token = self.finisher.child_token();
        let wcancel_token = self.finisher.child_token();
        let rcloned_is_ok = self.is_ok.clone();
        let wcloned_is_ok = self.is_ok.clone();

        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if rcancel_token.is_cancelled() {
                    break;
                }
                match msg {
                    Ok(msg) => {
                        // if let MessageType::Server(ServerMsg::ConnectionRefused) =
                        //     Message::from(msg.clone()).msg_type
                        // {
                        //     *rcloned_is_ok.write().unwrap() = false;
                        // }
                        if tx_in.send(msg).await.is_err() {
                            *rcloned_is_ok.write().unwrap() = false;
                            eprintln!("Receiver dropped");
                            return;
                        }
                    }
                    Err(e) => {
                        *rcloned_is_ok.write().unwrap() = false;
                        eprintln!("Error reading message: {}", e);
                        return;
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
                    *wcloned_is_ok.write().unwrap() = false;
                    eprintln!("Error sending message: {}", e);
                    return;
                }
            }
        });

        Ok(())
    }

    pub fn close_connection(&mut self) {
        self.finisher.cancel();
    }

    pub fn is_ok(&self) -> bool {
        *self.is_ok.read().unwrap()
    }

    pub async fn send_msg(&self, msg: Message) -> Result<(), SendError<TtMessage>> {
        if !self.is_ok() {
            return Ok(());
        }
        if let Some(transceiver) = &self.transceiver {
            transceiver.send(msg.to_ttmessage()).await?
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

            return Some(Message::from(receiver.recv().await.unwrap()).msg_type);
        }
        None
    }

    pub async fn sync(&self) -> Result<(), SendError<TtMessage>> {
        Ok(self
            .send_msg(Message::from((
                UserReqMsg::SyncReq,
                self.room.lock().unwrap().passwd.clone(),
            )))
            .await?)
    }

    pub async fn ban(&self, addr: &SocketAddr) -> Result<(), SendError<TtMessage>> {
        Ok(self
            .send_msg(Message::from((
                UserReqMsg::BanReq { addr: addr.clone() },
                self.room.lock().unwrap().passwd.clone(),
            )))
            .await?)
    }
}
