use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::{error::Error, io};
use clap::{Parser, Subcommand};

mod app;
mod auth;
mod github;
mod ui;

use app::{App, AppMode};
use tui_input::backend::crossterm::EventHandler;

#[derive(Parser, Debug)]
#[command(name = "ghcli", about = "A terminal UI for GitHub")]
#[command(version)]
#[command(disable_version_flag = true)]
struct Args {
    #[arg(short = 'v', short_alias = 'V', long = "version", action = clap::ArgAction::Version, help = "Print version")]
    version: (),

    #[arg(short = 'a', long = "auth", help = "Re-authenticate with a new token")]
    auth: bool,

    #[arg(long = "browser", help = "Extract cookies automatically from a browser (e.g. chrome, firefox)")]
    browser: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check authentication status
    Status,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments (-h, --help, -v, --version are handled automatically)
    let args = Args::parse();

    dotenvy::dotenv().ok();

    if let Some(Commands::Status) = args.command {
        match auth::GitHubCredentials::load() {
            Some(creds) => {
                println!("Authenticated: YES");
                println!("Token: {}", creds.token);
                if let Some(path) = auth::GitHubCredentials::get_config_path() {
                    println!("Config path: {:?}", path);
                }
            }
            None => {
                println!("Authenticated: NO");
                println!("Please run 'ghcli --auth' to authenticate.");
            }
        }
        return Ok(());
    }

    // Check for existing GitHub credentials, prompt if missing or forced
    let creds = if args.auth || args.browser.is_some() {
        let c = auth::manual_auth_flow()?;
        let _ = c.save();
        c
    } else {
        match auth::GitHubCredentials::load() {
            Some(c) => c,
            None => {
                let c = auth::manual_auth_flow()?;
                let _ = c.save();
                c
            }
        }
    };
    unsafe {
        std::env::set_var("GITHUB_TOKEN", creds.token);
    }

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new();
    
    // Initial fetches
    app.fetch_user_info().await?;
    app.fetch_repos().await?;

    let res = run_app(&mut terminal, &mut app).await;

    // restore terminal
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui::ui(f, app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            let event = event::read()?;
            
            match app.mode {
                AppMode::Normal => {
                    if let Event::Key(key) = event {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('/') => {
                                app.mode = AppMode::Searching;
                                app.input = tui_input::Input::new(app.search_query.clone());
                            }
                            KeyCode::Char('c') => {
                                app.mode = AppMode::CreatingRepoName;
                            }
                            KeyCode::Char('x') | KeyCode::Delete => {
                                if !app.repos.is_empty() {
                                    app.mode = AppMode::DeletingRepoConfirmation;
                                    app.input.reset();
                                }
                            }
                            KeyCode::Enter | KeyCode::Char('d') => {
                                if !app.repos.is_empty() {
                                    app.mode = AppMode::CloningRepoPath;
                                }
                            }
                            KeyCode::Char('r') => {
                                if !app.repos.is_empty() {
                                    app.mode = AppMode::AddingRemoteName;
                                    app.input.reset();
                                }
                            }
                            KeyCode::Char('o') | KeyCode::Char('b') => {
                                app.open_selected_repo_in_browser();
                            }
                            KeyCode::Down | KeyCode::Char('j') => app.next(),
                            KeyCode::Up | KeyCode::Char('k') => app.previous(),
                            _ => {}
                        }
                    }
                }
                AppMode::CreatingRepoName => {
                    if let Event::Key(key) = event {
                        match key.code {
                            KeyCode::Enter => {
                                app.new_repo_name = app.input.value().to_string();
                                if !app.new_repo_name.is_empty() {
                                    app.mode = AppMode::CreatingRepoVisibility;
                                }
                            }
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                                app.input.reset();
                                app.new_repo_name.clear();
                            }
                            _ => {
                                app.input.handle_event(&Event::Key(key));
                            }
                        }
                    }
                }
                AppMode::CreatingRepoVisibility => {
                    if let Event::Key(key) = event {
                        match key.code {
                            KeyCode::Up | KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('k') => {
                                app.new_repo_private = !app.new_repo_private;
                            }
                            KeyCode::Enter => {
                                app.create_repo().await?;
                            }
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                                app.input.reset();
                                app.new_repo_name.clear();
                            }
                            _ => {}
                        }
                    }
                }
                AppMode::Error(_) | AppMode::Message(_) => {
                    if let Event::Key(key) = event {
                        match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                                app.input.reset();
                                app.new_repo_name.clear();
                            }
                            _ => {}
                        }
                    }
                }
                AppMode::CloningRepoPath => {
                    if let Event::Key(key) = event {
                        match key.code {
                            KeyCode::Enter => {
                                app.clone_path = app.input.value().trim().to_string();
                                if app.clone_path.is_empty() {
                                    app.clone_path = ".".to_string();
                                }
                                app.input.reset();
                                app.mode = AppMode::CloningRepoRename;
                            }
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                                app.input.reset();
                                app.clone_path.clear();
                            }
                            _ => {
                                app.input.handle_event(&Event::Key(key));
                            }
                        }
                    }
                }
                AppMode::CloningRepoRename => {
                    if let Event::Key(key) = event {
                        match key.code {
                            KeyCode::Enter => {
                                let rename = app.input.value().trim().to_string();
                                app.input.reset();
                                let path = app.clone_path.clone();
                                app.clone_path.clear();
                                app.clone_selected_repo(&path, &rename);
                            }
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                                app.input.reset();
                                app.clone_path.clear();
                            }
                            _ => {
                                app.input.handle_event(&Event::Key(key));
                            }
                        }
                    }
                }
                AppMode::DeletingRepoConfirmation => {
                    if let Event::Key(key) = event {
                        match key.code {
                            KeyCode::Enter => {
                                let confirm = app.input.value().trim().to_string();
                                if let Some(i) = app.state.selected() {
                                    if let Some(repo) = app.repos.get(i) {
                                        if confirm == repo.full_name {
                                            app.delete_selected_repo().await?;
                                        } else {
                                            app.mode = AppMode::Error("Repository name did not match.".into());
                                            app.input.reset();
                                        }
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                                app.input.reset();
                            }
                            _ => {
                                app.input.handle_event(&Event::Key(key));
                            }
                        }
                    }
                }
                AppMode::AddingRemoteName => {
                    if let Event::Key(key) = event {
                        match key.code {
                            KeyCode::Enter => {
                                let remote_name = app.input.value().trim().to_string();
                                app.input.reset();
                                app.add_remote_to_repo(&remote_name);
                            }
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                                app.input.reset();
                            }
                            _ => {
                                app.input.handle_event(&Event::Key(key));
                            }
                        }
                    }
                }
                AppMode::Searching => {
                    if let Event::Key(key) = event {
                        match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                            }
                            _ => {
                                app.input.handle_event(&Event::Key(key));
                                app.search_query = app.input.value().to_string();
                                app.search_repos();
                            }
                        }
                    }
                }
                AppMode::PromptInitGit { ref remote_name, ref clone_url } => {
                    if let Event::Key(key) = event {
                        match key.code {
                            KeyCode::Enter => {
                                let remote_name = remote_name.clone();
                                let clone_url = clone_url.clone();
                                match github::init_git() {
                                    Ok(_) => {
                                        match github::add_remote(&remote_name, &clone_url) {
                                            Ok(_) => {
                                                app.mode = AppMode::Message(format!("Git initialized and remote '{}' added successfully!", remote_name));
                                            }
                                            Err(e) => {
                                                app.mode = AppMode::Error(e.to_string());
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        app.mode = AppMode::Error(e.to_string());
                                    }
                                }
                            }
                            KeyCode::Esc => {
                                app.mode = AppMode::Normal;
                                app.input.reset();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
