use crate::network::client::ChatClient;
use crate::network::{Message, MessageType, UserMsg, UserReqMsg};
use crate::schema::TextMessage;
use crate::tui::ui::{MessageItem, PopupState, StatefulArea, StatefulList, Tui, WidgetStyle};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, style::Style};
use regex::Regex;
use std::io;
use tokio::time::Duration;
use tui_textarea::CursorMove;

pub struct ChatApp<'a> {
    pub(super) running: bool,
    pub(super) client: ChatClient,
    pub(super) style: WidgetStyle,
    pub(super) messages: StatefulList<Text<'a>>,
    pub(super) commands: Vec<Command>,
    pub(super) current_popup: PopupState,
    pub(super) msg_area: StatefulArea<'a>,
}

impl<'a> ChatApp<'a> {
    pub fn new(client: ChatClient, light_mode: bool) -> Self {
        let mut style = WidgetStyle::new(
            Style::new().bg(Color::Rgb(0, 0, 0)).fg(Color::White),
            Style::new().bg(Color::Rgb(0, 0, 0)).fg(Color::White),
        );

        if light_mode {
            style.font_style = style.font_style.reversed();
            style.block_style = style.block_style.reversed();
        }

        Self {
            style: style.clone(),
            messages: StatefulList::default(),
            msg_area: StatefulArea::new(style),
            current_popup: PopupState::None,
            commands: vec![(Regex::new(r"/ban\s+(\S+)").unwrap(), Action::Ban)],
            running: true,
            client,
        }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        let mut tui = Tui::new(terminal);
        tui.term_init()?;

        while self.running {
            self.receive_msg().await;
            tui.draw(self)?;
            self.handle_input().await?;
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
            if !self.parse_commands(&text).await {
                let msg = TextMessage::new(&self.client.user, &self.client.room._id, &text);

                self.client
                    .send_msg(&Message {
                        msg_type: MessageType::User(UserMsg::Normal { msg: msg.clone() }),
                        passwd: self.client.room.passwd.clone(),
                    })
                    .await
                    .unwrap();

                self.messages.items.push(MessageItem::from(msg).0);
                self.messages.select_last();
            }
        }
    }

    async fn receive_msg(&mut self) {
        if let Some(msg) = self.client.recv_msg().await {
            match msg.msg_type {
                MessageType::User(_) => todo!(),
                MessageType::UserReq(_) => todo!(),
                MessageType::Server(_) => todo!(),
            }
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

    async fn parse_commands(&self, haystack: &str) -> bool {
        for command in self.commands.iter() {
            if let Some(args) = Self::parse_command(command, haystack) {
                match command.1 {
                    Action::Ban => self
                        .client
                        .send_msg(&Message {
                            msg_type: MessageType::UserReq(UserReqMsg::BanReq {
                                addr: self.client.users.get(&args[0]).unwrap().addr.unwrap(),
                            }),
                            passwd: self.client.room.passwd.clone(),
                        })
                        .await
                        .unwrap(),
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

enum Action {
    Ban,
}

#[cfg(test)]
mod test {}
