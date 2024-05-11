use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::widgets::{Block, Borders, Padding, ScrollbarState};
use ratatui::{
    layout::*,
    prelude::*,
    style::Style,
    widgets::{block::Position, *},
};
use std::{
    io::{self, Stdout},
    str::FromStr,
    time::{Duration, Instant},
};
use tui_textarea::{Input, Key, TextArea};

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.clear()?;

    let mut FG: &str = "#ebbcba";
    let mut BG: &str = "#191724";
    let mut MSG_FG: &str = "#e0def4";

    let mut list_state = ListState::default();

    let mut messages = Vec::<Message>::new();

    let mut textarea = TextArea::default();
    textarea.set_style(Style::default().fg(Color::from_str(MSG_FG).unwrap()));
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(
        Style::default()
            .fg(Color::from_str(MSG_FG).unwrap())
            .bg(Color::from_str(MSG_FG).unwrap()),
    );
    textarea.set_block(
        Block::default().borders(Borders::ALL).set_style(
            Style::default()
                .fg(Color::from_str(FG).unwrap())
                .bg(Color::from_str(BG).unwrap()),
        ),
    );

    loop {
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
                .split(frame.size());

            let formatted_msgs = messages.iter().map(|msg| {
                Text::from(format!("[{}]: {}", msg.sender, msg.content))
                    .set_style(Style::default().fg(Color::from_str(MSG_FG).unwrap()))
            });
            let msgs_list = List::new(formatted_msgs)
                .block(Block::default().title("someroom").borders(Borders::ALL))
                .style(
                    Style::default()
                        .bg(Color::from_str(BG).unwrap())
                        .fg(Color::from_str(FG).unwrap()),
                )
                .direction(ListDirection::TopToBottom);

            frame.render_stateful_widget(msgs_list, layout[0], &mut list_state);
            frame.render_widget(textarea.widget(), layout[1]);
        })?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            match event::read()?.into() {
                Input {
                    key: Key::Char('p'),
                    ctrl: true,
                    ..
                } => {
                    previous(&mut list_state, messages.len());
                }
                Input {
                    key: Key::Char('n'),
                    ctrl: true,
                    ..
                } => {
                    next(&mut list_state, messages.len());
                }
                Input {
                    key: Key::Char('q'),
                    ctrl: true,
                    ..
                } => break,
                Input {
                    key: Key::Enter, ..
                } => {
                    let lines = textarea.lines()[0].clone();
                    textarea.delete_line_by_head();
                    if lines == "change" {
                        BG = "#e0def4";
                        FG = "#f6c177";
                        MSG_FG = "#f6c177";
                    } else {
                        messages.push(Message {
                            sender: "me".into(),
                            content: lines,
                        });
                        list_state.select(Some(messages.len()));
                    }
                }
                input => {
                    textarea.input(input);
                }
            }
        }
    }

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

struct Message {
    sender: String,
    content: String,
}

pub fn next(state: &mut ListState, len: usize) {
    let i = match state.selected() {
        Some(i) => {
            if i >= len - 1 {
                0
            } else {
                i + 1
            }
        }
        None => 0,
    };
    state.select(Some(i));
}

pub fn previous(state: &mut ListState, len: usize) {
    let i = match state.selected() {
        Some(i) => {
            if i == 0 {
                len - 1
            } else {
                i - 1
            }
        }
        None => 0,
    };
    state.select(Some(i));
}
