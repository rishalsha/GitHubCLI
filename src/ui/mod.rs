use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppMode};

pub fn ui(f: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(f.area());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(main_chunks[0]);

    draw_details_pane(f, app, chunks[0]);
    draw_repos_pane(f, app, chunks[1]);
    draw_footer(f, main_chunks[1]);
    
    match &app.mode {
        AppMode::CreatingRepoName => draw_create_repo_popup(f, app),
        AppMode::CreatingRepoVisibility => draw_create_repo_visibility_popup(f, app),
        AppMode::DeletingRepoConfirmation => draw_delete_repo_confirmation_popup(f, app),
        AppMode::Error(err) => draw_error_popup(f, err.clone()),
        AppMode::Message(msg) => draw_message_popup(f, msg.clone(), "Success".to_string()),
        AppMode::CloningRepoPath => draw_clone_repo_path_popup(f, app),
        AppMode::CloningRepoRename => draw_clone_repo_rename_popup(f, app),
        AppMode::AddingRemoteName => draw_add_remote_name_popup(f, app),
        AppMode::Searching => draw_search_popup(f, app),
        AppMode::PromptInitGit { remote_name, .. } => draw_prompt_init_git_popup(f, remote_name),
        _ => {}
    }
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let footer_text = vec![
        Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": create  |  "),
        Span::styled("Enter/d", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": clone  |  "),
        Span::styled("/", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": search  |  "),
        Span::styled("r", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": add remote  |  "),
        Span::styled("o/b", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
        Span::raw(": open browser  |  "),
        Span::styled("x/Del", Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)),
        Span::raw(": delete  |  "),
        Span::styled("↑/↓ or j/k", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": navigate  |  "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": quit"),
    ];

    let p = Paragraph::new(Line::from(footer_text))
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(p, area);
}

fn draw_details_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)].as_ref())
        .split(area);

    // User Info
    let user_info = if let Some(ref user) = app.user {
        vec![
            Line::from(vec![
                Span::styled("User: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&user.login),
            ]),
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(user.name.as_deref().unwrap_or("N/A")),
            ]),
            Line::from(vec![
                Span::styled("Bio: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(user.bio.as_deref().unwrap_or("N/A")),
            ]),
            Line::from(vec![
                Span::styled("Public Repos: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(user.public_repos.to_string()),
            ]),
        ]
    } else {
        vec![Line::from("Loading user info...")]
    };

    let user_block = Paragraph::new(user_info)
        .block(Block::default().borders(Borders::ALL).title("Profile"))
        .wrap(Wrap { trim: true });
    f.render_widget(user_block, chunks[0]);

    // Repository Details
    let selected_repo = app.state.selected().and_then(|i| app.repos.get(i));

    if let Some(repo) = selected_repo {
        let repo_info = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&repo.name),
            ]),
            Line::from(vec![
                Span::styled("URL: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&repo.html_url),
            ]),
            Line::from(vec![
                Span::styled("Stars: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(repo.stargazers_count.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Open Issues: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(repo.open_issues_count.to_string()),
            ]),
            Line::from(""),
            Line::from(Span::styled("Description", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(repo.description.as_deref().unwrap_or("No description provided.")),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("Available Actions:", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(vec![Span::styled(" c ", Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray)), Span::raw(" Create new repository")]),
            Line::from(vec![Span::styled(" d / Enter ", Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray)), Span::raw(" Clone repository")]),
            Line::from(vec![Span::styled(" r ", Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray)), Span::raw(" Add as git remote")]),
            Line::from(vec![Span::styled(" o / b ", Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray)), Span::raw(" Open in browser")]),
            Line::from(vec![Span::styled(" x / Del ", Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray)), Span::styled(" Delete repository", Style::default().fg(Color::Red))]),
        ];

        let detail_block = Paragraph::new(repo_info)
            .block(Block::default().borders(Borders::ALL).title("Repository Details"))
            .wrap(Wrap { trim: true });

        f.render_widget(detail_block, chunks[1]);
    } else {
        let detail_block = Paragraph::new("Select a repository to view details\n\nPress 'c' to create a new repository")
            .block(Block::default().borders(Borders::ALL).title("Repository Details"));
        f.render_widget(detail_block, chunks[1]);
    }
}

fn draw_repos_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let repos: Vec<ListItem> = app
        .repos
        .iter()
        .map(|repo| ListItem::new(repo.name.clone()))
        .collect();

    let list = List::new(repos)
        .block(Block::default().title("Repositories").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.state);
}

fn draw_create_repo_popup(f: &mut Frame, app: &mut App) {
    let block = Block::default()
        .title("Create New Repository")
        .borders(Borders::ALL);

    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);
    
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let p = Paragraph::new(app.input.value())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::BOTTOM).title("Repo Name (Press Enter to submit, Esc to cancel)"));
    
    f.render_widget(p, inner_area);

    match app.mode {
        AppMode::CreatingRepoName => {
            f.set_cursor_position((
                inner_area.x + app.input.visual_cursor() as u16,
                inner_area.y + 1,
            ))
        }
        _ => {}
    }
}

fn draw_create_repo_visibility_popup(f: &mut Frame, app: &mut App) {
    let block = Block::default()
        .title("Repository Visibility")
        .borders(Borders::ALL);

    let area = centered_rect(60, 20, f.area());
    f.render_widget(Clear, area);
    
    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let list_items = vec![
        ListItem::new(if !app.new_repo_private { "[X] Public" } else { "[ ] Public" }),
        ListItem::new(if app.new_repo_private { "[X] Private" } else { "[ ] Private" }),
    ];

    let p = List::new(list_items)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::BOTTOM).title(format!("Visibility for '{}' (Use Up/Down to toggle, Enter to submit)", app.new_repo_name)));
    
    f.render_widget(p, inner_area);
}

fn draw_error_popup(f: &mut Frame, error_msg: String) {
    let block = Block::default()
        .title("Error")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red));

    let area = centered_rect(60, 30, f.area());
    f.render_widget(Clear, area);
    
    let p = Paragraph::new(error_msg)
        .wrap(Wrap { trim: true })
        .block(block);
    
    f.render_widget(p, area);
}

fn draw_message_popup(f: &mut Frame, msg: String, title: String) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Green));

    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);
    
    let p = Paragraph::new(msg)
        .wrap(Wrap { trim: true })
        .block(block);
    
    f.render_widget(p, area);
}

fn draw_delete_repo_confirmation_popup(f: &mut Frame, app: &mut App) {
    if let Some(i) = app.state.selected() {
        if let Some(repo) = app.repos.get(i) {
            let block = Block::default()
                .title("Delete Repository")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Red));

            let area = centered_rect(60, 20, f.area());
            f.render_widget(Clear, area);
            
            let inner_area = block.inner(area);
            f.render_widget(block, area);

            let prompt = format!("Type '{}' to confirm deletion:", repo.full_name);
            let input_text = app.input.value();
            let text = vec![
                Line::from(vec![
                    Span::styled("WARNING: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw("This action "),
                    Span::styled("CANNOT", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" be undone. This will permanently delete the repository."),
                ]),
                Line::from(""),
                Line::from(prompt),
                Line::from(""),
                Line::from(Span::styled(input_text, Style::default().fg(Color::Yellow))),
            ];

            let p = Paragraph::new(text).wrap(Wrap { trim: true });
            f.render_widget(p, inner_area);

            // Render cursor
            f.set_cursor_position((
                inner_area.x + app.input.visual_cursor() as u16,
                inner_area.y + 4,
            ));
        }
    }
}

fn draw_clone_repo_path_popup(f: &mut Frame, app: &mut App) {
    let block = Block::default()
        .title("Clone to Path (default: .)")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);
    
    let text = format!("Path: {}\n\nPress Enter to proceed. To use default `.`, just press Enter.", app.input.value());
    let p = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(block);
    
    f.render_widget(p, area);

    // Render cursor
    f.set_cursor_position((
        area.x + 1 + app.input.cursor() as u16 + 6, // 6 for "Path: "
        area.y + 1
    ));
}

fn draw_clone_repo_rename_popup(f: &mut Frame, app: &mut App) {
    let block = Block::default()
        .title("Rename cloned directory (default: same as repo)")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);
    
    let text = format!("Rename: {}\n\nPress Enter to proceed. To keep original name, leave blank and press Enter.", app.input.value());
    let p = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(block);
    
    f.render_widget(p, area);

    // Render cursor
    f.set_cursor_position((
        area.x + 1 + app.input.cursor() as u16 + 8, // 8 for "Rename: "
        area.y + 1
    ));
}
fn draw_add_remote_name_popup(f: &mut Frame, app: &mut App) {
    let block = Block::default()
        .title("Add remote name (default: origin)")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);
    
    let text = format!("Remote: {}\n\nPress Enter to proceed.", app.input.value());
    let p = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(block);
    
    f.render_widget(p, area);

    f.set_cursor_position((
        area.x + 1 + app.input.cursor() as u16 + 8, // 8 for "Remote: "
        area.y + 1,
    ));
}

fn draw_search_popup(f: &mut Frame, app: &mut App) {
    let block = Block::default()
        .title("Search Repositories")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);

    let text = format!("Query: {}\n\nType to search. Selection moves to the first match. Press Enter or Esc to dismiss.", app.input.value());
    let p = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(block);

    f.render_widget(p, area);

    f.set_cursor_position((
        area.x + 1 + app.input.cursor() as u16 + 7, // 7 for "Query: "
        area.y + 1,
    ));
}

fn draw_prompt_init_git_popup(f: &mut Frame, remote_name: &str) {
    let block = Block::default()
        .title("Not a Git Repository")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    let area = centered_rect(50, 20, f.area());
    f.render_widget(Clear, area);
    
    let text = format!("Current directory is not a git repository.\n\nInitialize git in current directory and add remote '{}'?\n\nPress Enter to accept, Esc to cancel.", remote_name);
    let p = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(block);
    
    f.render_widget(p, area);
}
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
