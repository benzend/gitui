pub struct App {
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub list_branches_modal: Option<Modal>,
    pub branches: Option<BranchIterator>,
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
                Modal::Closed => self.list_branches_modal = Some(Modal::Open),
            }
        }
    }
}

pub enum CurrentScreen {
    Main,
    ListingBranches,
    Exiting,
}

pub enum Modal {
    Open,
    Closed,
}

#[derive(Debug)]
pub struct Branch {
    pub name: String,
    pub is_checked_out: bool,
}

impl Branch {
    pub fn new(name: &str, is_checked_out: bool) -> Self {
        Self {
            name: name.to_string(),
            is_checked_out,
        }
    }

    pub fn checkout(&mut self) -> Result<(), String> {
        if self.is_checked_out {
            return Err("branch is already checked out".to_string());
        }
        let stdout = std::process::Command::new("git")
            .arg("checkout")
            .arg(&self.name.trim())
            .output()
            .expect("couldnt checkout branch")
            .stdout;

        let msg = String::from_utf8(stdout).expect("couldn't parse output");

        if !msg.contains("error:") {
            self.is_checked_out = true;
            Ok(())
        } else {
            Err(format!("failed to checkout branch. output: {}", msg))
        }
    }

    pub fn get_display_name(&self) -> String {
        if self.is_checked_out {
            format!("* {}", self.name)
        } else {
            self.name.to_string()
        }
    }

    pub fn set_is_checked_out(&mut self, value: bool) {
        self.is_checked_out = value;
    }
}

pub struct BranchIterator {
    pub values: Vec<Branch>,
    pub index: usize,
}

impl BranchIterator {
    pub fn new(branches: Vec<Branch>) -> Self {
        BranchIterator {
            values: branches,
            index: 0,
        }
    }

    pub fn next(&mut self) -> &Branch {
        if self.index == self.values.len() - 1 {
            self.index = 0;
        } else {
            self.index += 1;
        }

        &self.values[self.index]
    }

    pub fn prev(&mut self) -> &Branch {
        if self.index == 0 {
            self.index = self.values.len() - 1;
        } else {
            self.index -= 1;
        }

        &self.values[self.index]
    }

    pub fn checkout_current(&mut self) -> Result<(), String> {
        self.values[self.index].checkout()?;

        let current_branch_name = &self.values[self.index].name;

        self.uncheckout_all_except(current_branch_name.to_string());

        Ok(())
    }

    pub fn uncheckout_all_except(&mut self, name: String) {
        for b in self.values.iter_mut() {
            if name != b.name {
                b.set_is_checked_out(false)
            }
        }
    }
}
