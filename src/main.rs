extern crate sdl2;

use std::env;
use std::fmt::Write;
use std::io::Stdout;
use std::time::{Duration, SystemTime};

use crate::core::chip::Chip8;

mod audio;
mod core;
mod screen;

const FPS: u128 = 60;

use crossterm::event::{KeyCode, KeyEvent, Event};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::Rect;
use ratatui::widgets::block;
use ratatui::Frame;
use crossterm::event::KeyCode::{Char, Esc};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Terminal,
};
use std::{io, thread};

fn main() -> Result<(), io::Error> {
    // Init Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        layout::<CrosstermBackend<Stdout>>(f);
    })?;

    // Start a thread to discard any input events. Without handling events, the
    // stdin buffer will fill up, and be read into the shell when the program exits.
    thread::spawn(|| loop {
        println!("{:?}", event::read());
        if let Ok(new_event) = event::read() {
            match new_event {
                Event::Key(KeyEvent {code: Esc, ..}) => {println!("vai sair")},
                Event::Key(KeyEvent {code: Char('/'), ..}) => println!("click na barra"),
                _ => {}
            }
        };
    });

    thread::sleep(Duration::from_millis(5000));

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
};

fn layout<B: Backend>(f: &mut Frame<B>) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(f.size());

    let top_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(layout[0]);

    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);

    let block = Block::new().borders(Borders::ALL).title("Game");
    f.render_widget(block, top_layout[0]);

    let block = Block::new().borders(Borders::ALL).title("OpCodes");
    f.render_widget(block, top_layout[1]);

    let block = Block::new().borders(Borders::ALL).title("Terminal");
    f.render_widget(block, bottom_layout[0]);

    let block = Block::new().borders(Borders::ALL).title("State");
    f.render_widget(block, bottom_layout[1]);
}
