pub mod client;
pub mod message;
pub mod server;

use crate::schema::Color;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub addr: Option<SocketAddr>,
    pub color: Color,
}

#[cfg(test)]
mod test {
    use crate::{
        db::DbRepo,
        network::{
            client::ChatClient,
            message::MessageType,
            message::{Message, ServerMsg, UserMsg, UserReqMsg},
            server::ChatServer,
            User,
        },
        schema::{Color, ServerRoom, TextMessage},
        util::hash_passwd,
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
    async fn messages_sending() {
        let passwd = String::from("password");
        hash_passwd(&passwd);

        let room = ServerRoom {
            id: "firstroom".into(),
            addr: SocketAddr::from_str("127.0.0.1:12345").unwrap(),
            passwd: Some(passwd),
            banned_addrs: vec![],
        };
        let header = room.room_header();

        let mut user = User {
            id: "user1".into(),
            addr: None,
            color: Color::LightRed,
        };
        let mut user2 = User {
            id: "user2".into(),
            addr: None,
            color: Color::LightGreen,
        };

        let db = Arc::new(Mutex::new(DbRepo::memory_init().unwrap()));
        db.lock()
            .unwrap()
            .server_rooms
            .insert_one(&room.clone())
            .unwrap();
        let mut peermap = HashMap::<SocketAddr, User>::new();

        let mut server = ChatServer::new(room.clone(), db.clone()).await.unwrap();
        server.run().await.unwrap();
        sleep(Duration::from_secs(1)).await;

        let mut client = ChatClient::new(header.clone(), user.clone());
        client.connect().await.unwrap();
        sleep(Duration::from_secs(1)).await;

        if let MessageType::Server(ServerMsg::Auth { user_addr, .. }) =
            client.recv_msg().await.unwrap()
        {
            user.addr = Some(user_addr);
        } else {
            assert!(false);
        }
        if let MessageType::Server(ServerMsg::Sync { .. }) = client.recv_msg().await.unwrap() {
            assert!(true);
        } else {
            assert!(false);
        }
        if let MessageType::User(UserMsg::UserJoined { user: user_ }) =
            client.recv_msg().await.unwrap()
        {
            peermap.insert(user.addr.unwrap(), user.clone());
            assert_eq!(user, user_)
        } else {
            assert!(false);
        }

        let sended_msg = TextMessage::new(&user.addr.unwrap(), &header.id, "some short message");
        let mut sended_msg2 = sended_msg.clone();
        client
            .send_msg(Message::from((
                UserMsg::Normal {
                    msg: sended_msg.clone(),
                },
                Some(header.passwd.clone().unwrap()),
            )))
            .await
            .unwrap();

        let mut client2 = ChatClient::new(header.clone(), user2.clone());
        client2.connect().await.unwrap();
        sleep(Duration::from_millis(1)).await;

        if let MessageType::Server(ServerMsg::Auth { user_addr, .. }) =
            client2.recv_msg().await.unwrap()
        {
            user2.addr = Some(user_addr);
        } else {
            assert!(false);
        }

        if let MessageType::Server(ServerMsg::Sync {
            messages, users, ..
        }) = client2.recv_msg().await.unwrap()
        {
            assert_eq!(messages[0].room_id, sended_msg.room_id);
            assert_eq!(messages[0].sender_addr, sended_msg.sender_addr);
            assert_eq!(messages[0].content, sended_msg.content);
            assert_eq!(users[0], peermap.get(&user.addr.unwrap()).unwrap().clone());
        } else {
            assert!(false);
        }
        if let MessageType::User(UserMsg::UserJoined { user: user_ }) =
            client2.recv_msg().await.unwrap()
        {
            user2.addr = user_.addr;
            peermap.insert(user.addr.unwrap(), user.clone());
        } else {
            assert!(false);
        }
        if let MessageType::User(UserMsg::UserJoined { .. }) = client.recv_msg().await.unwrap() {
            assert!(true);
        } else {
            assert!(false);
        }

        sended_msg2.sender_addr = user2.addr.unwrap();
        client2
            .send_msg(Message::from((
                UserMsg::Normal {
                    msg: sended_msg2.clone(),
                },
                header.passwd.clone(),
            )))
            .await
            .unwrap();
        sleep(Duration::from_secs(1)).await;
        let received_msg = client.recv_msg().await.unwrap();
        if let MessageType::User(UserMsg::Normal { msg }) = received_msg {
            assert_eq!(msg.room_id, sended_msg2.room_id);
            assert_eq!(msg.sender_addr, sended_msg2.sender_addr);
            assert_eq!(msg.content, sended_msg2.content);
        }

        client2
            .send_msg(Message::from((
                UserMsg::Normal {
                    msg: sended_msg2.clone(),
                },
                None,
            )))
            .await
            .unwrap();
        sleep(Duration::from_millis(1)).await;
        assert_eq!(
            client2.recv_msg().await.unwrap(),
            MessageType::Server(ServerMsg::AuthFailure)
        );

        client
            .send_msg(Message::from((
                UserReqMsg::BanReq {
                    addr: user2.addr.unwrap(),
                },
                header.passwd,
            )))
            .await
            .unwrap();
        sleep(Duration::from_secs(1)).await;
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

        server.stop().await;
        assert_eq!(
            client.recv_msg().await.unwrap(),
            MessageType::Server(ServerMsg::ServerShutdown)
        );

        client.disconnect();
        client2.disconnect();

        assert_eq!(
            server.room.lock().unwrap().banned_addrs,
            db.lock()
                .unwrap()
                .server_rooms
                .find_one(None)
                .unwrap()
                .unwrap()
                .banned_addrs
        );
    }
}
