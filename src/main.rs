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
    app::{App, Branch, BranchIterator, CurrentScreen, Modal},
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
                        app.list_branches_modal = Some(Modal::Open);

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

                        app.branches = Some(BranchIterator::new(branches));
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
                CurrentScreen::ListingBranches if key.kind == KeyEventKind::Press => match key.code
                {
                    KeyCode::Enter => {
                        if let Some(modal_open) = &app.list_branches_modal {
                            match modal_open {
                                Modal::Open => {
                                    if let Some(branches) = &mut app.branches {
                                        branches
                                            .checkout_current()
                                            .unwrap_or_else(|err| println!("{}", err))
                                    }
                                }
                                Modal::Closed => {
                                    app.current_screen = CurrentScreen::Main;
                                }
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(modal_open) = &app.list_branches_modal {
                            match modal_open {
                                Modal::Open => {}
                                Modal::Closed => {}
                            }
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        app.current_screen = CurrentScreen::Main;
                        app.list_branches_modal = None;
                    }
                    KeyCode::Tab => {
                        app.toggle_branches_modal_open();
                    }
                    KeyCode::Char(value) => {
                        if let Some(modal_open) = &app.list_branches_modal {
                            match modal_open {
                                Modal::Open => match value {
                                    'j' => {
                                        if let Some(branches) = &mut app.branches {
                                            branches.next();
                                        }
                                    }
                                    'k' => {
                                        if let Some(branches) = &mut app.branches {
                                            branches.prev();
                                        }
                                    }
                                    _ => {}
                                },
                                Modal::Closed => {}
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
