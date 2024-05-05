use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

<<<<<<< Updated upstream
use ratatui::{layout::*, prelude::*, widgets::block::Position, widgets::*};
use tui_textarea::TextArea;

use std::{
    collections::{vec_deque, VecDeque},
    io::{self, stdout, Stdout},
=======
use ratatui::{
    layout::*,
    prelude::*,
    style::Color,
    widgets::{block::Position, Block, Borders, Padding, ScrollbarState}  
>>>>>>> Stashed changes
};

use std::io::{self, StdoutLock};

<<<<<<< Updated upstream
pub struct ChatUi {
    term: Terminal<CrosstermBackend<Stdout>>,
    exit: bool,
    pub message_window: MessagesWindow,
=======
use crate::schema::Message;

pub async fn chat_ui() -> io::Result<()> {
    let term = term_init()?;

    term_deinit(term)?;
    Ok(())
>>>>>>> Stashed changes
}

fn term_init() -> io::Result<Terminal<CrosstermBackend<StdoutLock>>> {
    let stdout = io::stdout().lock();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.clear()?;

<<<<<<< Updated upstream
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
=======
    Ok(terminal)
>>>>>>> Stashed changes
}

fn term_deinit(term: Terminal<CrosstermBackend<StdoutLock>>) -> io::Result<()> {
    execute!(term.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

#[derive(Default)]
struct ScrollBar {
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
}

struct MessagesBox {
    messages: Vec<Message>,
    room_id: String,
    style: Style,
    scrollbar: ScrollBar,
}

impl MessagesBox {
    fn new(bgroud: Color, fgroud: Color) -> Self {
        Self {
            messages: vec![],
            room_id: String::new(),
            style: Style::default().bg(bgroud.into()).fg(fgroud.into()),
            scrollbar: ScrollBar::default(),
        }
    }

    async fn push_msg(&mut self, msg: &Message) {
        self.messages.push(msg.clone());
    }

    fn change_style(&mut self, bgroud: Color, fgroud: Color) {
        self.style = Style::default().bg(bgroud.into()).fg(fgroud.into());
    }

    fn scroll_down(&mut self) {
        self.scrollbar.vertical_scroll = self.scrollbar.vertical_scroll.saturating_add(1);
        self.scrollbar
            .vertical_scroll_state
            .position(self.scrollbar.vertical_scroll);
    }

    fn scroll_up(&mut self) {
        self.scrollbar.vertical_scroll = self.scrollbar.vertical_scroll.saturating_sub(1);
        self.scrollbar
            .vertical_scroll_state
            .position(self.scrollbar.vertical_scroll);
    }
}

<<<<<<< Updated upstream
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
=======
impl Widget for MessagesBox {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        const KEYS: Line<'_> = Line::from(vec![
            "quit <ctrl+x>  ".into(),
            "help <ctrl+h>".into(),
            "list users <ctrl+l>  ".into(),
            "next/prev page <ctrl+n/p>  ".into(),
            "command </>".into(),
        ])
        .alignment(Alignment::Right); // how to set this at the bottom

        let title = Line::from(self.room_id).alignment(Alignment::Left);

        let block = Block::new()
            .title(title)
            .title_position(Position::Top)
            .title(KEYS)
            .borders(Borders::ALL)
            .padding(Padding::new(0, 0, 0, 0))
            .style(self.style);

        let messages = self.messages.iter().map(|msg| {
            (Text::from(msg.sender_id).style(Style::new().fg(msg.sender_color)), Text::)
        }).collect::<Vec<(Text, Text)>>();


        let messages = Paragraph::new(self.messages.iter().map(|msg|{Text::from(msg.content)}))
        block.render(area, buf);
>>>>>>> Stashed changes
    }
}
