use super::{ChatMessage, User};
use crate::schema::Room;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::error::Error;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Debug)]
pub struct ChatClient {
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    pub user: User,
    pub room: Room,
    pub users: Vec<User>,
    pub owner: bool,
}

impl ChatClient {
    pub async fn connect(room: Room, user: &User, as_owner: bool) -> Result<Self, Error> {
        let (ws_stream, _) = connect_async(format!("ws://{}/", room.addr)).await?;
        let (write, read) = ws_stream.split();

        let (tx, mut rx) = mpsc::channel::<Message>(100);
        let (tx_in, rx_in) = mpsc::channel::<Message>(100);

        tx.send(Message::text(
            ChatMessage::UserJoined {
                user: user.clone(),
                passwd: room.passwd.clone(),
            }
            .to_string(),
        ))
        .await
        .unwrap();

        task::spawn(async move {
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

            // tx.send(Message::text(
            //     ChatMessage::UserLeft {
            //         user_id: user.id.clone(),
            //     }
            //     .to_string(),
            // ));
        });

        task::spawn(async move {
            let mut write = write;
            while let Some(msg) = rx.recv().await {
                if let Err(e) = write.send(msg).await {
                    eprintln!("Error sending message: {}", e);
                    return;
                }
            }
        });

        Ok(ChatClient {
            sender: tx,
            receiver: rx_in,
            user: user.clone(),
            room,
            users: vec![user.clone()],
            owner: as_owner,
        })
    }

    pub async fn send(&self, msg: &ChatMessage) -> Result<(), mpsc::error::SendError<Message>> {
        self.sender.send(Message::Text(msg.to_string())).await
    }

    pub async fn receive(&mut self) -> Option<ChatMessage> {
        if self.receiver.is_empty() {
            return None;
        }

        let msg = ChatMessage::from(self.receiver.recv().await.unwrap());
        match msg {
            ChatMessage::Ban { addr, .. } => self.users.retain(|r| r.addr.unwrap() != addr),
            ChatMessage::UserJoined { ref user, .. } => self.users.push(user.clone()),
            ChatMessage::UserLeft { ref user_id } => self.users.retain(|r| r.id != *user_id),
            _ => (),
        }

        Some(msg)
    }

    pub async fn messages_request(&self) {
        self.sender
            .send(Message::text(
                ChatMessage::FetchMessagesReq {
                    passwd: self.room.passwd.clone(),
                }
                .to_string(),
            ))
            .await
            .unwrap();
    }
}
