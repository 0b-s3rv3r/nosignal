use crate::{schema::TextMessage, tui::chat_app::ChatApp, util::systime_to_string};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    layout::*,
    prelude::*,
    style::{Style, Styled},
    symbols::border,
    widgets::*,
};
use regex::Regex;
use std::io;
use tui_pattern_highlighter::highlight_text;
use tui_popup::{Popup, SizedWrapper};
use tui_textarea::{CursorMove, Input, TextArea};

const HELP_POPUP_CONTENT: &str =
    "[ctrl+q] exit\n[ctrl+l] user list\n[ctrl+j] scroll down\n[ctrl+k] scroll up\n[/ban <username>] ban user\n[@<username>] mention";

#[derive(Debug)]
pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
}

impl<B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>) -> Self {
        Self { terminal }
    }

    pub fn draw(&mut self, app: &mut ChatApp) -> io::Result<()> {
        self.terminal.draw(|frame| Self::render(app, frame))?;
        Ok(())
    }

    pub fn render(app: &mut ChatApp, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(90 - app.msg_area.height),
                Constraint::Length(5 + app.msg_area.height),
            ])
            .split(frame.size());
        app.msg_area.width = layout[0].width;

        let mut msgs_list = List::new(app.messages.items.clone())
            .block(
                Block::default()
                    .title_top(
                        Line::from(app.client.room.lock().unwrap()._id.clone()).left_aligned(),
                    )
                    .title_bottom(Line::from("ctrl+[h]elp").right_aligned())
                    .borders(Borders::ALL)
                    .padding(Padding::new(2, 2, 1, 1))
                    .border_set(border::ROUNDED),
            )
            .style(app.style.block)
            .direction(ListDirection::TopToBottom);
        if app.messages.is_highlighted {
            msgs_list = msgs_list.highlight_style(app.style.msg_highlight);
        }

        frame.render_stateful_widget(msgs_list, layout[0], &mut app.messages.state);
        frame.render_widget(app.msg_area.textarea.widget(), layout[1]);

        match app.current_popup.clone() {
            PopupState::Help => {
                let popup_content = Paragraph::new(Text::from(HELP_POPUP_CONTENT));
                let help_popup = Popup::new(SizedWrapper {
                    inner: popup_content,
                    width: 27,
                    height: 6,
                })
                .style(app.style.block)
                .border_set(border::ROUNDED)
                .title("help");
                frame.render_widget(&help_popup, frame.size());
            }
            PopupState::List => {
                let user_list_popup = Popup::new(SizedWrapper {
                    inner: Paragraph::new(
                        app.users
                            .iter()
                            .enumerate()
                            .map(|(n, (_, user))| {
                                if n < 10 - 2 {
                                    if user.addr.unwrap() == app.client.user.addr.unwrap() {
                                        Line::from(format!(
                                            "{} [{}]*",
                                            user.id,
                                            user.addr.unwrap().to_string()
                                        ))
                                        .fg(user.color.clone())
                                    } else {
                                        Line::from(format!(
                                            "{} [{}]",
                                            user.id,
                                            user.addr.unwrap().to_string()
                                        ))
                                        .fg(user.color.clone())
                                    }
                                } else {
                                    Line::from("...")
                                }
                            })
                            .collect::<Text>(),
                    ),
                    width: 32,
                    height: 25,
                })
                .style(app.style.block)
                .border_set(border::ROUNDED)
                .title("users");
                frame.render_widget(&user_list_popup, frame.size());
            }
            _ => (),
        }
    }

    pub fn term_init(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen)?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn term_restore(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct StatefulArea<'a> {
    pub textarea: TextArea<'a>,
    pub height: u16,
    pub width: u16,
}

impl<'a> StatefulArea<'a> {
    const MAX_AREA_HEIGHT: u16 = 20;

    pub fn new(style: ChatStyle) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(style.block);
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .set_style(style.block)
                .padding(Padding::new(2, 2, 1, 1))
                .border_set(border::ROUNDED),
        );
        textarea.set_search_pattern(r"@\w+").unwrap();
        textarea.set_search_style(style.block.reversed());
        textarea.set_placeholder_text("Start typing...");
        textarea.set_placeholder_style(Style::new().fg(Color::Gray).bg(style.block.bg.unwrap()));

        Self {
            textarea,
            height: 0,
            width: 0,
        }
    }

    pub fn on_input_update(&mut self, input: Input) {
        if self.textarea.input_without_shortcuts(input) {
            self.move_last_word_to_new_line();
        }
    }

    fn move_last_word_to_new_line(&mut self) {
        let line = self.textarea.lines()[self.textarea.cursor().0].clone();

        let mut insert_nl = false;
        if line.len() >= (self.width - 6).into() {
            let rlines: String = line.chars().rev().collect();
            if let Some(caps) = Regex::new(r"\S+").unwrap().captures(&rlines) {
                let cap = caps.get(0).unwrap();
                if cap.start() == 0 {
                    let rword: String = cap.as_str().chars().rev().collect();
                    if rword.len() >= (self.width - 6).into() {
                        self.textarea.delete_char();
                    } else {
                        self.textarea.delete_word();
                        self.textarea.insert_newline();
                        self.textarea.insert_str(&rword);
                        insert_nl = true;
                    }
                } else {
                    self.textarea.delete_char();
                }
            }

            if self.height <= Self::MAX_AREA_HEIGHT && insert_nl {
                self.height += 1;
            }
        }
    }

    pub fn get_text(&mut self) -> Option<String> {
        let buffer = self.get_buffer();
        self.clear_buffer();
        if let Some(buf) = buffer {
            return Some(buf);
        }
        None
    }

    fn get_buffer(&mut self) -> Option<String> {
        let lines: String = self
            .textarea
            .lines()
            .iter()
            .map(|line| {
                let mut line_ = line.to_string();
                if !line_.is_empty() && line_ != *self.textarea.lines().last().unwrap() {
                    line_.push('\n');
                }
                line_
            })
            .collect();

        if lines.trim().is_empty() {
            return None;
        }
        Some(lines)
    }

    fn clear_buffer(&mut self) {
        for _ in 0..self.textarea.lines().len() {
            self.textarea.move_cursor(CursorMove::End);
            self.textarea.delete_line_by_head();
            self.textarea.delete_newline();
        }
    }
}

#[derive(Debug)]
pub struct StatefulList<T> {
    pub items: Vec<T>,
    pub state: ListState,
    pub is_highlighted: bool,
}

impl<T> Default for StatefulList<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            state: ListState::default(),
            is_highlighted: false,
        }
    }
}

impl<T> StatefulList<T> {
    pub fn select_last(&mut self) {
        self.state.select(Some(self.items.len()));
    }

    pub fn next(&mut self) {
        let len = self.items.len();
        if len != 0 {
            let i = match self.state.selected() {
                Some(i) => {
                    if i >= len - 1 {
                        i
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }

    pub fn previous(&mut self) {
        if !self.items.is_empty() {
            let i = match self.state.selected() {
                Some(i) => {
                    if i == 0 {
                        i
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.select(Some(i));
        }
    }
}

#[derive(Debug)]
pub struct MsgItem;

impl MsgItem {
    pub fn info_msg<'a>(msg: String, color: Color) -> Text<'a> {
        let mut text = Text::from(msg);
        text.push_line("");
        text.style(Style::new().fg(color).italic())
    }

    pub fn user_msg<'a>(
        text_msg: &TextMessage,
        user_id: String,
        user_color: impl Into<Color>,
        chat_style: &ChatStyle,
        target_user: String,
    ) -> Text<'a> {
        let mut text = Text::from(Line::from(vec![
            Span::from(user_id.clone()).style(Style::new().bold().fg(user_color.into())),
            Span::from(format!(" {}", {
                if let Some(ts) = text_msg.timestamp {
                    systime_to_string(ts)
                } else {
                    "unknown timestamp".to_string()
                }
            }))
            .fg(Color::Rgb(50, 50, 50))
            .italic(),
        ]));
        let content = highlight_text(
            text_msg.content.clone(),
            &format!(r"@{}", target_user),
            chat_style.block.reversed().bold(),
        );

        content
            .lines
            .into_iter()
            .for_each(|line| text.push_line(line.not_bold()));
        text.push_line("");
        text
    }
}

#[derive(Clone, Debug)]
pub struct ChatStyle {
    pub block: Style,
    pub msg_highlight: Style,
}

impl ChatStyle {
    pub fn new(block: Style, msg_highlight: Style) -> Self {
        Self {
            block,
            msg_highlight,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PopupState {
    Help,
    List,
    None,
}
