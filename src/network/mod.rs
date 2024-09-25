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
    async fn messages_sending() {
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

        let db = Arc::new(Mutex::new(DbRepo::memory_init().unwrap()));
        let mut peermap = HashMap::<SocketAddr, User>::new();

        let server_room = room.clone();
        let mut server = ChatServer::new(server_room, db).await.unwrap();
        server.run().await.unwrap();
        sleep(Duration::from_secs(1)).await;

        let room_ = room.clone();
        let mut client = ChatClient::new(room_, user.clone());
        client.connect().await.unwrap();
        sleep(Duration::from_millis(1)).await;

        if let MessageType::User(UserMsg::UserJoined { user: user_ }) =
            client.recv_msg().await.unwrap()
        {
            user.addr = user_.addr;
            peermap.insert(user.addr.unwrap(), user.clone());
            assert_eq!(user, user_)
        } else {
            assert!(false);
        }

        let sended_msg = TextMessage::new(&user.addr.unwrap(), &room._id, "some short message");
        let mut sended_msg2 = sended_msg.clone();
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
        sleep(Duration::from_millis(1)).await;
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

        client2.sync().await.unwrap();
        sleep(Duration::from_millis(1)).await;
        if let MessageType::Server(ServerMsg::Sync { messages, users }) =
            client2.recv_msg().await.unwrap()
        {
            assert_eq!(messages[0].room_id, sended_msg.room_id);
            assert_eq!(messages[0].sender_addr, sended_msg.sender_addr);
            assert_eq!(messages[0].content, sended_msg.content);
            assert_eq!(users[0], peermap.get(&user.addr.unwrap()).unwrap().clone());
        } else {
            assert!(false);
        }

        sended_msg2.sender_addr = user2.addr.unwrap();
        client2
            .send_msg(Message::from((
                UserMsg::Normal {
                    msg: sended_msg2.clone(),
                },
                room2.passwd.clone(),
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

        // client2
        //     .send_msg(Message::from((
        //         UserMsg::Normal {
        //             msg: sended_msg2.clone(),
        //         },
        //         None,
        //     )))
        //     .await
        //     .unwrap();
        //
        // sleep(Duration::from_millis(1)).await;
        //
        // assert_eq!(
        //     client2.recv_msg().await.unwrap(),
        //     MessageType::Server(ServerMsg::AuthFailure)
        // );

        client
            .send_msg(Message::from((
                UserReqMsg::BanReq {
                    addr: user2.addr.unwrap(),
                },
                room.passwd,
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

        client.close_connection();
        client2.close_connection();
    }
}
