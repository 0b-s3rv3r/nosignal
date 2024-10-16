use enum_stringify::EnumStringify;
use ratatui::style::Color as ratColor;
use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};
use std::{net::SocketAddr, str::FromStr, time::SystemTime};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum Room {
    Server(ServerRoom),
    Header(RoomHeader),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ServerRoom {
    pub _id: String,
    #[serde(deserialize_with = "des_soc_addr")]
    #[serde(serialize_with = "ser_soc_addr")]
    pub addr: SocketAddr,
    pub passwd: Option<String>,
    #[serde(deserialize_with = "des_soc_addr_vec")]
    #[serde(serialize_with = "ser_soc_addr_vec")]
    pub banned_addrs: Vec<SocketAddr>,
}

impl ServerRoom {
    pub fn room_header(&self) -> RoomHeader {
        RoomHeader {
            _id: self._id.clone(),
            addr: self.addr,
            passwd: self.passwd.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct RoomHeader {
    pub _id: String,
    #[serde(deserialize_with = "des_soc_addr")]
    #[serde(serialize_with = "ser_soc_addr")]
    pub addr: SocketAddr,
    pub passwd: Option<String>,
}

impl From<ServerRoom> for Room {
    fn from(val: ServerRoom) -> Self {
        Room::Server(val)
    }
}

impl From<RoomHeader> for Room {
    fn from(val: RoomHeader) -> Self {
        Room::Header(val)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TextMessage {
    pub room_id: String,
    pub sender_addr: SocketAddr,
    pub content: String,
    pub timestamp: Option<SystemTime>,
}

impl TextMessage {
    pub fn new(user_addr: &SocketAddr, room_id: &str, msg: &str) -> Self {
        Self {
            sender_addr: *user_addr,
            room_id: room_id.into(),
            content: msg.into(),
            timestamp: Some(SystemTime::now()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalData {
    pub id: usize,
    pub user_id: String,
    #[serde(deserialize_with = "des_soc_addr")]
    #[serde(serialize_with = "ser_soc_addr")]
    pub room_addr: SocketAddr,
    pub color: Color,
    pub light_mode: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, EnumStringify)]
#[serde(rename_all = "lowercase")]
#[enum_stringify(case = "lower")]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
}

impl From<Color> for ratColor {
    fn from(val: Color) -> Self {
        match val {
            Color::Black => ratColor::Black,
            Color::Red => ratColor::Red,
            Color::Green => ratColor::Green,
            Color::Yellow => ratColor::Yellow,
            Color::Blue => ratColor::Blue,
            Color::Magenta => ratColor::Magenta,
            Color::Cyan => ratColor::Cyan,
            Color::Gray => ratColor::Gray,
            Color::DarkGray => ratColor::DarkGray,
            Color::LightRed => ratColor::LightRed,
            Color::LightGreen => ratColor::LightGreen,
            Color::LightYellow => ratColor::LightYellow,
            Color::LightBlue => ratColor::LightBlue,
            Color::LightMagenta => ratColor::LightMagenta,
            Color::LightCyan => ratColor::LightCyan,
            Color::White => ratColor::White,
        }
    }
}

fn ser_soc_addr<S: Serializer>(addr: &SocketAddr, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&addr.to_string())
}

fn des_soc_addr<'de, D: Deserializer<'de>>(d: D) -> Result<SocketAddr, D::Error> {
    SocketAddr::from_str(&String::deserialize(d)?).map_err(serde::de::Error::custom)
}

fn ser_soc_addr_vec<S: Serializer>(addrs: &Vec<SocketAddr>, s: S) -> Result<S::Ok, S::Error> {
    let mut seq = s.serialize_seq(Some(addrs.len()))?;
    for element in addrs {
        seq.serialize_element(&element.to_string())?;
    }
    seq.end()
}

fn des_soc_addr_vec<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<SocketAddr>, D::Error> {
    Ok(Vec::<String>::deserialize(d)?
        .into_iter()
        .map(|s| SocketAddr::from_str(&s).unwrap())
        .collect::<Vec<SocketAddr>>())
}
