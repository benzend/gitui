pub struct App {
    pub current_screen: CurrentScreen, // the current screen the user is looking at, and will later determine what is rendered.
    pub in_search_bar: bool,
    pub search_query: String,
    pub error_modal: Modal,
    pub errors: Vec<GituiError>,
    pub branches: Branches,
    pub selected_command: Option<Command>,
    pub selected_branch_command: Option<BranchCommand>,
    pub commands: Scrollable,
    pub branch_commands: Scrollable,
}

impl App {
    pub fn new() -> App {
        App {
            current_screen: CurrentScreen::Main,
            in_search_bar: false,
            search_query: String::from(""),
            error_modal: Modal::Closed,
            errors: Vec::new(),
            branches: Branches::new(vec![]),
            selected_command: None,
            selected_branch_command: None,
            commands: Scrollable::new(
                vec![
                    (Command::Branch.to_string(), Index(0)),
                    (Command::FetchAll.to_string(), Index(1)),
                ],
                Some(0),
            ),
            branch_commands: Scrollable::new(
                vec![
                    (BranchCommand::Switch.to_string(), Index(0)),
                    (BranchCommand::Merge.to_string(), Index(1)),
                ],
                Some(0),
            ),
        }
    }
}

pub enum CurrentScreen {
    Main,
    ListingBranches,
    ListingCommands,
    ListingBranchCommands,
    Errors,
    Exiting,
}

pub enum Command {
    Branch,
    FetchAll,
}

impl Command {
    pub fn to_string(&self) -> String {
        String::from(match self {
            Command::Branch => "Branch",
            Command::FetchAll => "Fetch All",
        })
    }

    pub fn next_step(&self, app: &mut App) -> Result<(), GituiError> {
        match self {
            Command::FetchAll => {
                let stdout = std::process::Command::new("git")
                    .arg("fetch")
                    .arg("--all")
                    .output()
                    .expect("couldnt fetch")
                    .stdout;

                let msg = String::from_utf8(stdout).expect("couldn't parse output");

                if !msg.contains("error:") {
                    Ok(())
                } else {
                    Err(GituiError::FetchAll(format!(
                        "failed to fetch all. output: {}",
                        msg
                    )))
                }
            }
            Command::Branch => {
                app.current_screen = CurrentScreen::ListingBranchCommands;
                app.selected_command = Some(Command::Branch);

                Ok(())
            }
        }
    }
}

pub fn get_branches() -> Vec<Branch> {
    let stdout = std::process::Command::new("git")
        .arg("branch")
        .output()
        .expect("to get git branches")
        .stdout;

    String::from_utf8(stdout)
        .expect("couldnt parse stdout")
        .split("\n")
        .into_iter()
        .filter(|b| b.len() > 0)
        .map(|b| {
            let is_checked_out = b.contains("* ");
            let name = b.replace("* ", "");
            Branch::new(&name.trim_start(), is_checked_out)
        })
        .collect()
}

impl BranchCommand {
    pub fn next_step(&self, app: &mut App) -> Result<(), GituiError> {
        match self {
            BranchCommand::Switch => {
                app.current_screen = CurrentScreen::ListingBranches;

                app.branches = Branches::new(get_branches());

                app.selected_branch_command = Some(BranchCommand::Switch);

                Ok(())
            }

            BranchCommand::Merge => {
                app.current_screen = CurrentScreen::ListingBranches;

                app.branches = Branches::new(get_branches());

                app.selected_branch_command = Some(BranchCommand::Merge);

                Ok(())
            }
        }
    }
}

impl From<&str> for Command {
    fn from(value: &str) -> Self {
        match value {
            "Branch" => Command::Branch,
            "Fetch All" => Command::FetchAll,
            _ => panic!("{value} is not a valid command"),
        }
    }
}


impl From<&str> for BranchCommand {
    fn from(value: &str) -> Self {
        match value {
            "Switch" => BranchCommand::Switch,
            "Merge" => BranchCommand::Merge,
            _ => panic!("{value} is not a valid command"),
        }
    }
}

pub enum BranchCommand {
    Switch,
    Merge,
}

impl BranchCommand {
    pub fn to_string(&self) -> String {
        String::from(match self {
            BranchCommand::Switch => "Switch",
            BranchCommand::Merge => "Merge",
        })
    }
}

pub enum Modal {
    Open,
    Closed,
}

pub enum GituiError {
    BranchSwitch(String),
    FetchAll(String),
    BranchMerge(String),
}

impl std::fmt::Display for GituiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GituiError::BranchSwitch(s) => write!(f, "{}", s),
            GituiError::FetchAll(s) => write!(f, "{}", s),
            GituiError::BranchMerge(s) => write!(f, "{}", s),
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
            index,
        }
    }

    pub fn switch(&mut self) -> Result<(), GituiError> {
        if self.is_checked_out {
            return Err(GituiError::BranchSwitch(
                "branch is already checked out".to_string(),
            ));
        }
        let stdout = std::process::Command::new("git")
            .arg("switch")
            .arg(&self.name.trim())
            .output()
            .expect("couldnt switch branch")
            .stdout;

        let msg = String::from_utf8(stdout).expect("couldn't parse output");

        if !msg.contains("error:") {
            self.is_checked_out = true;
            Ok(())
        } else {
            Err(GituiError::BranchSwitch(format!(
                "failed to switch branch. output: {}",
                msg
            )))
        }
    }

    pub fn merge(&mut self) -> Result<(), GituiError> {
        if self.is_checked_out {
            return Err(GituiError::BranchMerge(
                "cant merge branch".to_string(),
            ));
        }
        let stdout = std::process::Command::new("git")
            .arg("merge")
            .arg(&self.name.trim())
            .output()
            .expect("couldnt switch branch")
            .stdout;

        let msg = String::from_utf8(stdout).expect("couldn't parse output");

        if !msg.contains("error:") {
            Ok(())
        } else {
            Err(GituiError::BranchSwitch(format!(
                "failed to merge branch. output: {}",
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
    curr_index: usize,
}

impl Branches {
    pub fn new(branches: Vec<Branch>) -> Self {
        let mut indexed: Vec<IndexedBranch> = Vec::new();
        for (i, b) in branches.iter().enumerate() {
            indexed.push(IndexedBranch::new(&b.name, b.is_checked_out, i));
        }

        Self {
            values: indexed,
            curr_index: 0,
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

    pub fn switch_current(&mut self) -> Result<(), GituiError> {
        self.values[self.curr_index].switch()?;

        let current_branch_name = &self.values[self.curr_index].name;

        self.uncheckout_all_except(current_branch_name.to_string());

        Ok(())
    }

    pub fn merge_current(&mut self) -> Result<(), GituiError> {
        self.values[self.curr_index].merge()?;

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

impl From<&Branches> for Scrollable {
    fn from(branches: &Branches) -> Scrollable {
        Scrollable::new(
            branches
                .values
                .iter()
                .map(|b| (String::from(b.get_display_name()), Index(b.index)))
                .collect(),
            Some(branches.curr_index),
        )
    }
}

pub struct Scrollable {
    items: Vec<(String, Index)>,
    selection: usize,
}

pub struct Index(pub usize);

impl Scrollable {
    pub fn new(items: Vec<(String, Index)>, index: Option<usize>) -> Self {
        Self {
            items,
            selection: index.unwrap_or(0),
        }
    }

    pub fn filtered(&self, query: &str) -> Scrollable {
        let mut scrollable = Vec::new();
        for item in self.items.iter() {
            if query != "" {
                if item.0.contains(query) {
                    scrollable.push((String::from(&item.0), Index(item.1 .0)));
                }
            } else {
                scrollable.push((String::from(&item.0), Index(item.1 .0)));
            }
        }
        Scrollable::new(scrollable, Some(self.selection))
    }

    pub fn get_current(&self) -> Option<&(String, Index)> {
        if self.is_empty() {
            return None;
        }
        if self.selection_invalid() {
            return None;
        }
        Some(&self.items[self.selection])
    }

    pub fn next(&mut self) -> Option<&(String, Index)> {
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

    pub fn prev(&mut self) -> Option<&(String, Index)> {
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

    pub fn reset_index(&mut self) {
        self.selection = 0;
    }

    pub fn select_from_index(&mut self, index: usize) -> &(String, Index) {
        self.selection = index;
        &self.items[self.selection]
    }

    pub fn get_index(&self) -> usize {
        self.selection
    }

    pub fn get_items(&self) -> &Vec<(String, Index)> {
        &self.items
    }
}
