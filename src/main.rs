use std::{error::Error, io};

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
    app::{App, Branch, Branches, CurrentScreen, Modal},
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
                        app.list_branches_modal = Modal::Open;
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
                CurrentScreen::ListingBranches
                    if !app.in_search_bar && key.kind == KeyEventKind::Press =>
                {
                    match key.code {
                        KeyCode::Enter => app
                            .branches
                            .checkout_current()
                            .unwrap_or_else(|err| {
                                if app.errors.len() > 0 {
                                    app.errors.push(err);
                                } else {
                                    app.errors = vec![err];
                                }

                                app.error_modal = Modal::Open;
                                app.list_branches_modal = Modal::Closed;
                                app.current_screen = CurrentScreen::Errors;
                            }),
                        KeyCode::Esc | KeyCode::Char('q') => {
                            app.current_screen = CurrentScreen::Main;
                            app.list_branches_modal = Modal::Closed;
                            app.branches.reset_index();
                        }

                        KeyCode::Char(value) => match value {
                            'j' => {
                                if app.branches.filtered(&app.search_query).is_last() {
                                    app.in_search_bar = true;
                                }
                                if let Some(next) = app.branches.filtered(&app.search_query).next()
                                {
                                    app.branches.select_from_index(next.index);
                                }
                            }
                            'k' => {
                                if app.branches.filtered(&app.search_query).is_first() {
                                    app.in_search_bar = true;
                                }
                                if let Some(prev) = app.branches.filtered(&app.search_query).prev()
                                {
                                    app.branches.select_from_index(prev.index);
                                }
                            }
                            c => {
                                print!("{}", c)
                            }
                        },
                        KeyCode::Tab => {
                            if app.branches.filtered(&app.search_query).is_last() {
                                app.in_search_bar = true;
                            }
                        }
                        KeyCode::BackTab => {
                            if app.branches.filtered(&app.search_query).is_first() {
                                app.in_search_bar = true;
                            }
                        }
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

                            if !app.branches.filtered(&app.search_query).is_first() {
                                if let Some(next) = app.branches.filtered(&app.search_query).next()
                                {
                                    app.branches.select_from_index(next.index);
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
