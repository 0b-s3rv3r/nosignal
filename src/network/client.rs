use super::{Message, MessageType, User, UserMsg};
use crate::schema::Room;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::error::Error;
use tokio_tungstenite::tungstenite::protocol::Message as ttMessage;

#[derive(Debug)]
pub struct ChatClient {
    sender: Sender<ttMessage>,
    receiver: Receiver<ttMessage>,
    pub user: User,
    pub room: Room,
    pub users: Vec<User>,
}

impl ChatClient {
    pub async fn connect(room: Room, user: &User) -> Result<Self, Error> {
        let (ws_stream, _) = connect_async(format!("ws://{}/", room.addr)).await?;
        let (write, read) = ws_stream.split();

        let (tx, mut rx) = mpsc::channel::<ttMessage>(100);
        let (tx_in, rx_in) = mpsc::channel::<ttMessage>(100);

        tx.send(
            Message {
                msg_type: MessageType::User(UserMsg::UserJoined { user: user.clone() }),
                passwd: room.passwd.clone(),
            }
            .to_ttmessage(),
        )
        .await
        .unwrap();

        tx.send(
            Message {
                msg_type: MessageType::User(UserMsg::FetchMessagesReq),
                passwd: room.passwd.clone(),
            }
            .to_ttmessage(),
        )
        .await
        .unwrap();

        // tx.send(Message::text(
        //     MessageType::UserJoined {
        //         user: user.clone(),
        //         passwd: room.passwd.clone(),
        //     }
        //     .to_string(),
        // ))
        // .await
        // .unwrap();

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
        })
    }

    pub async fn send_msg(&self, msg: &Message) -> Result<(), mpsc::error::SendError<ttMessage>> {
        self.sender.send(msg.to_ttmessage()).await
    }

    pub async fn recv_msg(&mut self) -> Option<Message> {
        if self.receiver.is_empty() {
            return None;
        }

        let msg = Message::from(self.receiver.recv().await.unwrap());

        match msg.msg_type {
            MessageType::User(user_msg) => match user_msg {
                super::UserMsg::Normal { msg } => todo!(),
                super::UserMsg::Ban { addr } => todo!(),
                super::UserMsg::UserJoined { user } => todo!(),
                super::UserMsg::FetchMessagesReq => todo!(),
            },
            MessageType::Server(server_msg) => match server_msg {
                super::ServerMsg::AuthFailure => todo!(),
                super::ServerMsg::MessagesFetch { messages } => todo!(),
                super::ServerMsg::UserLeft { addr } => todo!(),
            },
        }

        Some(msg)
    }
}
