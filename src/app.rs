use ratatui::widgets::ListState;
use tui_input::Input;
use crate::github::{self, Repo, User};

pub enum AppMode {
    Normal,
    CreatingRepoName,
    CreatingRepoVisibility,
    CloningRepoPath,
    CloningRepoRename,
    DeletingRepoConfirmation,
    Error(String),
    Message(String),
}

pub struct App {
    pub user: Option<User>,
    pub repos: Vec<Repo>,
    pub state: ListState,
    pub mode: AppMode,
    pub input: Input,
    pub new_repo_name: String,
    pub new_repo_private: bool,
    pub clone_path: String,
}

impl App {
    pub fn new() -> App {
        App {
            user: None,
            repos: vec![],
            state: ListState::default(),
            mode: AppMode::Normal,
            input: Input::default(),
            new_repo_name: String::new(),
            new_repo_private: true,
            clone_path: String::new(),
        }
    }

    pub fn next(&mut self) {
        if self.repos.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.repos.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.repos.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.repos.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub async fn fetch_user_info(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.user = Some(github::get_user_info().await?);
        Ok(())
    }

    pub async fn fetch_repos(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.repos = github::list_repos().await?;
        if !self.repos.is_empty() {
            self.state.select(Some(0));
        }
        Ok(())
    }

    pub async fn create_repo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let repo_name = self.new_repo_name.clone();
        let is_private = self.new_repo_private;

        if !repo_name.is_empty() {
            match github::create_repo(&repo_name, is_private).await {
                Ok(_) => {
                    self.input.reset();
                    self.new_repo_name.clear();
                    self.mode = AppMode::Normal;
                    self.fetch_repos().await?;
                }
                Err(e) => {
                    self.mode = AppMode::Error(e.to_string());
                }
            }
        }
        Ok(())
    }

    pub fn clone_selected_repo(&mut self, path: &str, rename: &str) {
        if let Some(i) = self.state.selected() {
            if let Some(repo) = self.repos.get(i) {
                match github::clone_repo(&repo.clone_url, path, rename) {
                    Ok(_) => {
                        self.mode = AppMode::Message("Repository cloned successfully!".into());
                    }
                    Err(e) => {
                        self.mode = AppMode::Error(e.to_string());
                    }
                }
            } else {
                self.mode = AppMode::Normal;
            }
        } else {
            self.mode = AppMode::Normal;
        }
    }

    pub async fn delete_selected_repo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(i) = self.state.selected() {
            if let Some(repo) = self.repos.get(i).cloned() {
                match github::delete_repo(&repo.full_name).await {
                    Ok(_) => {
                        self.mode = AppMode::Normal;
                        self.input.reset();
                        self.fetch_repos().await?;
                    }
                    Err(e) => {
                        self.mode = AppMode::Error(e.to_string());
                        self.input.reset();
                    }
                }
            }
        }
        Ok(())
    }
}
