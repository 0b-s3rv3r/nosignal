use crossterm::{
    event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{layout::*, prelude::*, style::Style, widgets::*};
use regex::Regex;
use std::{io, time::Duration};
use tui_pattern_highlighter::highlight_text;
use tui_popup::Popup;
use tui_textarea::{CursorMove, Input, Key, TextArea};

const HELP_POPUP_CONTENT: &str = "[ctrl+j] scroll down/n[ctrl+j] scroll up";

pub struct App<'a> {
    pub room_id: String,
    pub style: WidgetStyle,
    pub messages: StatefulList<Text<'a>>,
    pub msg_area: StatefulArea<'a>,
    pub current_popup: PopupState,
    pub users: Vec<String>,
    pub last_banned_user: String,
    pub width: u16,
    pub commands: Vec<(Regex, CommandEvent)>,
    pub popup_display_timer: Timer,
    pub running: bool,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let style = WidgetStyle::new(
            Style::new().bg(Color::Black).fg(Color::White),
            Style::new().bg(Color::Black).fg(Color::White),
        );

        let commands = vec![(
            Regex::new(r"/ban\s+(\S+)").unwrap(),
            CommandEvent::BannedUser,
        )];

        Self {
            room_id: String::from("someroom"),
            style: style.clone(),
            messages: StatefulList::default(),
            msg_area: StatefulArea::new(style),
            current_popup: PopupState::None,
            users: vec!["me".to_string()],
            last_banned_user: String::from(""),
            width: 0,
            commands,
            popup_display_timer: Timer::new(100),
            running: true,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        let mut tui = Tui::new(terminal);

        tui.term_init()?;

        while self.running {
            tui.draw(self)?;

            self.handle_input()?;

            self.popup_display_timer.dec();
            if self.popup_display_timer.has_time_passed() {
                self.popup_display_timer.lock();
            }
        }

        tui.term_restore()?;
        Ok(())
    }

    fn handle_input(&mut self) -> io::Result<()> {
        if crossterm::event::poll(Duration::from_millis(10))? {
            let key = event::read()?;
            match key {
                _ => {
                    if self.current_popup != PopupState::None {
                        self.current_popup = PopupState::None;
                    }
                }
            }

            match key.into() {
                Input { key: Key::Left, .. } => {
                    self.msg_area.textarea.move_cursor(CursorMove::Back);
                    Ok(())
                }
                Input {
                    key: Key::Right, ..
                } => {
                    self.msg_area.textarea.move_cursor(CursorMove::Forward);
                    Ok(())
                }
                Input { key: Key::Up, .. } => {
                    self.msg_area.textarea.move_cursor(CursorMove::Up);
                    Ok(())
                }
                Input { key: Key::Down, .. } => {
                    self.msg_area.textarea.move_cursor(CursorMove::Down);
                    Ok(())
                }
                Input {
                    key: Key::Char('k'),
                    ctrl: true,
                    ..
                } => {
                    self.messages.is_highlighted = true;
                    self.messages.previous();
                    Ok(())
                }
                Input {
                    key: Key::Char('j'),
                    ctrl: true,
                    ..
                } => {
                    self.messages.is_highlighted = true;
                    self.messages.next();
                    Ok(())
                }
                Input {
                    key: Key::Char('q'),
                    ctrl: true,
                    ..
                } => {
                    self.running = false;
                    Ok(())
                }
                Input {
                    key: Key::Enter,
                    ctrl: false,
                    ..
                } => {
                    self.msg_area.area_height = 0;
                    if let Some(msg) = self.msg_area.get_msg() {
                        if !self.handle_commands(&msg.text.to_string()) {
                            self.messages.items.push(msg.text);
                        }
                    }
                    Ok(())
                }
                Input {
                    key: Key::Char('l'),
                    ctrl: true,
                    ..
                } => {
                    self.current_popup = PopupState::List;
                    Ok(())
                }
                Input {
                    key: Key::Char('h'),
                    ctrl: true,
                    ..
                } => {
                    self.current_popup = PopupState::Help;
                    Ok(())
                }
                Input {
                    key: Key::Char('y'),
                    ctrl: true,
                    ..
                } => {
                    self.msg_area.textarea.copy();
                    Ok(())
                }
                Input {
                    key: Key::Char('p'),
                    ctrl: true,
                    ..
                } => {
                    _ = self.msg_area.textarea.paste();
                    Ok(())
                }
                Input {
                    key: Key::Backspace,
                    ..
                } => {
                    if self.msg_area.textarea.cursor().1 == 0
                        && self.msg_area.textarea.cursor().0 > 0
                    {
                        self.msg_area.textarea.delete_newline();
                        self.msg_area.area_height -= 1;
                    } else {
                        self.msg_area.textarea.delete_char();
                    }
                    Ok(())
                }
                input => {
                    self.messages.is_highlighted = false;
                    self.messages.state.select(Some(self.messages.items.len()));
                    self.msg_area.on_input_update(input, self.width);
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    fn handle_commands(&mut self, msg: &str) -> bool {
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
                    self.last_banned_user = capture;
                    self.current_popup = PopupState::Banned;
                    self.popup_display_timer.unlock();
                }
                CommandEvent::SetOption => (),
            }
            return true;
        }
        false
    }
}

struct Tui<B: Backend> {
    terminal: Terminal<B>,
}

impl<B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>) -> Self {
        Self { terminal }
    }

    pub fn draw(&mut self, app: &mut App) -> io::Result<()> {
        self.terminal.draw(|frame| Self::render(app, frame))?;
        Ok(())
    }

    fn render(app: &mut App, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(90 - app.msg_area.area_height),
                Constraint::Length(5 + app.msg_area.area_height),
            ])
            .split(frame.size());
        app.width = layout[0].width;

        let mut msgs_list = List::new(app.messages.items.clone())
            .block(
                Block::default()
                    .title(app.room_id.clone())
                    .borders(Borders::ALL)
                    .padding(Padding::new(2, 2, 1, 1))
                    .border_set(symbols::border::ROUNDED),
            )
            .style(app.style.block_style)
            .direction(ListDirection::TopToBottom);
        if app.messages.is_highlighted {
            msgs_list = msgs_list.highlight_style(Style::new().fg(Color::Yellow));
        }

        frame.render_stateful_widget(msgs_list, layout[0], &mut app.messages.state);

        frame.render_widget(app.msg_area.textarea.widget(), layout[1]);

        match app.current_popup {
            PopupState::Help => {
                let help_popup =
                    Popup::new("help", HELP_POPUP_CONTENT).style(app.style.block_style);
                frame.render_widget(&help_popup, frame.size());
            }
            PopupState::List => {
                let user_list_popup = Popup::new(
                    "",
                    app.users
                        .iter()
                        .map(|user| Line::from(user.clone()).style(app.style.font_style))
                        .collect::<Text>(),
                )
                .style(app.style.block_style);
                frame.render_widget(&user_list_popup, frame.size());
            }
            PopupState::Banned => {
                if app.popup_display_timer.has_time_passed() {
                    app.current_popup = PopupState::None
                }

                let banned_user_popup = Popup::new(
                    "",
                    Text::from(format!("{} has been banned!", app.last_banned_user))
                        .style(app.style.font_style),
                )
                .style(app.style.block_style);
                frame.render_widget(&banned_user_popup, frame.size());
            }
            PopupState::None => {}
        }
    }

    fn term_init(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen)?;
        self.terminal.clear()?;
        Ok(())
    }

    fn term_restore(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }
}

pub struct StatefulArea<'a> {
    textarea: TextArea<'a>,
    area_height: u16,
}

impl<'a> StatefulArea<'a> {
    const MAX_AREA_HEIGHT: u16 = 20;

    pub fn new(style: WidgetStyle) -> Self {
        let mut textarea = TextArea::default();
        textarea.set_style(style.font_style);
        textarea.set_cursor_line_style(style.font_style);
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .set_style(style.block_style)
                .padding(Padding::new(2, 2, 1, 1))
                .border_set(symbols::border::ROUNDED),
        );
        _ = textarea.set_search_pattern(r"@\w+");
        textarea.set_placeholder_text("Type your message here...");
        textarea.set_placeholder_style(Style::new().fg(Color::Gray));

        Self {
            textarea,
            area_height: 0,
        }
    }

    pub fn on_input_update(&mut self, input: Input, width: u16) {
        if self.textarea.input_without_shortcuts(input) {
            let line = self.textarea.lines()[self.textarea.cursor().0].clone();

            if line.len() >= (width - 6).into() {
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

                if self.area_height <= Self::MAX_AREA_HEIGHT && !line.ends_with(' ') {
                    self.area_height += 1;
                }
            }
        }
    }

    pub fn get_msg(&mut self) -> Option<MessageItem<'a>> {
        let buffer = self.get_buffer();
        self.clear_buffer();
        if let Some(buf) = buffer {
            return Some(MessageItem::new("me".into(), buf));
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

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
    pub is_highlighted: bool,
}

impl<T> Default for StatefulList<T> {
    fn default() -> Self {
        Self {
            state: ListState::default(),
            items: Vec::new(),
            is_highlighted: false,
        }
    }
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
            is_highlighted: false,
        }
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

pub struct MessageItem<'a> {
    pub text: Text<'a>,
}

impl<'a> MessageItem<'a> {
    pub fn new(sender_id: String, content: String) -> Self {
        let name = Line::from(sender_id.clone()).bold();
        let mut content = highlight_text(content, r"@(\w+)", Style::new().bg(Color::LightBlue));
        let mut text = Text::from(name);
        content
            .lines
            .iter()
            .for_each(|line| text.push_line(line.clone().italic()));
        content.push_line(Line::from("\n"));

        Self { text }
    }
}

pub struct Timer {
    treshold_time: usize,
    counter: usize,
    start: bool,
}

impl Timer {
    pub fn new(treshold_time: usize) -> Self {
        Self {
            treshold_time,
            counter: 0,
            start: false,
        }
    }

    pub fn unlock(&mut self) {
        self.start = true;
    }

    pub fn lock(&mut self) {
        self.start = false;
    }

    pub fn dec(&mut self) {
        if self.start {
            if self.counter <= 0 {
                self.counter = self.treshold_time;
            }

            self.counter -= 1;
        }
    }

    pub fn has_time_passed(&self) -> bool {
        if self.counter > 0 {
            return false;
        }
        true
    }
}

#[derive(Clone)]
pub struct WidgetStyle {
    pub block_style: Style,
    pub font_style: Style,
}

impl WidgetStyle {
    pub fn new(block_style: Style, font_style: Style) -> Self {
        Self {
            block_style,
            font_style,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum PopupState {
    Help,
    List,
    Banned,
    None,
}

#[derive(Clone, Copy)]
pub enum CommandEvent {
    BannedUser,
    SetOption,
}
