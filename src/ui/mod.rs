#[macro_use]
mod macros;
mod menu;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen, SetTitle,
    },
};

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, BorderType, List, ListItem},
    widgets::{Borders, ListState},
    Frame, Terminal,
};

struct MenuList {
    state: ListState,
    items: Vec<&'static str>,
}

struct MenuTree {
    name: &'static str,
    children: Option<Vec<Box<MenuTree>>>,
}

impl MenuTree {
    pub fn is_final(&self) -> bool {
        self.children.is_none()
    }

    fn generate_final(name: &'static str) -> Self {
        Self {
            name,
            children: None,
        }
    }

    fn files_menu() -> MenuTree {
        let options = ["Open database", "Save database", "Close without saving"]
            .map(MenuTree::generate_final);

        Self::generate_non_final("Files", options.into())
    }

    fn generate_non_final(name: &'static str, old_children: Vec<MenuTree>) -> MenuTree {
        let mut children = Vec::new();

        children.push(Box::new(MenuTree::generate_final("Go back")));

        for child in old_children {
            children.push(Box::new(child));
        }

        Self {
            name,
            children: Some(children),
        }
    }

    pub fn application_tree() -> Self {
        let mut vec_main = Vec::new();

        // Files menu
        vec_main.push(Box::new(Self::files_menu()));

        //Quit
        vec_main.push(Box::new(Self::generate_final("Quit")));

        Self {
            name: "Menu",
            children: Some(vec_main),
        }
    }
}

impl MenuList {
    fn new<T>(items: &T) -> Self
    where
        T: AsRef<[&'static str]>,
    {
        Self {
            state: ListState::default(),
            items: Vec::from(items.as_ref()),
        }
    }

    fn next(&mut self) {
        let index = match self.state.selected() {
            None => 0,
            Some(index) => (index + 1) % self.items.len(),
        };

        self.state.select(Some(index));
    }

    fn previous(&mut self) {
        let index = match self.state.selected() {
            None => 0,
            Some(index) => {
                if index == 0 {
                    self.items.len() - 1
                } else {
                    index - 1
                }
            }
        };

        self.state.select(Some(index));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

use tokio::sync::{mpsc::UnboundedReceiver, oneshot::Sender};

pub struct UIState {
    menu_state: MenuList,
    user_focus: Focus,
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

pub async fn user_interface() -> Result<(), UIError> {
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

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let _ = execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
    disable_raw_mode()?;

    Ok(())
}
