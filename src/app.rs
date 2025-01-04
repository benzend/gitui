use itertools::Itertools;
use ratatui::text::Text;

pub struct App {
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub list_branches_modal: Modal,
    pub list_commands_modal: Modal,
    pub list_branch_commands_modal: Modal,
    pub in_search_bar: bool,
    pub search_query: String,
    pub error_modal: Modal,
    pub errors: Vec<GituiError>,
    pub branches: Branches,
    pub command_chain: Vec<Command>,
    pub commands: Vec<Command>,
    pub branch_commands: Vec<BranchCommand>
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Main,
            list_branches_modal: Modal::Closed,
            list_commands_modal: Modal::Closed,
            list_branch_commands_modal: Modal::Closed,
            in_search_bar: false,
            search_query: String::from(""),
            error_modal: Modal::Closed,
            errors: Vec::new(),
            branches: Branches::new(vec![]),
            command_chain: Vec::new(),
            commands: vec![Command::Branch, Command::Fetch],
            branch_commands: vec![BranchCommand::Checkout, BranchCommand::Switch, BranchCommand::FastForward]
        }
    }
}

pub enum CurrentScreen {
    Main,
    ListingBranches,
    Errors,
    Exiting,
}

pub enum Command {
    Branch,
    Fetch
}

pub enum BranchCommand {
    Checkout,
    Switch,
    FastForward,
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
}

impl From<&Branch> for Branch {
    fn from(branch: &Branch) -> Branch {
        Branch::new(branch.name.as_str(), branch.is_checked_out)
    }
}

#[derive(Debug)]
pub struct IndexedBranch {
    pub name: String,
    pub is_checked_out: bool,
    pub index: usize,
}

impl IndexedBranch {
    pub fn new(name: &str, is_checked_out: bool, index: usize) -> Self {
        Self {
            name: name.to_string(),
            is_checked_out,
            index
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
            format!("* {}", self.get_name())
        } else {
            self.get_name()
        }
    }

    pub fn get_name(&self) -> String {
        self.name.to_string()
    }

    pub fn set_is_checked_out(&mut self, value: bool) {
        self.is_checked_out = value;
    }
}

impl From<&IndexedBranch> for IndexedBranch {
    fn from(branch: &IndexedBranch) -> IndexedBranch {
        IndexedBranch::new(branch.name.as_str(), branch.is_checked_out, branch.index)
    }
}

pub struct Branches {
    values: Vec<IndexedBranch>,
    curr_index: usize
}

impl Branches {
    pub fn new(branches: Vec<Branch>) -> Self {
        let mut indexed: Vec<IndexedBranch> = Vec::new();
        for (i, b) in branches.iter().enumerate() {
            indexed.push(IndexedBranch::new(&b.name, b.is_checked_out, i));
        }

        Self {
            values: indexed,
            curr_index: 0
        }
    }

    pub fn get_values(&self) -> &Vec<IndexedBranch> {
        &self.values
    }

    pub fn get_index(&self) -> usize {
        self.curr_index
    }

    pub fn reset_index(&mut self) {
        self.curr_index = 0;
    }

    pub fn filtered(&self, query: &str) -> Branches {
        let mut branches = Vec::new();
        for b in self.values.iter() {
            if query != "" {
                if b.name.contains(query) {
                    branches.push(IndexedBranch::from(b));
                }
            } else {
                branches.push(IndexedBranch::from(b));
            }
        }
        Branches {
            values: branches,
            curr_index: self.get_index(),
        }
    }

    pub fn select_from_index(&mut self, index: usize) -> &IndexedBranch {
        self.curr_index = index;
        &self.values[self.curr_index]
    }

    pub fn get_currently_checkedout_name(&self) -> Option<String> {
        if let Some(b) = self.values.iter().find(|b| b.is_checked_out) {
            Some(b.name.to_string())
        } else {
            None
        }
    }

    pub fn checkout_current(&mut self) -> Result<(), GituiError> {
        self.values[self.curr_index].checkout()?;

        let current_branch_name = &self.values[self.curr_index].name;

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

impl<S, I> From<&Branches> for Scrollable<S, I>
where
    S: Into<Text<'static>>,
    I: Into<Index>,
    Vec<(S, I)>: FromIterator<(Text<'static>, Index)>
{
    fn from(branches: &Branches) -> Scrollable<S, I> {
        Scrollable::new(branches.values.iter().map(|b| (Text::from(b.get_display_name()), Index(b.index))).collect(), Some(branches.curr_index))
    }
}

pub struct Scrollable<S, I>
where
    S: Into<Text<'static>>,
    I: Into<Index>
{
    items: Vec<(S, I)>,
    selection: usize
}

pub struct Index(pub usize);

impl<S, I> Scrollable<S, I>
where
    S: Into<Text<'static>>,
    I: Into<Index>
{
    pub fn new(items: Vec<(S, I)>, index: Option<usize>) -> Self {
        Self {
            items,
            selection: index.unwrap_or(0),
        }
    }

    pub fn next(&mut self) -> Option<&(S, I)> {
        if self.is_empty() {
            return None;
        }
        if self.selection_invalid() || self.is_last() {
            self.selection = 0;
        } else {
            self.selection += 1;
        }

        Some(&self.items[self.selection])
    }

    pub fn prev(&mut self) -> Option<&(S, I)> {
        if self.is_empty() {
            return None;
        }
        if self.selection_invalid() || self.is_first() {
            self.selection = self.items.len() - 1;
        } else {
            self.selection -= 1;
        }

        Some(&self.items[self.selection])
    }

    pub fn is_last(&self) -> bool {
        if self.is_empty() {
            return true;
        }
        self.selection == self.items.len() - 1
    }

    fn selection_invalid(&self) -> bool {
        self.selection >= self.items.len()
    }

    pub fn is_first(&self) -> bool {
        self.selection == 0
    }

    pub fn is_empty(&self) -> bool {
        self.items.len() == 0
    }

}
