use super::{
    message::{Message, MessageType, UserMsg, UserReqMsg},
    User,
};
use crate::schema::Room;
use futures_util::{SinkExt, StreamExt};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::{
    sync::mpsc::{self, error::SendError, Receiver, Sender},
    task::JoinHandle,
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as TtError, Message as TtMessage},
};

#[derive(Debug)]
pub struct ChatClient {
    pub room: Arc<Mutex<Room>>,
    pub user: User,
    transceiver: Option<Sender<TtMessage>>,
    in_receiver: Option<Receiver<TtMessage>>,
    event_loop_handlers: Option<(JoinHandle<()>, JoinHandle<()>)>,
}

impl ChatClient {
    pub fn new(room: Room, user: User) -> Self {
        Self {
            room: Arc::new(Mutex::new(room)),
            user,
            transceiver: None,
            in_receiver: None,
            event_loop_handlers: None,
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

        let read_handler = tokio::spawn(async move {
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

        let write_handler = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = write.send(msg).await {
                    eprintln!("Error sending message: {}", e);
                    return;
                }
            }
        });

        self.event_loop_handlers = Some((read_handler, write_handler));

        Ok(())
    }

    pub fn close_connection(&mut self) {
        if let Some(ref handlers) = self.event_loop_handlers {
            handlers.0.abort();
            handlers.1.abort();
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
