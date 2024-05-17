use crossterm::{
    event::{self, read, Event, KeyCode, KeyEventKind},
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
use regex::Regex;
use std::{
    collections::btree_map::Keys,
    io::{self, Stdout},
    result,
    str::FromStr,
    thread::sleep,
    time::{Duration, Instant},
    usize, vec,
};
use tui_popup::Popup;
use tui_textarea::{CursorMove, Input, Key, TextArea};

#[derive(PartialEq, Eq)]
enum PopupShow {
    Help,
    List,
    Banned,
    None,
}

enum ChatCommand {
    Set(String),
    Ban(String),
}

pub fn ui() -> io::Result<()> {
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

    let HELP_CONTENT: Text =
        Text::from("<ctrl-k> highlight next message\n<ctrl-j> highlight previous message");
    let help_popup = Popup::new("help", HELP_CONTENT).style(
        Style::new()
            .bg(Color::from_str(BG).unwrap())
            .fg(Color::from_str(FG).unwrap()),
    );

    let mut banned_user = String::new();
    let banned_popup = Popup::new("", Text::from(banned_user)).style(
        Style::new()
            .bg(Color::from_str(BG).unwrap())
            .fg(Color::from_str(FG).unwrap()),
    );

    let mut msg_height = 0;
    let mut width = 0;
    const MAX_HEIGHT: u16 = 20;

    loop {
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(90 - msg_height),
                    Constraint::Percentage(9 + msg_height),
                ])
                .split(frame.size());

            width = layout[1].width;

            let formatted_msgs: Vec<Line> = messages
                .iter()
                .map(|msg| {
                    let mut formatted_content = Vec::<Span>::new();
                    let mut last_index = 0;
                    let reg = Regex::new(r"@[\w]+(?:\s|$)").unwrap();
                    for m in reg.find_iter(&msg.content) {
                        if last_index != m.start() {
                            formatted_content.push(
                                Span::from(&msg.content[last_index..m.start()])
                                    .set_style(Style::new().fg(Color::from_str(MSG_FG).unwrap())),
                            );
                        }
                        formatted_content.push(
                            Span::from(m.as_str())
                                .set_style(Style::new().bg(Color::LightYellow).fg(Color::Black)),
                        );
                        last_index = m.end();
                    }
                    if last_index < msg.content.len() {
                        formatted_content.push(
                            Span::from(&msg.content[last_index..])
                                .fg(Color::from_str(MSG_FG).unwrap()),
                        );
                    }

                    let mut line = Line::from(vec![
                        Span::styled(
                            msg.sender.clone(),
                            Style::new().fg(Color::from_str(MSG_FG).unwrap()).bold(),
                        ),
                        Span::from(" "),
                    ]);
                    formatted_content
                        .iter()
                        .for_each(|span| line.push_span(span.clone()));

                    line
                })
                .collect();
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
                PopupShow::Help => frame.render_widget(&help_popup, frame.size()),
                PopupShow::List => {}
                PopupShow::Banned => {
                    frame.render_widget(&banned_popup, frame.size());
                    sleep(Duration::from_secs(2));
                    popup_state = PopupShow::None;
                }
                PopupShow::None => {}
            }
        })?;

        if crossterm::event::poll(Duration::from_millis(50))? {
            let key: Input = event::read()?.into();
            if let Input {
                key: Key::Char(_) | Key::Enter,
                ..
            } = key
            {
                if popup_state != PopupShow::None {
                    popup_state = PopupShow::None;
                }
            }
            match key {
                Input {
                    key: Key::Char('k'),
                    ctrl: true,
                    ..
                } => {
                    previous(&mut list_state, messages.len());
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
                    key: Key::Enter,
                    ctrl: false,
                    ..
                } => {
                    let lines: String = textarea
                        .lines()
                        .iter()
                        .map(|line| {
                            let mut line_ = line.to_string();
                            if !line_.is_empty() {
                                line_.push('\n');
                            }
                            line_
                        })
                        .collect();

                    let ban_pattern = Regex::new(r"^/ban\s+(\w+)$").unwrap();
                    match ban_pattern.captures(&lines) {
                        Some(captures) => {
                            if let Some(user_id) = captures.get(0) {
                                banned_user = user_id.as_str().into();
                                popup_state = PopupShow::Banned;
                            }
                        }
                        None => {
                            if !textarea.is_empty() {
                                for _ in 0..textarea.lines().len() {
                                    textarea.move_cursor(CursorMove::End);
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
                    }
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
                    if popup_state != PopupShow::None {
                        popup_state = PopupShow::None;
                    } else {
                        popup_state = PopupShow::Help
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
                Input {
                    key: Key::Char(' '),
                    ..
                } => {
                    if !textarea.is_empty() {
                        textarea.insert_char(' ');
                    }
                }
                input => {
                    if textarea.input(input) {
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
    if len != 0 {
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
}

pub fn previous(state: &mut ListState, len: usize) {
    if len != 0 {
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
}
