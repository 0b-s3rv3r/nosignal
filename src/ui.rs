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
    collections::btree_map::Keys,
    io::{self, Stdout},
    str::FromStr,
    time::{Duration, Instant},
    usize,
};
use tui_textarea::{CursorMove, Input, Key, TextArea};

#[derive(PartialEq, Eq)]
enum PopupShow {
    Help,
    List,
    None,
}

enum ChatCommand {
    Set(String),
    Ban(String),
}

fn ui() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.clear()?;

    let mut FG: &str = "#ebbcba";
    let mut BG: &str = "#191724";
    let mut MSG_FG: &str = "#e0def4";

    let mut list_state = ListState::default();
    let mut popup_state = PopupShow::None;

    let mut messages = Vec::<Message>::new();
    let mut textarea = TextArea::default();
    textarea.set_style(Style::default().fg(Color::from_str(MSG_FG).unwrap()));
    textarea.set_cursor_line_style(Style::default());
    textarea.set_cursor_style(
        Style::default()
            .fg(Color::from_str(MSG_FG).unwrap())
            .bg(Color::from_str(MSG_FG).unwrap())
            .slow_blink(),
    );
    textarea.set_block(
        Block::default().borders(Borders::ALL).set_style(
            Style::default()
                .fg(Color::from_str(FG).unwrap())
                .bg(Color::from_str(BG).unwrap()),
        ),
    );

    const HELP_CONTENT: &str =
        "<ctrl-n> highlight next message\n<ctrl-p> highlight previous message";
    let help_popup =
        Paragraph::new(HELP_CONTENT).block(Block::new().borders(Borders::ALL).title("help"));

    let mut msg_height = 0;
    let mut width = 0;
    const MAX_HEIGHT: u16 = 20;

    loop {
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(90 - msg_height),
                    Constraint::Percentage(10 + msg_height),
                ])
                .split(frame.size());
            width = layout[1].width;

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
                .direction(ListDirection::TopToBottom)
                .highlight_style(Style::default().fg(Color::Yellow));

            frame.render_stateful_widget(msgs_list, layout[0], &mut list_state);
            frame.render_widget(textarea.widget(), layout[1]);
            match popup_state {
                PopupShow::Help => {
                    frame.render_widget(&help_popup, centered_rect(60, 25, frame.size()))
                }
                PopupShow::List => {}
                PopupShow::None => {}
            }
        })?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            match event::read()?.into() {
                Input {
                    key: Key::Char('k'),
                    ctrl: true,
                    ..
                } => {
                    previous(&mut list_state);
                }
                Input {
                    key: Key::Char('j'),
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
                    if popup_state != PopupShow::None {
                        popup_state = PopupShow::None;
                    }

                    let lines: String = textarea
                        .lines()
                        .iter()
                        .map(|line| {
                            let mut line_ = line.to_string();
                            line_.push('\n');
                            line_
                        })
                        .collect();
                    if !textarea.is_empty() && {
                        let temp = no_space(&lines);
                        !temp.is_empty()
                    } {
                        textarea.move_cursor(CursorMove::End);
                        for _ in 0..textarea.lines().len() {
                            textarea.delete_line_by_head();
                            textarea.delete_newline();
                        }
                        messages.push(Message {
                            sender: "me".into(),
                            content: lines,
                        });
                        list_state.select(Some(messages.len()));
                    }

                    msg_height = 0;
                }
                Input {
                    key: Key::Char('l'),
                    ctrl: true,
                    ..
                } => {}
                Input {
                    key: Key::Char('h'),
                    ctrl: true,
                    ..
                } => {
                    if popup_state == PopupShow::None {
                        popup_state = PopupShow::Help
                    } else {
                        popup_state = PopupShow::None
                    }
                }
                Input {
                    key: Key::Char('y'),
                    ctrl: true,
                    ..
                } => textarea.copy(),
                Input {
                    key: Key::Char('p'),
                    ctrl: true,
                    ..
                } => {
                    let _ = textarea.paste();
                }
                input => {
                    if textarea.input(input) {
                        if popup_state != PopupShow::None {
                            popup_state = PopupShow::None;
                        }

                        if textarea.lines()[textarea.cursor().0].len() == (width - 2).into() {
                            textarea.insert_newline();
                            if msg_height <= MAX_HEIGHT {
                                msg_height += 2;
                            }
                        }
                    }
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
                i
            } else {
                i + 1
            }
        }
        None => 0,
    };
    state.select(Some(i));
}

pub fn previous(state: &mut ListState) {
    let i = match state.selected() {
        Some(i) => {
            if i == 0 {
                i
            } else {
                i - 1
            }
        }
        None => 0,
    };
    state.select(Some(i));
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn no_space(x: &str) -> String {
    x.replace(" ", "")
}
