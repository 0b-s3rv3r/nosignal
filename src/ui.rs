use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Color, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{layout::*, prelude::*, widgets::block::Position, widgets::*};
use tui_textarea::TextArea;

use std::{
    collections::{vec_deque, VecDeque},
    io::{self, stdout, Stdout},
};

// use crate::schema::AppOption;
//
// pub enum ChatEvent {
//     SendMessage(String),
//     KickUser(String),
//     SetOption(AppOption),
// }

pub struct ChatUi {
    term: Terminal<CrosstermBackend<Stdout>>,
    exit: bool,
    pub message_window: MessagesWindow,
}

impl ChatUi {
    pub fn init() -> io::Result<Self> {
        execute!(stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;

        Ok(ChatUi {
            term: Terminal::new(CrosstermBackend::new(stdout())).unwrap(),
            exit: false,
            message_window: MessagesWindow::new(),
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut text_area = TextArea::default();

        while !self.exit {
            self.term.draw(|frame| {
                self.render_frame(frame);
            })?;
            self.handle_message_input()?;
        }

        ChatUi::deinit()?;

        Ok(())
    }

    fn deinit() -> io::Result<()> {
        execute!(stdout(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) -> io::Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
            .split(frame.size());

        frame.render_widget(&MessagesWindow::new(), layout[0]);
        frame.render_widget(&InputBox::new(), layout[1]);

        Ok(())
    }

    fn handle_message_input(&mut self) -> io::Result<Option<String>> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => return Ok(Some(self.input_box.submit_message())),
                    KeyCode::Char(to_insert) => {
                        self.input_box.enter_char(to_insert);
                        return Ok(None);
                    }
                    KeyCode::Backspace => {
                        self.input_box.delete_char();

                        return Ok(None);
                    }
                    KeyCode::Left => {
                        self.input_box.move_cursor_left();

                        return Ok(None);
                    }
                    KeyCode::Right => {
                        self.input_box.move_cursor_right();

                        return Ok(None);
                    }
                    _ => return Ok(None),
                }
            }
        }
        Ok(None)
    }
}

struct MessagesWindow {
    messages: VecDeque<String>,
}

impl MessagesWindow {
    fn new() -> Self {
        MessagesWindow {
            messages: VecDeque::new(),
        }
    }

    fn push_msg(&mut self, message: &str) {
        self.messages.push_back(message.to_owned());
    }
}

impl Widget for &MessagesWindow {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let keys = Line::from(vec![
            "quit ".into(),
            "<ctrl+x>".into(),
            "commands ".into(),
            "</>".into(),
        ]);

        let window = Block::new()
            .title("kioto")
            .title_position(Position::Bottom)
            .title_alignment(Alignment::Center)
            .title(keys)
            .title_position(Position::Bottom)
            .title_alignment(Alignment::Left)
            .borders(Borders::TOP)
            .padding(Padding::new(0, 0, 0, 0))
            .render(area, buf);
    }
}
