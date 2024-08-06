use super::{
    message::{Message, MessageType, UserMsg, UserReqMsg},
    User,
};
use crate::schema::Room;
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
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
    event_loop_handle: Option<JoinHandle<()>>,
    transceiver: Option<Sender<TtMessage>>,
    in_receiver: Option<Receiver<TtMessage>>,
}

impl ChatClient {
    pub fn new(room: Room, user: User) -> Self {
        Self {
            room: Arc::new(Mutex::new(room)),
            user,
            event_loop_handle: None,
            transceiver: None,
            in_receiver: None,
        }
    }

    pub async fn connect(&mut self) -> Result<(), TtError> {
        let (ws_stream, _) =
            connect_async(format!("ws://{}/", self.room.lock().unwrap().addr)).await?;
        let (write, read) = ws_stream.split();

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

            return Some(Message::from(receiver.recv().await.unwrap()).msg_type);
        }
        None
    }

    pub async fn sync(&self) -> Result<(), SendError<TtMessage>> {
        if let Some(transceiver) = &self.transceiver {
            transceiver
                .send(
                    Message::from((
                        UserReqMsg::SyncReq,
                        self.room.lock().unwrap().passwd.clone(),
                    ))
                    .to_ttmessage(),
                )
                .await?
        }
        Ok(())
    }
}
