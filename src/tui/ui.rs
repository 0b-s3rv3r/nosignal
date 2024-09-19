use crate::{
    schema::{Color as ChatColor, TextMessage},
    tui::chat_app::ChatApp,
    util::systime_to_string,
};
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
    "[ctrl+l] user list\n[ctrl+j] scroll down\n[ctrl+j] scroll up\n[ctrl+q] exit";

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
                    .title(app.client.room.lock().unwrap()._id.clone())
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
                    width: 21,
                    height: 5,
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
                            .map(|(_, user)| {
                                Line::from(format!(
                                    "{} [{}]",
                                    user._id,
                                    user.addr.unwrap().ip().to_string()
                                ))
                            })
                            .collect::<Text>(),
                    ),
                    width: 21,
                    height: 5,
                })
                .style(app.style.block)
                .border_set(border::ROUNDED)
                .title("users list");
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

    pub fn new(style: ChatStyle, user_id: String) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(style.block);
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .set_style(style.block)
                .padding(Padding::new(2, 2, 1, 1))
                .border_set(border::ROUNDED),
        );
        textarea
            .set_search_pattern(&format!(r"@{}", user_id))
            .unwrap();
        textarea.set_search_style(style.mentioning);
        textarea.set_placeholder_text("Start typing...");
        textarea.set_placeholder_style(Style::new().fg(Color::Gray));

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

        if line.len() >= (self.width - 6).into() {
            let rlines: String = line.chars().rev().collect();
            if let Some(caps) = Regex::new(r"\S+").unwrap().captures(&rlines) {
                let cap = caps.get(0).unwrap();
                if cap.start() == 0 {
                    self.textarea.delete_word();
                    self.textarea.insert_newline();
                    let rword: String = cap.as_str().chars().rev().collect();
                    self.textarea.insert_str(&rword);
                } else {
                    self.textarea.delete_char();
                }
            }

            if self.height <= Self::MAX_AREA_HEIGHT && !line.ends_with(' ') {
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
        text.style(Style::new().fg(color.into()).italic())
    }

    pub fn user_msg<'a>(
        text_msg: &TextMessage,
        user_color: impl Into<Color>,
        content_color: impl Into<Color>,
        user_id: String,
        target_user: String,
    ) -> Text<'a> {
        let mut text = Text::from(Line::from(vec![
            Span::from(user_id.clone()).style(Style::new().bold().fg(user_color.into())),
            Span::from(format!(" {}", systime_to_string(text_msg.timestamp)))
                .fg(Color::Rgb(50, 50, 50))
                .italic(),
        ]));
        let content = highlight_text(
            text_msg.content.clone(),
            &format!(r"@{}", target_user),
            Style::new().bg(Color::White).fg(Color::Black).bold(),
        );

        let content_color_ = content_color.into();
        content
            .lines
            .into_iter()
            .for_each(|line| text.push_line(line.fg(content_color_).not_bold()));
        text.push_line("");
        text
    }
}

#[derive(Clone, Debug)]
pub struct ChatStyle {
    pub block: Style,
    pub msg_content: Style,
    pub msg_highlight: Style,
    pub mentioning: Style,
}

impl ChatStyle {
    pub fn new(block: Style, msg_content: Style, msg_highlight: Style, mentioning: Style) -> Self {
        Self {
            block,
            msg_content,
            msg_highlight,
            mentioning,
        }
    }

    pub fn reverse_colors(&mut self) {
        self.block = self.block.reversed();
        self.msg_content = self.msg_content.reversed();
        self.msg_highlight = self.msg_highlight.reversed();
        self.mentioning = self.mentioning.reversed();
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PopupState {
    Help,
    List,
    None,
}
