use crossterm::{
    event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{layout::*, prelude::*, style::Style, widgets::*};
use regex::Regex;
use std::{
    io::{self, Stdout},
    rc::Rc,
    time::Duration,
    usize, vec,
};
use tui_pattern_highlighter::highlight_text;
use tui_popup::Popup;
use tui_textarea::{CursorMove, Input, Key, TextArea};

#[derive(PartialEq, Eq)]
enum PopupState {
    Help,
    List,
    Banned,
    None,
}

pub struct ChatTui<'a> {
    msg_list: MsgList<'a>,
    msg_area: MsgArea<'a>,
    help_popup: HelpPopup<'a>,
    user_list_popup: UserListPopup<'a>,
    banned_user_popup: BannedUserPopup,
    popup_state: PopupState,
    width: usize,
    running: bool,
}

impl<'a> ChatTui<'a> {
    pub fn new() {}
    pub fn render(&mut self) -> io::Result<()> {
        let mut term = Self::term_init()?;

        self.ui_render(&mut term);

        Self::term_restore(&mut term)?;
        Ok(())
    }

    fn ui_render(&mut self, term: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        while self.running {
            let msg_list = &mut self.msg_list;
            let msg_area_height = self.msg_area.area_height;

            term.draw(|frame| {
                let layout = Self::get_layout(frame.size(), msg_area_height);
                frame.render_stateful_widget(msg_list, layout[0], &mut msg_list.state);

                match self.popup_state {
                    PopupState::Help => frame.render_widget(self.help_popup, frame.size()),
                    PopupState::List => frame.render_widget(self.user_list_popup, frame.size()),
                    PopupState::Banned => {
                        frame.render_widget(self.banned_user_popup, frame.size());
                        if self.banned_user_popup.has_time_passed() {
                            self.popup_state = PopupState::None;
                        }
                    }
                    PopupState::None => (),
                }
            })?;

            self.handle_input()?;
        }

        Ok(())
    }

    fn handle_input(&mut self) -> io::Result<()> {
        if crossterm::event::poll(Duration::from_millis(50))? {
            match event::read()?.into() {
                Input {
                    key: Key::Char('k'),
                    ctrl: true,
                    ..
                } => self.msg_list.previous(),
                Input {
                    key: Key::Char('j'),
                    ctrl: true,
                    ..
                } => self.msg_list.next(),
                Input {
                    key: Key::Char('q'),
                    ctrl: true,
                    ..
                } => self.running = false,
                Input {
                    key: Key::Enter,
                    ctrl: false,
                    ..
                } => self.msg_list.push_msg(self.msg_area.get_msg()),
                Input {
                    key: Key::Char('l'),
                    ctrl: true,
                    ..
                } => {}
                Input {
                    key: Key::Char('h'),
                    ctrl: true,
                    ..
                } => {}
                Input {
                    key: Key::Char('y'),
                    ctrl: true,
                    ..
                } => self.msg_area.textarea.copy(),
                Input {
                    key: Key::Char('p'),
                    ctrl: true,
                    ..
                } => _ = self.msg_area.textarea.paste(),
                input => self.msg_area.on_input_update(input, self.width),
            }
        }
        Ok(())
    }

    fn term_init() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        terminal.clear()?;
        Ok(terminal)
    }

    fn term_restore(term: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        execute!(term.backend_mut(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn get_layout(frame_size: Rect, msg_area_height: u16) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(90 - msg_area_height),
                Constraint::Percentage(9 + msg_area_height),
            ])
            .split(frame_size)
    }
}

#[derive(Clone)]
struct WidgetStyle {
    block_style: Style,
    font_style: Style,
}

pub struct MsgList<'a> {
    room_name: String,
    messages: Vec<Text<'a>>,
    style: WidgetStyle,
    state: ListState,
}

impl<'a> MsgList<'a> {
    pub fn new(room_name: &str, style: WidgetStyle) -> Self {
        Self {
            room_name: room_name.into(),
            messages: vec![],
            style: style,
            state: ListState::default(),
        }
    }

    pub fn push_msg(&mut self, msg: Message) {
        self.messages.push({
            let name = Line::from(msg.sender_id.clone()).bold();
            let content = highlight_text(msg.content, r"@(\w+)", Style::new().bg(Color::LightBlue));
            let mut text = Text::from(name);
            content
                .lines
                .iter()
                .for_each(|line| text.push_line(line.clone().italic()));
            text
        })
    }

    pub fn next(&mut self) {
        let len = self.messages.len();
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
        if self.messages.len() != 0 {
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

impl<'a> StatefulWidget for MsgList<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut ListState) {
        let msgs_list = List::new(self.messages)
            .block(Block::default().title(self.room_name).borders(Borders::ALL))
            .style(self.style.block_style)
            .direction(ListDirection::TopToBottom)
            .highlight_style(Style::default().fg(Color::Yellow));

        StatefulWidget::render(msgs_list, area, buf, state);
    }
}

pub struct Message {
    sender_id: String,
    content: String,
}

struct MsgArea<'a> {
    style: WidgetStyle,
    textarea: TextArea<'a>,
    area_height: u16,
}

impl<'a> MsgArea<'a> {
    const MAX_AREA_HEIGHT: u16 = 20;

    pub fn new(style: WidgetStyle) -> Self {
        Self {
            style: style,
            textarea: TextArea::default(),
            area_height: 0,
        }
    }

    pub fn on_input_update(&mut self, input: Input, width: usize) {
        if self.textarea.input_without_shortcuts(input) {
            let lines = self.textarea.lines()[self.textarea.cursor().0].clone();

            if lines == " " {
                self.textarea.delete_char();
            }

            if lines.len() >= width - 2 {
                let rlines: String = lines.chars().rev().collect();
                if let Some(caps) = Regex::new(r"\S+").unwrap().captures(&rlines) {
                    let cap = caps.get(0).unwrap();
                    if cap.start() == 0 {
                        self.textarea.delete_word();
                        self.textarea.insert_newline();
                        let rword: String = cap.as_str().chars().rev().collect();
                        self.textarea.insert_str(&rword);
                    } else {
                        self.textarea.move_cursor(CursorMove::Back);
                    }
                }

                if self.area_height <= Self::MAX_AREA_HEIGHT {
                    self.area_height += 2;
                }
            }
        }
    }

    pub fn get_msg(&mut self) -> Message {
        let buf = self.get_buffer();
        self.clear_buffer();
        Message {
            sender_id: "me".into(),
            content: buf,
        }
    }

    fn get_buffer(&mut self) -> String {
        self.textarea
            .lines()
            .iter()
            .map(|line| {
                let mut line_ = line.to_string();
                if !line_.is_empty() && line_ != *self.textarea.lines().last().unwrap() {
                    line_.push('\n');
                }
                line_
            })
            .collect()
    }

    fn clear_buffer(&mut self) {
        for _ in 0..self.textarea.lines().len() {
            self.textarea.move_cursor(CursorMove::End);
            self.textarea.delete_line_by_head();
            self.textarea.delete_newline();
        }
    }
}

impl<'a> Widget for MsgArea<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.textarea.set_style(self.style.font_style);
        self.textarea.set_cursor_line_style(Style::default());
        self.textarea.set_cursor_style(self.style.font_style);
        self.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .set_style(self.style.block_style),
        );
        _ = self.textarea.set_search_pattern(r"@\w+");

        self.textarea.widget().render(area, buf);
    }
}

struct HelpPopup<'a> {
    style: WidgetStyle,
    content: Text<'a>,
}

impl<'a> HelpPopup<'a> {
    pub fn new(content: Vec<Line<'a>>, style: WidgetStyle) -> Self {
        Self {
            style: style.clone(),
            content: Text::from(content).style(style.font_style),
        }
    }
}

impl<'a> Widget for HelpPopup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let help_popup = Popup::new("help", self.content).style(self.style.block_style);
        help_popup.render(area, buf);
    }
}

struct BannedUserPopup {
    style: WidgetStyle,
    display_time: u16,
    counter: u16,
    banned_id: String,
}

impl BannedUserPopup {
    pub fn new(display_time: u16, style: WidgetStyle) -> Self {
        Self {
            style: style,
            display_time: display_time,
            counter: 0,
            banned_id: String::new(),
        }
    }

    pub fn set_banned(&mut self, banned_id: &str) {
        self.banned_id = banned_id.into();
        self.counter = 0;
    }

    pub fn has_time_passed(&mut self) -> bool {
        self.counter >= self.display_time
    }
}

impl Widget for BannedUserPopup {
    fn render(mut self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let banned_user_popup = Popup::new(
            "",
            Text::from(format!("{} has been banned!", self.banned_id)).style(self.style.font_style),
        )
        .style(self.style.block_style);
        banned_user_popup.render(area, buf);

        if self.counter < self.display_time {
            self.counter += 1;
        } else {
            self.counter = 0;
        }
    }
}

struct UserListPopup<'a> {
    style: WidgetStyle,
    users: Text<'a>,
}

impl<'a> UserListPopup<'a> {
    pub fn new(users: Vec<&'a str>, style: WidgetStyle) -> Self {
        Self {
            style: style,
            users: Text::from(
                users
                    .iter()
                    .map(|user| Line::from(*user))
                    .collect::<Text<'a>>(),
            ),
        }
    }
}

impl<'a> Widget for UserListPopup<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let banned_user_popup =
            Popup::new("", self.users.style(self.style.font_style)).style(self.style.block_style);
        banned_user_popup.render(area, buf);
    }
}

enum ChatCommand {
    Set(String),
    Ban(String),
}

enum Event {}

struct Command {
    pattern: String,
    event: Event,
}

struct Commander {
    current_event: Event,
}

impl Commander {
    pub fn new(commands: Vec<Command>) {}

    pub fn add_cmd(pattern: &Regex) {}
}
