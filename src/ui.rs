use crossterm::{
    event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{layout::*, prelude::*, style::Style, widgets::*};
use regex::Regex;
use std::{borrow::BorrowMut, io, str::FromStr, time::Duration, usize, vec};
use tui_pattern_highlighter::{highlight_line, highlight_text};
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
    _ = textarea.set_search_pattern(r"@\w+");

    let mut banned_user = String::new();
    let mut counter = 100;

    let mut msg_height = 0;
    let mut width = 0;
    const MAX_HEIGHT: u16 = 20;

    let mut line_temp = 0;

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

            let msgs_list = List::new(
                messages
                    .iter()
                    .map(|msg| {
                        let name = Line::from(msg.sender.clone()).bold();
                        let content = highlight_text(
                            &msg.content,
                            r"@(\w+)",
                            Style::new().bg(Color::LightBlue),
                        );
                        let mut text = Text::from(name);
                        content
                            .lines
                            .iter()
                            .for_each(|line| text.push_line(line.clone().italic()));
                        text
                    })
                    .collect::<Vec<Text>>(),
            )
            .block(
                Block::default()
                    .title(format!("line {} width {}", line_temp, width))
                    .borders(Borders::ALL),
            )
            .style(
                Style::default()
                    .bg(Color::from_str(BG).unwrap())
                    .fg(Color::from_str(FG).unwrap()),
            )
            .direction(ListDirection::TopToBottom)
            .highlight_style(Style::default().fg(Color::Yellow));

            let help_content: Text =
                Text::from("<ctrl-k> highlight next message\n<ctrl-j> highlight previous message");
            let help_popup = Popup::new("help", help_content).style(
                Style::new()
                    .bg(Color::from_str(BG).unwrap())
                    .fg(Color::from_str(FG).unwrap()),
            );

            let banned_popup = Popup::new(
                "banned user",
                Text::from(format!("{} has been removed!", banned_user)),
            )
            .style(
                Style::new()
                    .bg(Color::from_str(BG).unwrap())
                    .fg(Color::from_str(FG).unwrap()),
            );

            frame.render_stateful_widget(msgs_list, layout[0], &mut list_state);
            frame.render_widget(textarea.widget(), layout[1]);

            match popup_state {
                PopupShow::Help => frame.render_widget(&help_popup, frame.size()),
                PopupShow::List => {}
                PopupShow::Banned => {
                    counter -= 1;
                    if counter != 0 {
                        frame.render_widget(&banned_popup, frame.size());
                    } else {
                        popup_state = PopupShow::None;
                        counter = 70;
                    }
                }
                PopupShow::None => {}
            }
        })?;

        if crossterm::event::poll(Duration::from_millis(10))? {
            match event::read()?.into() {
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
                            if !line_.is_empty() && line_ != *textarea.lines().last().unwrap() {
                                line_.push('\n');
                            }
                            line_
                        })
                        .collect();

                    let ban_pattern = Regex::new(r"^/ban\s+(\w+)$").unwrap();
                    if ban_pattern.is_match(&lines.trim()) {
                        banned_user = lines[5..].to_string();
                        popup_state = PopupShow::Banned;
                    }

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
                    if popup_state == PopupShow::Help {
                        popup_state = PopupShow::None;
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
                    if popup_state == PopupShow::Help {
                        popup_state = PopupShow::None;
                    } else if popup_state == PopupShow::None {
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
                // Input {
                //      key: Key::Char(' '),
                //      ..
                //  } => {
                //      if !textarea.is_empty() {
                //          textarea.insert_char(' ');
                //      }
                //  }
                input => {
                    if textarea.input(input) {
                        let ta = textarea.borrow_mut();
                        let lines = &ta.lines()[ta.cursor().0];
                        line_temp = lines.len();

                        if lines.len() >= (width - 2).into() {
                            let rlines: String = lines.chars().rev().collect();
                            if let Some(caps) = Regex::new(r"\S+").unwrap().captures(&rlines) {
                                let cap = caps.get(0).unwrap();
                                if cap.start() == 0 {
                                    ta.delete_word();
                                    ta.insert_newline();
                                    let rword: String = cap.as_str().chars().rev().collect();
                                    ta.insert_str(&rword);
                                } else {
                                    ta.move_cursor(CursorMove::Back);
                                }
                            }

                            if msg_height <= MAX_HEIGHT {
                                msg_height += 2;
                            }
                        }
                        if popup_state != PopupShow::None {
                            popup_state = PopupShow::None;
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
