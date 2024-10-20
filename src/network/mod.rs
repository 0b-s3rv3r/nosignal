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
            message::{ServerMsg, UserMsg},
            server::ChatServer,
            User,
        },
        schema::{Color, ServerRoom, TextMessage},
        util::hash_passwd,
    };
    use bson::doc;
    use polodb_core::CollectionT;
    use std::{
        net::SocketAddr,
        path::Path,
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
            _id: "firstroom".into(),
            addr: SocketAddr::from_str("127.0.0.1:12345").unwrap(),
            passwd: Some(passwd),
            banned_addrs: vec![],
        };
        let header = room.room_header();

        let user = User {
            id: "user1".into(),
            addr: None,
            color: Color::LightRed,
        };
        let user2 = User {
            id: "user2".into(),
            addr: None,
            color: Color::LightGreen,
        };

        let db_path = Path::new("db");
        let db = Arc::new(Mutex::new(DbRepo::new(&db_path).unwrap()));
        db.lock().unwrap().server_rooms.insert_one(&room).unwrap();

        let mut server = ChatServer::new(room, db.clone()).await;
        server.run().await.unwrap();
        sleep(Duration::from_secs(1)).await;
        let mut client = ChatClient::new(header.clone(), user);
        client.connect().await.unwrap();
        sleep(Duration::from_secs(1)).await;

        client
            .send_msg(UserMsg::SyncReq {
                user: client.user.clone(),
            })
            .await
            .unwrap();

        if let MessageType::Server(ServerMsg::AuthReq { .. }) = client.recv_msg().await.unwrap() {
            assert!(true);
        } else {
            assert!(false);
        }
        sleep(Duration::from_millis(1)).await;
        if let MessageType::Server(ServerMsg::Sync { .. }) = client.recv_msg().await.unwrap() {
            assert!(true);
        } else {
            assert!(false);
        }
        client
            .send_msg(UserMsg::UserJoined {
                user: client.user.clone(),
            })
            .await
            .unwrap();
        sleep(Duration::from_millis(1)).await;

        let sended_msg = TextMessage::new(
            &client.user,
            &client.room.lock().unwrap()._id,
            "some short message",
        );
        let mut sended_msg2 = sended_msg.clone();

        client
            .send_msg(UserMsg::Normal {
                msg: sended_msg.clone(),
            })
            .await
            .unwrap();

        let mut client2 = ChatClient::new(header.clone(), user2);
        client2.connect().await.unwrap();

        client2
            .send_msg(UserMsg::SyncReq {
                user: client2.user.clone(),
            })
            .await
            .unwrap();

        client2
            .send_msg(UserMsg::UserJoined {
                user: client2.user.clone(),
            })
            .await
            .unwrap();

        sleep(Duration::from_millis(1)).await;

        if let MessageType::Server(ServerMsg::AuthReq { .. }) = client2.recv_msg().await.unwrap() {
            assert!(true);
        } else {
            assert!(false);
        }

        if let MessageType::Server(ServerMsg::Sync {
            messages, users, ..
        }) = client2.recv_msg().await.unwrap()
        {
            // assert_eq!(messages[0].room_id, sended_msg.room_id);
            // assert_eq!(messages[0].sender_addr, sended_msg.sender_addr);
            // assert_eq!(messages[0].content, sended_msg.content);
            assert!(users.iter().any(|user| *user == client.user));
        } else {
            assert!(false);
        }
        if let MessageType::User(UserMsg::UserJoined { .. }) = client.recv_msg().await.unwrap() {
            assert!(true);
        } else {
            assert!(false);
        }

        sended_msg2.sender_addr = client2.user.addr.unwrap();
        client2
            .send_msg(UserMsg::Normal {
                msg: sended_msg2.clone(),
            })
            .await
            .unwrap();
        sleep(Duration::from_secs(1)).await;
        let received_msg = client.recv_msg().await.unwrap();
        if let MessageType::User(UserMsg::Normal { msg }) = received_msg {
            assert_eq!(msg.room_id, sended_msg2.room_id);
            assert_eq!(msg.sender_addr, sended_msg2.sender_addr);
            assert_eq!(msg.content, sended_msg2.content);
        }

        client
            .send_msg(UserMsg::BanReq {
                addr: client2.user.addr.unwrap(),
            })
            .await
            .unwrap();
        sleep(Duration::from_secs(1)).await;
        assert_eq!(
            client.recv_msg().await.unwrap(),
            MessageType::Server(ServerMsg::BanConfirm {
                addr: client2.user.addr.unwrap()
            })
        );
        assert_eq!(
            client.recv_msg().await.unwrap(),
            MessageType::Server(ServerMsg::UserLeft {
                addr: client2.user.addr.unwrap(),
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
                .find_one(doc! {})
                .unwrap()
                .unwrap()
                .banned_addrs
        );

        std::fs::remove_dir_all(&db_path).unwrap();
    }
}
