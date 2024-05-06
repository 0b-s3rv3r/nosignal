use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{layout::*, prelude::*, widgets::block::Position, widgets::*};
use ratatui::{
    style::Color as rColor,
    widgets::{Block, Borders, Padding, ScrollbarState},
};
use std::{
    io::{self, Stdout},
    simd::SimdElement,
    time::{Duration, Instant},
};
use tui_textarea::{Input, Key, TextArea};

use crate::schema::{Color, Message};

struct ChatUi {
    terminal: Option<Terminal<CrosstermBackend<Stdout>>>,
    tick_rate: Duration,
    msgsbox: MessagesBox,
    msginput: MessageInput,
    style: (Color, Color),
}

impl ChatUi {
    pub fn new(style: (Color, Color)) -> Self {
        Self {
            terminal: None,
            tick_rate: Duration::from_millis(250),
            msgsbox: MessagesBox::new(style.0, style.1),
            msginput: MessageInput::new(style.0, style.1),
            style: style,
        }
    }
    pub async fn run(&mut self) -> io::Result<()> {
        self.term_init()?;
        self.render()?;
        self.term_deinit()?;
        Ok(())
    }

    fn render(&mut self) -> io::Result<()> {
        let last_tick = Instant::now();

        loop {
            self.terminal.unwrap().draw(|frame| {
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
                    .split(frame.size());

                frame.render_widget(MessagesBox::new(self.style.0, self.style.1), layout[0]);
                frame.render_widget(MessageInput::new(self.style.0, self.style.1), layout[1]);
            })?;

            let timeout = self.tick_rate.saturating_sub(last_tick.elapsed());
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('j') | KeyCode::Down => {
                            self.msgsbox.scrollbar.vertical_scroll =
                                self.msgsbox.scrollbar.vertical_scroll.saturating_add(1);
                            self.msgsbox.scrollbar.vertical_scroll_state = self
                                .msgsbox
                                .scrollbar
                                .vertical_scroll_state
                                .position(self.msgsbox.scrollbar.vertical_scroll);
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            self.msgsbox.scrollbar.vertical_scroll =
                                self.msgsbox.scrollbar.vertical_scroll.saturating_sub(1);
                            self.msgsbox.scrollbar.vertical_scroll_state = self
                                .msgsbox
                                .scrollbar
                                .vertical_scroll_state
                                .position(self.msgsbox.scrollbar.vertical_scroll);
                        }
                        _ => {}
                    }
                }
            }
            if last_tick.elapsed() >= self.tick_rate {
                last_tick = Instant::now();
            }

            match crossterm::event::read()?.into() {
                Input { key: Key::Esc, .. } => break,
                input => {
                    todo!();
                    // if textarea.input(input) {
                    // When the input modified its text, validate the text content
                    // }
                }
            }
        }

        Ok(())
    }

    fn term_init(&self) -> io::Result<()> {
        let stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        terminal.clear()?;
        Ok(())
    }

    fn term_deinit(&self) -> io::Result<()> {
        execute!(self.terminal.unwrap().backend_mut(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
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
    pub scrollbar: ScrollBar,
}

impl MessagesBox {
    fn new(bg: Color, fg: Color) -> Self {
        Self {
            messages: vec![],
            room_id: String::new(),
            style: Style::default().bg(bg.into()).fg(fg.into()),
            scrollbar: ScrollBar::default(),
        }
    }

    async fn push_msg(&mut self, msg: &Message) {
        self.messages.push(msg.clone());
    }

    fn change_style(&mut self, bg: Color, fg: Color) {
        self.style = Style::default().bg(bg.into()).fg(fg.into());
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

        let messages = self
            .messages
            .iter()
            .map(|msg| {
                (
                    Text::from(msg.sender_id).style(Style::new().fg(msg.sender_color.into())),
                    Text::from(msg.content),
                )
            })
            .collect::<Vec<(Text, Text)>>();

        let paragraph = Paragraph::new("some").block(Block::new());
        block.render(area, buf);
    }
}

struct MessageInput {
    message: String,
    style: Style,
}

impl MessageInput {
    pub fn new(fg: Color, bg: Color) -> Self {
        Self {
            message: String::new(),
            style: Style::default().fg(fg.into()).bg(bg.into()),
        }
    }
}

impl Widget for MessageInput {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut textarea = TextArea::default();
        textarea.set_cursor_style(Style::default());
        textarea.set_style(self.style);
        textarea.set_block(Block::default().borders(Borders::ALL));
        textarea.widget().render(area, buf);
    }
}
