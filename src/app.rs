use serde_json::Result;
use std::collections::HashMap;

pub enum CurrentScreen {
    Main,
    Editing,
    Exiting,
}

pub enum Modal {
    Open,
    Closed
}

pub struct Branch {
    name: String
}

impl Branch {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

pub struct App {
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub list_branches_modal: Option<Modal>,
    pub branches: Option<Vec<Branch>>,
    pub selected_branch: Option<Branch>,
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Main,
            list_branches_modal: None,
            selected_branch: None,
            branches: None,
        }
    }

    pub fn toggle_branches_modal_open(&mut self) {
        if let Some(is_open) = &self.list_branches_modal {
            match is_open {
                Modal::Open => self.list_branches_modal = Some(Modal::Closed),
                Modal::Closed => self.list_branches_modal = Some(Modal::Open)
            }
        }
    }
}
