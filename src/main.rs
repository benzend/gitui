use std::{error::Error, io};

use app::{BranchCommand, Index};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

mod app;
mod ui;
use crate::{
    app::{App, Branch, Branches, Command, CurrentScreen, Modal, Scrollable},
    ui::ui,
};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                // Skip events that are not KeyEventKind::Press
                continue;
            }
            match app.current_screen {
                CurrentScreen::Main => match key.code {
                    KeyCode::Char('b') => {
                        app.current_screen = CurrentScreen::ListingBranches;
                        app.in_search_bar = true;

                        let stdout = std::process::Command::new("git")
                            .arg("branch")
                            .output()
                            .expect("to get git branches")
                            .stdout;

                        let branches: Vec<Branch> = String::from_utf8(stdout)
                            .expect("couldnt parse stdout")
                            .split("\n")
                            .into_iter()
                            .filter(|b| b.len() > 0)
                            .map(|b| {
                                let is_checked_out = b.contains("* ");
                                let name = b.replace("* ", "");
                                Branch::new(&name.trim_start(), is_checked_out)
                            })
                            .collect();

                        app.branches = Branches::new(branches);
                    }
                    KeyCode::Char('c') => {
                        app.current_screen = CurrentScreen::ListingCommands;
                        app.in_search_bar = true;
                    }
                    KeyCode::Char('q') => {
                        app.current_screen = CurrentScreen::Exiting;
                    }
                    _ => {}
                },
                CurrentScreen::Exiting => match key.code {
                    KeyCode::Char('y') => {
                        return Ok(true);
                    }
                    KeyCode::Char('n') | KeyCode::Char('q') => {
                        return Ok(false);
                    }
                    _ => {}
                },
                CurrentScreen::ListingCommands
                    if app.in_search_bar && key.kind == KeyEventKind::Press =>
                {
                    match key.code {
                        KeyCode::Backspace => {
                            if !app.search_query.is_empty() {
                                app.search_query = remove_last_char(&app.search_query).to_string();
                            }
                        }
                        KeyCode::Esc => {
                            app.in_search_bar = false;

                            app.commands.reset_index();

                            let mut filtered = app.commands.filtered(&app.search_query);

                            if !filtered.is_first() {
                                if let Some((_, Index(i))) = filtered.next() {
                                    app.commands.select_from_index(*i);
                                }
                            }
                        }
                        KeyCode::Char(value) => {
                            app.search_query = format!("{}{}", app.search_query, value);
                        }
                        _ => {}
                    }
                }
                CurrentScreen::ListingCommands
                    if !app.in_search_bar && key.kind == KeyEventKind::Press =>
                {
                    match key.code {
                        KeyCode::Enter => {
                            Command::from(app.commands.get_current().unwrap().0.as_str())
                                .next_step(app)
                                .unwrap_or_else(|err| {
                                    if app.errors.len() > 0 {
                                        app.errors.push(err);
                                    } else {
                                        app.errors = vec![err];
                                    }

                                    app.error_modal = Modal::Open;
                                    app.current_screen = CurrentScreen::Errors;
                                })
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            app.current_screen = CurrentScreen::Main;
                            app.commands.reset_index();
                        }

                        KeyCode::Char(value) => match value {
                            'j' => {
                                if let Some((_, Index(i))) =
                                    &app.commands.filtered(&app.search_query).next()
                                {
                                    app.commands.select_from_index(*i);
                                }
                            }
                            'k' => {
                                if let Some((_, Index(i))) =
                                    &app.commands.filtered(&app.search_query).prev()
                                {
                                    app.commands.select_from_index(*i);
                                }
                            }
                            'i' => {
                                app.in_search_bar = true;
                            }
                            c => {
                                print!("{}", c)
                            }
                        },
                        KeyCode::Tab => {}
                        KeyCode::BackTab => {}
                        _ => {}
                    }
                }
                CurrentScreen::ListingBranchCommands
                    if app.in_search_bar && key.kind == KeyEventKind::Press =>
                {
                    match key.code {
                        KeyCode::Backspace => {
                            if !app.search_query.is_empty() {
                                app.search_query = remove_last_char(&app.search_query).to_string();
                            }
                        }
                        KeyCode::Esc => {
                            app.in_search_bar = false;

                            app.branch_commands.reset_index();

                            let mut filtered = app.branch_commands.filtered(&app.search_query);

                            if !filtered.is_first() {
                                if let Some((_, Index(i))) = filtered.next() {
                                    app.branch_commands.select_from_index(*i);
                                }
                            }
                        }
                        KeyCode::Char(value) => {
                            app.search_query = format!("{}{}", app.search_query, value);
                        }
                        _ => {}
                    }
                }
                CurrentScreen::ListingBranchCommands
                    if !app.in_search_bar && key.kind == KeyEventKind::Press =>
                {
                    match key.code {
                        KeyCode::Enter => BranchCommand::from(
                            app.branch_commands.get_current().unwrap().0.as_str(),
                        )
                        .next_step(app)
                        .unwrap_or_else(|err| {
                            if app.errors.len() > 0 {
                                app.errors.push(err);
                            } else {
                                app.errors = vec![err];
                            }

                            app.error_modal = Modal::Open;
                            app.current_screen = CurrentScreen::Errors;
                        }),
                        KeyCode::Esc | KeyCode::Char('q') => {
                            app.current_screen = CurrentScreen::Main;
                            app.branch_commands.reset_index();
                        }

                        KeyCode::Char(value) => match value {
                            'j' => {
                                if let Some((_, Index(i))) =
                                    &app.branch_commands.filtered(&app.search_query).next()
                                {
                                    app.branch_commands.select_from_index(*i);
                                }
                            }
                            'k' => {
                                if let Some((_, Index(i))) =
                                    &app.branch_commands.filtered(&app.search_query).prev()
                                {
                                    app.branch_commands.select_from_index(*i);
                                }
                            }
                            'i' => {
                                app.in_search_bar = true;
                            }
                            c => {
                                print!("{}", c)
                            }
                        },
                        KeyCode::Tab => {}
                        KeyCode::BackTab => {}
                        _ => {}
                    }
                }

                CurrentScreen::ListingBranches
                    if !app.in_search_bar && key.kind == KeyEventKind::Press =>
                {
                    match key.code {
                        KeyCode::Enter => match &app.selected_branch_command {
                            Some(BranchCommand::Switch) => {
                                app.branches.switch_current().unwrap_or_else(|err| {
                                    if app.errors.len() > 0 {
                                        app.errors.push(err);
                                    } else {
                                        app.errors = vec![err];
                                    }

                                    app.error_modal = Modal::Open;
                                    app.current_screen = CurrentScreen::Errors;
                                })
                            }
                            Some(BranchCommand::Merge) => {
                                app.branches.merge_current().unwrap_or_else(|err| {
                                    if app.errors.len() > 0 {
                                        app.errors.push(err);
                                    } else {
                                        app.errors = vec![err];
                                    }

                                    app.error_modal = Modal::Open;
                                    app.current_screen = CurrentScreen::Errors;
                                })
                            }

                            None => app.branches.switch_current().unwrap_or_else(|err| {
                                if app.errors.len() > 0 {
                                    app.errors.push(err);
                                } else {
                                    app.errors = vec![err];
                                }

                                app.error_modal = Modal::Open;
                                app.current_screen = CurrentScreen::Errors;
                            }),
                        },
                        KeyCode::Esc | KeyCode::Char('q') => {
                            app.current_screen = CurrentScreen::Main;
                            app.branches.reset_index();
                        }

                        KeyCode::Char(value) => match value {
                            'j' => {
                                if let Some((_, Index(i))) =
                                    Scrollable::from(&app.branches.filtered(&app.search_query))
                                        .next()
                                {
                                    app.branches.select_from_index(*i);
                                }
                            }
                            'k' => {
                                if let Some((_, Index(i))) =
                                    Scrollable::from(&app.branches.filtered(&app.search_query))
                                        .prev()
                                {
                                    app.branches.select_from_index(*i);
                                }
                            }
                            'i' => {
                                app.in_search_bar = true;
                            }
                            c => {
                                print!("{}", c)
                            }
                        },
                        KeyCode::Tab => {}
                        KeyCode::BackTab => {}
                        _ => {}
                    }
                }
                CurrentScreen::ListingBranches
                    if app.in_search_bar && key.kind == KeyEventKind::Press =>
                {
                    match key.code {
                        KeyCode::Backspace => {
                            if !app.search_query.is_empty() {
                                app.search_query = remove_last_char(&app.search_query).to_string();
                            }
                        }
                        KeyCode::Esc => {
                            app.in_search_bar = false;

                            app.branches.reset_index();

                            let mut scrollable =
                                Scrollable::from(&app.branches.filtered(&app.search_query));

                            if !scrollable.is_first() {
                                if let Some((_, Index(i))) = scrollable.next() {
                                    app.branches.select_from_index(*i);
                                }
                            }
                        }
                        KeyCode::Char(value) => {
                            app.search_query = format!("{}{}", app.search_query, value);
                        }
                        _ => {}
                    }
                }
                CurrentScreen::Errors if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        app.current_screen = CurrentScreen::Main;
                        app.error_modal = Modal::Closed;
                        app.errors = Vec::new();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn remove_last_char(s: &str) -> &str {
    if s.len() == 0 {
        return s;
    }

    let mut ch = s.chars();
    ch.next_back();
    ch.as_str()
}
