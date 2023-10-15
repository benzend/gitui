pub struct App {
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub list_branches_modal: Option<Modal>,
    pub error_modal: Option<Modal>,
    pub errors: Option<Vec<GituiError>>,
    pub branches: Option<BranchIterator>,
    pub selected_branch: Option<Branch>,
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Main,
            list_branches_modal: None,
            error_modal: None,
            errors: None,
            selected_branch: None,
            branches: None,
        }
    }
}

pub enum CurrentScreen {
    Main,
    ListingBranches,
    Errors,
    Exiting,
}

pub enum Modal {
    Open,
    Closed,
}

pub enum GituiError {
    BranchCheckout(String),
}

impl std::fmt::Display for GituiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GituiError::BranchCheckout(s) => write!(f, "{}", s),
        }
    }
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

    pub fn checkout(&mut self) -> Result<(), GituiError> {
        if self.is_checked_out {
            return Err(GituiError::BranchCheckout(
                "branch is already checked out".to_string(),
            ));
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
            Err(GituiError::BranchCheckout(format!(
                "failed to checkout branch. output: {}",
                msg
            )))
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

    pub fn get_current_name(&self) -> String {
        self.values[self.index].name.to_string()
    }

    pub fn get_currently_checkedout_name(&self) -> Option<String> {
        if let Some(b) = self.values.iter().find(|b| b.is_checked_out) {
            Some(b.name.to_string())
        } else {
            None
        }
    }

    pub fn checkout_current(&mut self) -> Result<(), GituiError> {
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
