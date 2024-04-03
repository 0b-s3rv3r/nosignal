use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{Color, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{layout::*, prelude::*, widgets::block::Position, widgets::*};

use std::{
    collections::{vec_deque, VecDeque},
    io::{self, stdout, Stdout},
};

pub struct ChatUi {
    term: Terminal<CrosstermBackend<Stdout>>,
    exit: bool,
    pub input_box: InputBox,
    pub message_window: MessagesWindow,
}

impl ChatUi {
    pub fn init() -> io::Result<Self> {
        execute!(stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;

        Ok(ChatUi {
            term: Terminal::new(CrosstermBackend::new(stdout())).unwrap(),
            exit: false,
            input_box: InputBox::new(),
            message_window: MessagesWindow::new(),
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        while !self.exit {
            self.term.draw(|frame| {
                self.render_frame(frame);
            })?;
            self.handle_message_input()?;
        }

        ChatUi::restore()?;

        Ok(())
    }

    fn restore() -> io::Result<()> {
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

struct InputBox {
    input: String,
    cursor_position: usize,
}

impl InputBox {
    fn new() -> Self {
        InputBox {
            input: String::new(),
            cursor_position: 0,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    fn submit_message(&mut self) -> String {
        let msg = self.input.clone();
        self.input.clear();
        self.reset_cursor();
        msg
    }
}

impl Widget for &InputBox {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let window = Block::new()
            .borders(Borders::TOP)
            .padding(Padding::new(0, 0, 0, 0))
            .render(area, buf);
    }
}
