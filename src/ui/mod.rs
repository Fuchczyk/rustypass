#[macro_use]
mod macros;
mod menu;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen, SetTitle,
    },
};

use tui::{
    backend::Backend,
    layout::{self, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, BorderType, List, ListItem},
    widgets::{Borders, ListState},
    Frame, Terminal,
};

use crate::manager::Task;
use menu::MainMenu;
use std::{
    sync::{mpsc::Receiver, mpsc::Sender},
    thread::JoinHandle,
};

pub struct UIState {
    menu_state: MainMenu,
    user_focus: Focus,
}

impl UIState {
    pub fn new() -> Self {
        UIState {
            menu_state: MainMenu::new(),
            user_focus: Focus::Menu,
        }
    }

    pub fn focus(&self) -> &Focus {
        &self.user_focus
    }

    pub fn menu_positions<'a>(&'a self) -> impl Iterator<Item = &'static str> + 'a {
        self.menu_state.position_iterator()
    }

    pub fn menu_position_title(&self) -> &'static str {
        self.menu_state.title()
    }

    pub fn menu_state(&mut self) -> &mut ListState {
        self.menu_state.menu_state()
    }
}

enum Focus {
    Menu,
}

#[derive(Debug)]
pub enum UIError {
    IOError(std::io::Error),
    ImpossibleAction,
}

impl From<std::io::Error> for UIError {
    fn from(error: std::io::Error) -> Self {
        Self::IOError(error)
    }
}

/// This should be non thread blocking function.
pub fn run_user_interface(backend_connector: Sender<Task>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        if let Err(e) = ui_main_function(backend_connector) {
            panic!("Unable to run user-interface. Error occurred = [{:?}]", e);
        }
    })
}

fn main_screen<B: Backend>(f: &mut Frame<B>, state: &mut UIState) {
    let layout = Layout::default()
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .direction(Direction::Horizontal)
        .split(f.size());

    let layout = Layout::default()
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .direction(Direction::Vertical)
        .split(layout[0]);

    let items: Vec<ListItem> = state
        .menu_positions()
        .map(|name| ListItem::new(name))
        .collect();

    let menu_box = Block::default()
        .borders(Borders::all())
        .border_style(Style::default().fg(Color::LightMagenta))
        .title(state.menu_position_title());

    let list = List::new(items)
        .highlight_symbol("-> ")
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::LightCyan),
        )
        .block(menu_box);

    f.render_stateful_widget(list, layout[0], state.menu_state());
}

fn ui_main_function(backend_connector: Sender<Task>) -> Result<(), UIError> {
    let mut stdout = std::io::stdout();

    execute!(
        stdout,
        Clear(ClearType::All),
        EnterAlternateScreen,
        SetTitle(env!("CARGO_PKG_NAME")),
        EnableMouseCapture
    )
    .expect("Unable to adjust settings of terminal");
    enable_raw_mode()?;

    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut render_interface = true;
    let mut state = UIState::new();

    // Main interface loop
    loop {
        terminal.draw(|frame| main_screen(frame, &mut state))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => {
                    state.menu_state.previous();
                }
                KeyCode::Down => {
                    state.menu_state.next();
                }
                KeyCode::Enter | KeyCode::Right => {
                    let action = state.menu_state.clicked();
                    println!("CLICKED ACTION = [{:?}]", action);

                    if action == menu::MenuActions::QuitApp {
                        break;
                    }
                }
                KeyCode::Left => {
                    state.menu_state.shallow();
                }
                _ => {}
            }
        }
    }

    let _ = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    disable_raw_mode()?;

    Ok(())
}

fn user_interface() -> Result<(), UIError> {
    let mut stdout = std::io::stdout();

    execute!(
        stdout,
        Clear(ClearType::All),
        EnterAlternateScreen,
        SetTitle(env!("CARGO_PKG_NAME")),
        EnableMouseCapture
    )
    .expect("Unable to adjust settings of terminal");
    enable_raw_mode()?;

    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let items = [ListItem::new("one"), ListItem::new("two")];
            let list = List::new(items)
                .highlight_style(
                    Style::default()
                        .fg(Color::LightCyan)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ")
                .repeat_highlight_symbol(true);
            let size = f.size();

            f.render_stateful_widget(list, size, &mut {
                let mut state = ListState::default();
                state.select(Some(1));

                state
            });
        })?;

        //crossterm::event::poll(timeout)
    }

    let _ = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    disable_raw_mode()?;

    Ok(())
}
