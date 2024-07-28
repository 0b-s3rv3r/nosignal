use crate::network::client::ChatClient;
use crate::network::ChatMessage;
use crate::schema::Message;
use crate::tui::ui::{
    MessageItem, PopupState, StatefulArea, StatefulList, Timer, Tui, WidgetStyle,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, style::Style};
use regex::Regex;
use std::io;
use std::time::SystemTime;
use tokio::time::Duration;
use tui_textarea::CursorMove;

#[derive(Debug)]
pub struct ChatApp<'a> {
    pub style: WidgetStyle,
    pub messages: StatefulList<Text<'a>>,
    pub msg_area: StatefulArea<'a>,
    pub current_popup: PopupState,
    pub users: Vec<String>,
    pub last_banned_user: String,
    pub commands: Vec<(Regex, CommandEvent)>,
    pub popup_display_timer: Timer,
    pub running: bool,
    pub client: ChatClient,
}

impl<'a> ChatApp<'a> {
    pub fn new(client: ChatClient) -> Self {
        let style = WidgetStyle::new(
            Style::new().bg(Color::Black).fg(Color::White),
            Style::new().bg(Color::Black).fg(Color::White),
        );

        let commands = vec![(
            Regex::new(r"/ban\s+(\S+)").unwrap(),
            CommandEvent::BannedUser,
        )];

        Self {
            style: style.clone(),
            messages: StatefulList::default(),
            msg_area: StatefulArea::new(style),
            current_popup: PopupState::None,
            users: vec!["me".to_string()],
            last_banned_user: String::from(""),
            commands,
            popup_display_timer: Timer::new(100),
            running: true,
            client,
        }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        let mut tui = Tui::new(terminal);
        tui.term_init()?;

        while self.running {
            tui.draw(self)?;
            self.handle_input().await?;
            self.receive_msg().await;
            self.handle_popup_timer();
        }

        tui.term_restore()?;
        Ok(())
    }

    async fn handle_input(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(10))? {
            let key_event = event::read()?;

            // this have to be fixed
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
                        self.running = false;
                    }
                    KeyCode::Enter => {
                        self.handle_text_buffer().await;
                    }
                    KeyCode::Char('l') if modifiers.contains(KeyModifiers::CONTROL) => {
                        self.current_popup = PopupState::List;
                    }
                    KeyCode::Char('h') if modifiers.contains(KeyModifiers::CONTROL) => {
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
            if !self.handle_commands(&text).await {
                let msg = Message {
                    msg_id: 0,
                    sender_id: self.client.user.id.clone(),
                    sender_color: self.client.user.color.clone(),
                    chatroom_id: self.client.room.id.clone(),
                    content: text,
                    timestamp: SystemTime::now(),
                };

                self.client
                    .send(&ChatMessage::Normal {
                        msg: msg.clone(),
                        passwd: self.client.room.passwd.clone(),
                    })
                    .await
                    .unwrap();

                self.messages.items.push(MessageItem::from(msg).0);
                self.messages.select_last();
            }
        }
    }

    async fn handle_commands(&mut self, msg: &str) -> bool {
        if let Some((event, capture)) = (|| {
            for command in self.commands.iter() {
                if let Some(captures) = command.0.captures(msg) {
                    return Some((command.1, captures.get(1).unwrap().as_str().to_string()));
                }
            }
            None
        })() {
            match event {
                CommandEvent::BannedUser => {
                    if let Some(user) = self
                        .client
                        .users
                        .iter()
                        .filter(|user| user.id == capture)
                        .next()
                    {
                        self.client
                            .send(&ChatMessage::Ban {
                                addr: user.addr.unwrap(),
                                passwd: self.client.room.passwd.clone(),
                            })
                            .await
                            .unwrap();
                        self.last_banned_user = capture;
                        self.current_popup = PopupState::Banned;
                        self.popup_display_timer.unlock();
                    }
                    return false;
                }
                CommandEvent::SetOption => (),
            }
            return true;
        }
        false
    }

    async fn receive_msg(&mut self) {
        if let Some(msg) = self.client.receive().await {
            match msg {
                ChatMessage::Normal { msg, .. } => {
                    self.messages.items.push(MessageItem::from(msg).0);
                    self.messages.state.select(Some(self.messages.items.len()));
                }
                ChatMessage::Ban { addr, .. } => {
                    if let Some(user) = self
                        .client
                        .users
                        .iter()
                        .filter(|user| user.addr.unwrap() == addr)
                        .next()
                    {
                        self.last_banned_user = user.id.clone();
                        self.current_popup = PopupState::Banned;
                    }
                }
                ChatMessage::UserJoined { user, .. } => {
                    todo!("change search_pattern of textarea when new user joined")
                }
                ChatMessage::UserLeft { user_id } => {
                    todo!("change search_pattern of textarea when new user joined")
                }
                ChatMessage::ServerShutdown => {}
                ChatMessage::AuthFailure_ => {}
                _ => (),
            }
        }
    }

    fn handle_popup_timer(&mut self) {
        self.popup_display_timer.dec();
        if self.popup_display_timer.has_time_passed() {
            self.popup_display_timer.lock();
        }
    }

    fn handle_deleting_chars(&mut self) {
        if self.msg_area.textarea.cursor().1 == 0 && self.msg_area.textarea.cursor().0 > 0 {
            self.msg_area.textarea.delete_newline();
            self.msg_area.height -= 1;
        } else {
            self.msg_area.textarea.delete_char();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CommandEvent {
    BannedUser,
    SetOption,
}

#[cfg(test)]
mod test {}
