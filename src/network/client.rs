use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::error::Error;
use tokio_tungstenite::tungstenite::protocol::Message;

pub struct ChatClient {
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

impl ChatClient {
    pub async fn connect(url: &str) -> Result<Self, Error> {
        let (ws_stream, _) = connect_async(url).await?;
        let (write, read) = ws_stream.split();

        let (tx, mut rx) = mpsc::channel::<Message>(100);
        let (tx_in, rx_in) = mpsc::channel::<Message>(100);

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
        })
    }

    pub async fn send(&self, msg: Message) -> Result<(), mpsc::error::SendError<Message>> {
        self.sender.send(msg).await
    }

    pub async fn receive(&mut self) -> Option<Message> {
        if self.receiver.is_empty() {
            return None;
        }
        self.receiver.recv().await
    }
}
