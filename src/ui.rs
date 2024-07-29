use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, CurrentScreen, Modal};

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    // Create the layout sections.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.size());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let title =
        Paragraph::new(Text::styled("Gitui", Style::default().fg(Color::Green))).block(title_block);

    f.render_widget(title, chunks[0]);
    let current_navigation_text = vec![
        // The first half of the text
        match app.current_screen {
            CurrentScreen::Main => Span::styled("Normal Mode", Style::default().fg(Color::Green)),
            CurrentScreen::ListingBranches => {
                Span::styled("Listing Branches", Style::default().fg(Color::Blue))
            }
            CurrentScreen::Exiting => Span::styled("Exiting", Style::default().fg(Color::LightRed)),
            CurrentScreen::Errors => Span::styled("Error", Style::default().fg(Color::Red)),
        }
        .to_owned(),
        // A white divider bar to separate the two sections
        Span::styled(" | ", Style::default().fg(Color::White)),
        // The final section of the text, with hints on what the user is editing
        {
            if matches!(&app.list_branches_modal, Modal::Open) {
                let msg = if let Some(name) = app.branches.get_currently_checkedout_name() {
                        format!("Current branch: {}", name)
                } else {
                    "No branch selected".to_string()
                };
                Span::styled(msg, Style::default().fg(Color::Green))
            } else if matches!(app.error_modal, Modal::Open) {
                Span::styled("Branches", Style::default().fg(Color::Green))
            } else {
                Span::styled(
                    "Waiting for something to happen",
                    Style::default().fg(Color::DarkGray),
                )
            }
        },
    ];

    let mode_footer = Paragraph::new(Line::from(current_navigation_text))
        .block(Block::default().borders(Borders::ALL));

    let current_keys_hint = {
        match app.current_screen {
            CurrentScreen::Main => Span::styled(
                "(q) to quit / (b) to list branches",
                Style::default().fg(Color::Red),
            ),
            CurrentScreen::ListingBranches => Span::styled(
                "(ESC|q) to cancel/(j/k) to navigate/(ENTER) to select",
                Style::default().fg(Color::Red),
            ),
            CurrentScreen::Exiting => Span::styled(
                "(q) to quit / (b) to list branches",
                Style::default().fg(Color::Red),
            ),
            CurrentScreen::Errors => {
                Span::styled("(ESC|q) to quit", Style::default().fg(Color::Red))
            }
        }
    };

    let key_notes_footer =
        Paragraph::new(Line::from(current_keys_hint)).block(Block::default().borders(Borders::ALL));

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    f.render_widget(mode_footer, footer_chunks[0]);
    f.render_widget(key_notes_footer, footer_chunks[1]);

    if matches!(app.list_branches_modal, Modal::Open) {
        let popup_block = Block::default()
            .title("Branches")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let area = centered_rect(60, 30, f.size());
        f.render_widget(popup_block, area);

        let popup_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(area);

        let search_block = if !app.in_search_bar {
            Block::default()
                .title("Search")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray))
        } else {
            Block::default()
                .title("Searching... <esc> to exit")
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::DarkGray))
        };

        f.render_widget(search_block, popup_chunks[0]);

        let search_text = if !app.search_query.is_empty() {
            Paragraph::new(app.search_query.to_string())
        } else {
            Paragraph::new("")
        };

        f.render_widget(search_text, popup_chunks[0].inner(&Margin::new(1, 1)));

        let list_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        f.render_widget(list_block, popup_chunks[1]);

        let mut list_items = Vec::<ListItem>::new();

        for (i, branch) in app.branches.get_values().iter().enumerate() {
            let style = if app.branches.get_index() == i && !app.in_search_bar {
                Style::default().fg(Color::Red).bg(Color::White)
            } else {
                Style::default().fg(Color::Yellow)
            };
            let can_push = if !app.search_query.is_empty() {
                branch.get_display_name().contains(app.search_query.as_str())
            } else {
                true
            };

            if can_push {
                list_items.push(ListItem::new(Line::from(Span::styled(
                    branch.get_display_name(),
                    style,
                ))));
            }
        }

        let list_inner_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let list = List::new(list_items).block(list_inner_block);

        f.render_widget(list, popup_chunks[1].inner(&Margin::new(1, 1)));
    }

    if matches!(app.error_modal, Modal::Open) {
        let popup_block = Block::default()
            .title("Errors")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray).fg(Color::Red));

        let area = centered_rect(60, 25, f.size());
        f.render_widget(popup_block, area);

        let mut list_items = Vec::<ListItem>::new();

        if !app.errors.is_empty() {
            for err in app.errors.iter() {
                list_items.push(ListItem::new(Line::from(Span::styled(
                    err.to_string(),
                    Style::default().fg(Color::Red),
                ))));
            }
        }

        let list = List::new(list_items);

        let area = centered_rect(55, 20, f.size());
        f.render_widget(list, area);
    }

    if let CurrentScreen::Exiting = app.current_screen {
        f.render_widget(Clear, f.size()); //this clears the entire screen and anything already drawn
        let popup_block = Block::default()
            .title("Y/N")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let exit_text = Text::styled(
            "Would you like to output the buffer as json? (y/n)",
            Style::default().fg(Color::Red),
        );
        // the `trim: false` will stop the text from being cut off when over the edge of the block
        let exit_paragraph = Paragraph::new(exit_text)
            .block(popup_block)
            .wrap(Wrap { trim: false });

        let area = centered_rect(60, 25, f.size());
        f.render_widget(exit_paragraph, area);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}
