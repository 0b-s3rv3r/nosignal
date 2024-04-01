use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    style::Stylize,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    style::Stylize,
    symbols::border,
    widgets::{
        block::{Position, Title},
        canvas::Line,
        *,
    },
};
use std::io::{self, stdout, Stdout};

pub struct ChatUi {
    term: Terminal<CrosstermBackend<Stdout>>,
    exit: bool,
}

impl ChatUi {
    pub fn init() -> io::Result<Self> {
        execute!(stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;

        Ok(ChatUi {
            term: Terminal::new(CrosstermBackend::new(stdout())).unwrap(),
            exit: false,
        })
    }

    pub fn run(&mut self) -> io::Result<()> {
        while !self.exit {
            self.term.draw(|frame| {
                self.render_frame(frame);
            })?;
            self.handle_events()?;
        }

        Ok(())
    }

    pub fn restore() -> io::Result<()> {
        execute!(stdout(), LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) -> io::Result<()> {
        frame.render_widget(self, frame.size());
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {}
}

struct MessagesWindow;
struct InputBox;

impl Widget for &ChatUi {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(Span::raw("kioto"));
        let keys = [
            Span::raw("Quit "),
            Span::styled(
                "<ctrl+x>",
                Style::default().fg(Color::Blue).bg(Color::White),
            ),
            Span::raw("Chat Commands"),
            Span::styled("/", Style::default().fg(Color::Blue).bg(Color::White)),
        ];
        let instructions = Line::from(keys);
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Left)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);
    }
}
