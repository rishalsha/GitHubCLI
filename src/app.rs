use ratatui::widgets::ListState;
use tui_input::Input;
use crate::github::{self, Repo, User, GithubClient};

pub enum AppMode {
    Normal,
    CreatingRepoName,
    CreatingRepoVisibility,
    CloningRepoPath,
    CloningRepoRename,
    DeletingRepoConfirmation,
    AddingRemoteName,
    Searching,
    InitGitConfirmation,
    InitGitGitignoreReview,
    PromptInitGit { remote_name: String, clone_url: String },
    Error(String),
    Message(String),
}

pub enum AppUpdate {
    User(User),
    Repos(Vec<Repo>),
    UserError(String),
    ReposError(String),
}

pub struct App {
    pub github: GithubClient,
    pub user: Option<User>,
    pub repos: Vec<Repo>,
    pub state: ListState,
    pub mode: AppMode,
    pub input: Input,
    pub search_query: String,
    pub new_repo_name: String,
    pub new_repo_private: bool,
    pub clone_path: String,
    pub gitignore_content: String,
    pub update_rx: Option<tokio::sync::mpsc::UnboundedReceiver<AppUpdate>>,
}

impl App {
    pub fn new(github: GithubClient) -> App {
        App {
            github,
            user: None,
            repos: vec![],
            state: ListState::default(),
            mode: AppMode::Normal,
            input: Input::default(),
            search_query: String::new(),
            new_repo_name: String::new(),
            new_repo_private: false,
            clone_path: String::new(),
            gitignore_content: String::new(),
            update_rx: None,
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


    pub async fn fetch_repos(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.repos = self.github.list_repos().await?;
        if !self.repos.is_empty() {
            self.state.select(Some(0));
        }
        Ok(())
    }

    pub fn search_repos(&mut self) {
        let query = self.search_query.trim().to_lowercase();
        if query.is_empty() {
            return;
        }
        
        if let Some(index) = self.repos.iter().position(|r| r.name.to_lowercase().contains(&query)) {
            self.state.select(Some(index));
        }
    }

    pub async fn create_repo(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let repo_name = self.new_repo_name.clone();
        let is_private = self.new_repo_private;

        if !repo_name.is_empty() {
            match self.github.create_repo(&repo_name, is_private).await {
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
                match self.github.clone_repo(&repo.clone_url, path, rename) {
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

    pub fn add_remote_to_repo(&mut self, remote_name: &str) {
        if let Some(i) = self.state.selected() {
            if let Some(repo) = self.repos.get(i).cloned() {
                let name = if remote_name.trim().is_empty() { "origin" } else { remote_name.trim() };
                // Actually add_remote could stay static since it just calls git.
                // But if we moved it inside GithubClient or left it, I should just use `github::add_remote` if it wasn't moved.
                // Wait, in github.rs I didn't move it inside impl GithubClient. So it's still `github::add_remote`. 
                // Ah, let me double check my github.rs modification. I'll leave add_remote and delete_repo as is for now and verify.
                match github::add_remote(name, &repo.clone_url) {
                    Ok(_) => {
                        self.mode = AppMode::Message(format!("Remote '{}' added successfully!", name));
                    }
                    Err(e) => {
                        if e.to_string() == "NOT_A_GIT_REPO" {
                            self.mode = AppMode::PromptInitGit {
                                remote_name: name.to_string(),
                                clone_url: repo.clone_url,
                            };
                        } else {
                            self.mode = AppMode::Error(e.to_string());
                        }
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
                match self.github.delete_repo(&repo.full_name).await {
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

    pub fn open_selected_repo_in_browser(&mut self) {
        if let Some(i) = self.state.selected() {
            if let Some(repo) = self.repos.get(i) {
                if let Err(e) = open::that(&repo.html_url) {
                    self.mode = AppMode::Error(format!("Failed to open browser: {}", e));
                }
            }
        }
    }
}
