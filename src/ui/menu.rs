use super::{macros::*, UIError};
use std::cell::Ref;
use std::collections::LinkedList as Stack;
use std::str::FromStr;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use tui::widgets::ListState;

type Children = Vec<Rc<Box<MenuList>>>;

struct MenuList {
    name: &'static str,
    children: Option<Children>,
}

impl MenuList {
    pub fn get_name(&self) -> &'static str {
        self.name
    }

    #[inline(always)]
    pub fn is_final(&self) -> bool {
        self.children.is_none()
    }

    #[inline(always)]
    pub fn generate_final(name: &'static str) -> Self {
        Self {
            name,
            children: None,
        }
    }

    #[inline(always)]
    pub fn generate_non_final(name: &'static str, children: Children) -> Self {
        Self {
            name,
            children: Some(children),
        }
    }

    pub fn child_number(&self) -> Option<usize> {
        if let Some(children) = &self.children {
            Some(children.len())
        } else {
            None
        }
    }

    pub fn debug_print(&self) {
        if let Some(children) = &self.children {
            println!(
                "I'm in node {} and it has {} children. Printing children.",
                self.name,
                self.children.as_ref().unwrap().len()
            );

            for child in children {
                child.debug_print();
            }

            println!("End of printing child of node {}.", self.name);
        } else {
            println!("I'm in node {} and it is final node.", self.name);
        }
    }

    pub fn new() -> Self {
        menu_list! {
            "Menu": {
                "Files...": {
                    "...Go back",
                    "Open database",
                    "Save database",
                    "Close database",
                },
                "Quit application"
            }
        }
    }
}

const DEEPER_ACTIONS: [&'static str; 1] = ["Files..."];

pub enum MenuActions {
    Tick,
    OpenDatabase,
    SaveDatabase,
    CloseDatabase,
    QuitApp,
}

impl FromStr for MenuActions {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Open database" => Ok(MenuActions::OpenDatabase),
            "Save database" => Ok(MenuActions::SaveDatabase),
            "Close database" => Ok(MenuActions::CloseDatabase),
            "Quit application" => Ok(MenuActions::QuitApp),
            _ => Err(format!("Unable to parse {} into MenuActions.", s)),
        }
    }
}

pub struct MainMenu {
    list: Rc<Box<MenuList>>,
    list_state: ListState,
    actual_level: Weak<Box<MenuList>>,
    depth: Stack<(ListState, Weak<Box<MenuList>>)>,
}

impl MainMenu {
    pub fn new() -> Self {
        let menu_list = Rc::new(Box::new(MenuList::new()));
        let actual_level = Rc::downgrade(&menu_list);

        MainMenu {
            list: menu_list,
            list_state: ListState::default(),
            actual_level,
            depth: Stack::new(),
        }
    }

    pub fn next(&mut self) {
        let mut index = self.list_state.selected().unwrap_or(0);

        let children = self.actual_level.upgrade().unwrap().child_number().unwrap();

        if index + 1 >= children {
            index = 0;
        }

        self.list_state.select(Some(index));
    }

    pub fn previous(&mut self) {
        let mut index = self.list_state.selected().unwrap_or(0);

        let children = self.actual_level.upgrade().unwrap().child_number().unwrap();

        if index == 0 {
            index = children - 1;
        } else {
            index -= 1;
        }

        self.list_state.select(Some(index));
    }

    pub fn deeper(&mut self) {
        // TODO: Enforce picked position
        let target = self.list_state.selected().unwrap();

        let rc_level = self.actual_level.upgrade().unwrap();
        let target_option = &rc_level.children.as_ref().unwrap()[target];

        if target_option.is_final() {
            return;
        }

        let mut state = ListState::default();
        state.select(Some(0));
        std::mem::swap(&mut state, &mut self.list_state);

        let mut target = Rc::downgrade(target_option);
        std::mem::swap(&mut target, &mut self.actual_level);

        self.depth.push_back((state, target));
    }

    pub fn shallow(&mut self) {
        todo!();
    }

    pub fn clicked(&mut self) -> MenuActions {
        let target = self.list_state.selected().unwrap();

        let rc_level = self.actual_level.upgrade().unwrap();
        let target_option = &rc_level.children.as_ref().unwrap()[target];
        let target_name = target_option.get_name();

        if DEEPER_ACTIONS.contains(&target_name) {
            self.deeper();
            MenuActions::Tick
        } else if target_name == "...Go back" {
            self.shallow();
            MenuActions::Tick
        } else {
            target_name.parse().unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_matches_menu_actions() {
        let main_menu = MenuList::new();

        fn recursive_walk(menu: &MenuList) {
            if !(menu.name.starts_with("...") || menu.name.ends_with("...") || menu.name == "Menu")
            {
                let parsing_result: Result<MenuActions, _> = menu.name.parse();

                assert!(
                    parsing_result.is_ok(),
                    "Unable to parse menu name = {}",
                    menu.name
                );
            }

            if let Some(children) = &menu.children {
                for child in children {
                    recursive_walk(child);
                }
            }
        }

        recursive_walk(&main_menu);
    }
}
