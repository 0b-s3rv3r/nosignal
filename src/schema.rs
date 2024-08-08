use enum_stringify::EnumStringify;
use ratatui::style::Color as ratColor;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::SystemTime};

use crate::network::User;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Room {
    pub _id: String,
    pub addr: SocketAddr,
    pub passwd: Option<String>,
    pub banned_addrs: Vec<SocketAddr>,
    pub is_owner: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TextMessage {
    room_id: String,
    sender_addr: SocketAddr,
    content: String,
    timestamp: SystemTime,
}

impl TextMessage {
    pub fn new(user_addr: &SocketAddr, room_id: &str, msg: &str) -> Self {
        Self {
            sender_addr: *user_addr,
            room_id: room_id.into(),
            content: msg.into(),
            timestamp: SystemTime::now(),
        }
    }

    pub fn sender_addr(&self) -> &SocketAddr {
        &self.sender_addr
    }

    pub fn room_id(&self) -> &String {
        &self.room_id
    }

    pub fn content(&self) -> &String {
        &self.content
    }

    pub fn timestamp(&self) -> &SystemTime {
        &self.timestamp
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalData {
    pub default_user_id: String,
    pub default_room_addr: SocketAddr,
    pub default_color: Color,
    pub remember_passwords: bool,
    pub light_mode: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, EnumStringify)]
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

impl Into<ratColor> for Color {
    fn into(self) -> ratColor {
        match self {
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
