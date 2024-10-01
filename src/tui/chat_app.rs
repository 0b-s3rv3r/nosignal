use crate::error::AppError;
use crate::network::client::ChatClient;
use crate::network::{
    message::{Message, MessageType, ServerMsg, UserMsg},
    User,
};
use crate::schema::TextMessage;
use crate::tui::ui::{ChatStyle, MsgItem, PopupState, StatefulArea, StatefulList, Tui};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use log::{info, warn};
use ratatui::{prelude::*, style::Style};
use regex::Regex;
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use tokio::time::Duration;
use tui_textarea::CursorMove;

type IsAuthorized = bool;

pub struct ChatApp<'a> {
    pub running: bool,
    pub style: ChatStyle,
    pub client: ChatClient,
    pub users: HashMap<SocketAddr, User>,
    pub messages: StatefulList<Text<'a>>,
    pub current_popup: PopupState,
    pub msg_area: StatefulArea<'a>,
    pub commands: Vec<Command>,
}

impl<'a> ChatApp<'a> {
    pub fn new(client: ChatClient, light_mode: bool) -> Self {
        let style = if !light_mode {
            ChatStyle::new(
                Style::new().bg(Color::Rgb(0, 0, 0)).fg(Color::White),
                Style::new().fg(Color::Yellow),
            )
        } else {
            ChatStyle::new(
                Style::new().bg(Color::White).fg(Color::Rgb(0, 0, 0)),
                Style::new().fg(Color::Yellow),
            )
        };

        Self {
            running: true,
            style: style.clone(),
            client,
            users: HashMap::new(),
            messages: StatefulList::default(),
            msg_area: StatefulArea::new(style),
            current_popup: PopupState::None,
            commands: vec![(Regex::new(r"/ban\s+(\S+)").unwrap(), Action::Ban)],
        }
    }

    pub async fn run(&mut self) -> Result<(), AppError> {
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        let mut tui = Tui::new(terminal);
        tui.term_init()?;

        while self.running {
            if self.client.is_ok() {
                if !self.handle_msgs().await {
                    return Err(AppError::AuthFailure);
                }
            }
            tui.draw(self)?;
            self.handle_input().await?;
        }

        tui.term_restore()?;
        Ok(())
    }

    async fn handle_input(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(10))? {
            let key_event = event::read()?;

            // this has to be fixed
            if let Event::Key(_) = key_event {
                if self.current_popup != PopupState::None {
                    self.current_popup = PopupState::None;
                }
            }

            match key_event {
                Event::Key(KeyEvent {
                    code, modifiers, ..
                }) => match code {
                    KeyCode::Left => {
                        self.msg_area.textarea.move_cursor(CursorMove::Back);
                    }
                    KeyCode::Right => {
                        self.msg_area.textarea.move_cursor(CursorMove::Forward);
                    }
                    KeyCode::Up => {
                        self.msg_area.textarea.move_cursor(CursorMove::Up);
                    }
                    KeyCode::Down => {
                        self.msg_area.textarea.move_cursor(CursorMove::Down);
                    }
                    KeyCode::Char('k') if modifiers.contains(KeyModifiers::CONTROL) => {
                        self.messages.is_highlighted = true;
                        self.messages.previous();
                    }
                    KeyCode::Char('j') if modifiers.contains(KeyModifiers::CONTROL) => {
                        self.messages.is_highlighted = true;
                        self.messages.next();
                    }
                    KeyCode::Char('q') if modifiers.contains(KeyModifiers::CONTROL) => {
                        self.client.disconnect();
                        self.running = false;
                    }
                    KeyCode::Enter => {
                        if self.client.is_ok() {
                            self.handle_text_buffer().await;
                        }
                    }
                    KeyCode::Char('l') if modifiers.contains(KeyModifiers::CONTROL) => {
                        if self.current_popup == PopupState::List {
                            self.current_popup = PopupState::None;
                        } else {
                            self.current_popup = PopupState::List;
                        }
                    }
                    KeyCode::Char('h') if modifiers.contains(KeyModifiers::CONTROL) => {
                        if self.current_popup == PopupState::Help {
                            self.current_popup = PopupState::None;
                        } else {
                            self.current_popup = PopupState::None;
                        }
                        self.current_popup = PopupState::Help;
                    }
                    KeyCode::Char('y') if modifiers.contains(KeyModifiers::CONTROL) => {
                        self.msg_area.textarea.copy();
                    }
                    KeyCode::Char('p') if modifiers.contains(KeyModifiers::CONTROL) => {
                        _ = self.msg_area.textarea.paste();
                    }
                    KeyCode::Backspace => {
                        self.handle_deleting_chars();
                    }
                    _ => {
                        self.messages.is_highlighted = false;
                        self.msg_area.on_input_update(key_event.into());
                    }
                },
                _ => (),
            }
        }
        Ok(())
    }

    async fn handle_text_buffer(&mut self) {
        self.msg_area.height = 0;

        if let Some(text) = self.msg_area.get_text() {
            if !self.parse_commands(&text).await {
                let user = self.client.user.clone();
                let room = self.client.room.lock().unwrap();
                let msg = TextMessage::new(&user.addr.unwrap(), &room.id, &text);

                match self
                    .client
                    .send_msg(Message::from((
                        UserMsg::Normal { msg: msg.clone() },
                        room.passwd.clone(),
                    )))
                    .await
                {
                    Ok(_) => self.messages.items.push(MsgItem::user_msg(
                        &msg,
                        user.color.clone(),
                        &self.style,
                        user.id.clone(),
                        self.client.user.id.clone(),
                    )),
                    Err(err) => {
                        self.messages.items.push(MsgItem::info_msg(
                            "Failed sending message".to_string(),
                            Color::Rgb(255, 127, 127),
                        ));
                        info!("{}", err);
                    }
                }
            }
        }
    }

    async fn handle_msgs(&mut self) -> IsAuthorized {
        if let Some(msg_type) = self.client.recv_msg().await.take() {
            match msg_type {
                MessageType::User(user_msg) => match user_msg {
                    UserMsg::Normal { msg } => {
                        let user = self.users.get(&msg.sender_addr).unwrap();
                        self.messages.items.push(MsgItem::user_msg(
                            &msg,
                            user.color.clone(),
                            &self.style,
                            user.id.clone(),
                            self.client.user.id.clone(),
                        ));
                    }
                    UserMsg::UserJoined { user } => {
                        self.users.insert(user.addr.unwrap(), user.clone());

                        self.messages.items.push(MsgItem::info_msg(
                            format!("{} has joined", user.id),
                            Color::Rgb(75, 75, 75).into(),
                        ));
                    }
                },
                MessageType::Server(server_msg) => match server_msg {
                    ServerMsg::Sync {
                        messages, users, ..
                    } => {
                        self.users.extend(
                            users
                                .into_iter()
                                .map(|user| (user.addr.unwrap(), user))
                                .collect::<HashMap<SocketAddr, User>>(),
                        );

                        self.messages.items.append(
                            &mut messages
                                .iter()
                                .map(|msg| {
                                    let user = self.users.get(&msg.sender_addr).unwrap();
                                    MsgItem::user_msg(
                                        &msg,
                                        user.color.clone(),
                                        &self.style,
                                        user.id.clone(),
                                        self.client.user.id.clone(),
                                    )
                                })
                                .collect::<Vec<Text>>(),
                        );
                    }
                    ServerMsg::UserLeft { addr } => {
                        self.messages.items.push(MsgItem::info_msg(
                            format!("{} has left", self.users.get(&addr).unwrap().id),
                            Color::Rgb(75, 75, 75).into(),
                        ));
                        self.users.remove(&addr).unwrap();
                    }
                    ServerMsg::BanConfirm { addr } => {
                        if addr == self.client.user.addr.unwrap() {
                            self.messages.items.push(MsgItem::info_msg(
                                "You has been banned from this server".to_string(),
                                Color::Rgb(75, 75, 75),
                            ));
                            self.users.clear();
                        } else {
                            self.messages.items.push(MsgItem::info_msg(
                                format!("{} has been banned", self.users.get(&addr).unwrap().id),
                                Color::Rgb(75, 75, 75).into(),
                            ));
                        }
                    }
                    ServerMsg::ServerShutdown => {
                        self.messages.items.push(MsgItem::info_msg(
                            String::from("Server has been shutted down."),
                            Color::Rgb(75, 75, 75).into(),
                        ));
                    }
                    ServerMsg::AuthFailure => {
                        self.running = false;
                        return false;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        if !self.messages.is_highlighted {
            self.messages.select_last();
        }
        true
    }

    fn handle_deleting_chars(&mut self) {
        if self.msg_area.textarea.cursor().1 == 0 && self.msg_area.textarea.cursor().0 > 0 {
            self.msg_area.textarea.delete_newline();
            self.msg_area.height -= 1;
        } else {
            self.msg_area.textarea.delete_char();
        }
    }

    async fn parse_commands(&self, haystack: &str) -> bool {
        for command in self.commands.iter() {
            if let Some(args) = Self::parse_command(command, haystack) {
                match command.1 {
                    Action::Ban => {
                        if let Some(user_addr) =
                            self.users.iter().find(|(_, user)| user.id == args[1])
                        {
                            self.client
                                .ban(&user_addr.1.addr.unwrap())
                                .await
                                .unwrap_or_else(|err| warn!("{}", err));
                        }
                    }
                }
                return true;
            }
        }

        false
    }

    fn parse_command(command: &Command, haystack: &str) -> Option<Vec<String>> {
        if let Some(captures) = command.0.captures(haystack) {
            return Some(
                captures
                    .iter()
                    .map(|cap| cap.unwrap().as_str().to_string())
                    .collect::<Vec<String>>(),
            );
        }
        None
    }
}

type Command = (Regex, Action);

pub enum Action {
    Ban,
}

#[cfg(test)]
mod test {}
