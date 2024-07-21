use enum_stringify::EnumStringify;
use ratatui::style::Color as ratColor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Room {
    pub id: String,
    pub addr: String,
    pub passwd: Option<String>,
    pub banned_addrs: Vec<String>,
    pub is_owner: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Message {
    pub msg_id: u32,
    pub sender_id: String,
    pub sender_color: Color,
    pub chatroom_id: String,
    pub content: String,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalData {
    pub addr: String,
    pub username: String,
    pub color: Color,
    pub remember_passwords: bool,
    pub light_mode: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, EnumStringify)]
pub enum Color {
    Reset,
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
            Color::Reset => ratColor::Reset,
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
