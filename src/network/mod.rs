pub mod client;
pub mod message;
pub mod server;

use crate::schema::Color;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub _id: String,
    pub addr: Option<SocketAddr>,
    pub color: Color,
}

#[cfg(test)]
mod test {
    use super::message::MessageType;
    use crate::{
        db::DbRepo,
        network::message::{Message, ServerMsg, UserMsg, UserReqMsg},
        schema::{Color, Room, TextMessage},
        util::hash_passwd,
        ChatClient, ChatServer, User,
    };
    use std::{
        collections::HashMap,
        net::SocketAddr,
        str::FromStr,
        sync::{Arc, Mutex},
        time::Duration,
    };
    use tokio::time::sleep;

    #[tokio::test]
    async fn messages_are_correct() {
        let passwd = String::from("password");
        hash_passwd(&passwd);
        let room = Room {
            _id: "firstroom".into(),
            addr: SocketAddr::from_str("127.0.0.1:12345").unwrap(),
            passwd: Some(passwd),
            banned_addrs: vec![],
            is_owner: true,
        };

        let mut room2 = room.clone();
        room2.is_owner = false;

        let mut user = User {
            _id: "user1".into(),
            addr: None,
            color: Color::LightRed,
        };

        let mut user2 = User {
            _id: "user2".into(),
            addr: None,
            color: Color::LightGreen,
        };

        let room_ = room.clone();

        let db = Arc::new(Mutex::new(DbRepo::memory_init().unwrap()));

        let mut peermap = HashMap::<SocketAddr, User>::new();

        let mut server = ChatServer::new(room.clone(), db).await.unwrap();
        server.run().await.unwrap();
        sleep(Duration::from_secs(1)).await;
        let mut client = ChatClient::new(room_, user.clone());
        client.connect().await.unwrap();
        sleep(Duration::from_secs(1)).await;
        server.stop();

        if let MessageType::User(UserMsg::UserJoined { user: user_ }) =
            client.recv_msg().await.unwrap()
        {
            user.addr = user_.addr;
            peermap.insert(user.addr.unwrap(), user.clone()).unwrap();
        }

        let sended_msg = TextMessage::new(&user.addr.unwrap(), &room._id, "some short message");

        client
            .send_msg(Message::from((
                UserMsg::Normal {
                    msg: sended_msg.clone(),
                },
                Some(room.passwd.clone().unwrap()),
            )))
            .await
            .unwrap();

        let mut client2 = ChatClient::new(room2.clone(), user2.clone());
        client2.connect().await.unwrap();

        if let MessageType::User(UserMsg::UserJoined { user: user_ }) =
            client2.recv_msg().await.unwrap()
        {
            user2.addr = user_.addr;
            peermap.insert(user.addr.unwrap(), user.clone()).unwrap();
        }

        client2.sync().await.unwrap();

        if let MessageType::Server(ServerMsg::Sync { messages, users }) =
            client2.recv_msg().await.unwrap()
        {
            assert_eq!(messages[0], sended_msg);
            assert_eq!(users[0], peermap.get(&user.addr.unwrap()).unwrap().clone());
        }

        client2
            .send_msg(Message::from((
                UserMsg::Normal {
                    msg: sended_msg.clone(),
                },
                room2.passwd.clone(),
            )))
            .await
            .unwrap();

        if let MessageType::User(UserMsg::Normal { msg }) = client.recv_msg().await.unwrap() {
            assert_eq!(msg, sended_msg);
        }

        client2
            .send_msg(Message::from((
                UserMsg::Normal {
                    msg: sended_msg.clone(),
                },
                None,
            )))
            .await
            .unwrap();

        assert_eq!(
            client.recv_msg().await.unwrap(),
            MessageType::Server(ServerMsg::AuthFailure)
        );

        client
            .send_msg(Message::from((
                UserReqMsg::BanReq {
                    addr: user2.addr.unwrap(),
                },
                room.passwd,
            )))
            .await
            .unwrap();

        assert_eq!(
            client.recv_msg().await.unwrap(),
            MessageType::Server(ServerMsg::BanConfirm {
                addr: user2.addr.unwrap()
            })
        );

        assert_eq!(
            client.recv_msg().await.unwrap(),
            MessageType::Server(ServerMsg::UserLeft {
                addr: user2.addr.unwrap(),
            })
        );

        server.stop();

        assert_eq!(
            client.recv_msg().await.unwrap(),
            MessageType::Server(ServerMsg::ServerShutdown)
        );

        client.close_connection();
    }
}
