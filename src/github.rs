use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use directories::ProjectDirs;
use std::fs;
use std::path::Path;

pub fn get_cache_dir() -> Option<std::path::PathBuf> {
    let dirs = ProjectDirs::from("com", "githubcli", "ghcli")?;
    Some(dirs.cache_dir().to_path_buf())
}

pub fn get_cached_user() -> Option<User> {
    let path = get_cache_dir()?.join("user.json");
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn get_cached_repos() -> Option<Vec<Repo>> {
    let path = get_cache_dir()?.join("repos.json");
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_cached_user(user: &User) {
    if let Some(cache_dir) = get_cache_dir() {
        fs::create_dir_all(&cache_dir).ok();
        let path = cache_dir.join("user.json");
        if let Ok(json) = serde_json::to_string(user) {
            fs::write(path, json).ok();
        }
    }
}

fn save_cached_repos(repos: &[Repo]) {
    if let Some(cache_dir) = get_cache_dir() {
        fs::create_dir_all(&cache_dir).ok();
        let path = cache_dir.join("repos.json");
        if let Ok(json) = serde_json::to_string(repos) {
            fs::write(path, json).ok();
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub login: String,
    pub name: Option<String>,
    pub bio: Option<String>,
    pub public_repos: u32,
    pub followers: u32,
    pub following: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Repo {
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub clone_url: String,
    pub description: Option<String>,
    pub stargazers_count: u32,
    pub open_issues_count: u32,
}

#[derive(Clone)]
pub struct GithubClient {
    client: reqwest::Client,
    pub token: String,
}

impl GithubClient {
    pub fn new(token: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("GitHubCli-Ratatui-Rust"));
        if !token.is_empty() {
            // Standard PAT bearer token
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", token)) {
                headers.insert(AUTHORIZATION, val);
            }
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Self { client, token }
    }

    pub async fn get_user_info(&self) -> Result<User, Box<dyn std::error::Error>> {
        let url = "https://api.github.com/user"; // Requires token to fetch
        let res = self.client.get(url).send().await?;
        
        // Fallback if no token (it gets rate limited quickly and may fail auth on /user)
        if res.status() != 200 {
            let err_body = res.text().await.unwrap_or_default();
            return Ok(User {
                login: format!("No Token / Unauthorized (Status: {})", err_body),
                name: None,
                bio: Some("Set GITHUB_TOKEN using --auth or .env file to fetch user info".into()),
                public_repos: 0,
                followers: 0,
                following: 0,
            });
        }

        let user: User = res.json().await?;
        save_cached_user(&user);
        Ok(user)
    }

    pub async fn list_repos(&self) -> Result<Vec<Repo>, Box<dyn std::error::Error>> {
        let mut all_repos = Vec::new();
        let mut url = "https://api.github.com/user/repos?sort=updated&per_page=100".to_string();

        loop {
            let res = self.client.get(&url).send().await?;

            if res.status() != 200 {
                break;
            }

            let headers = res.headers().clone();
            let page_repos: Vec<Repo> = res.json().await?;
            if page_repos.is_empty() {
                break;
            }
            all_repos.extend(page_repos);

            // Check if there is a next page
            let mut next_url = None;
            if let Some(link_header) = headers.get(reqwest::header::LINK) {
                if let Ok(link_str) = link_header.to_str() {
                    for link in link_str.split(',') {
                        if link.contains("rel=\"next\"") {
                            if let Some(start) = link.find('<') {
                                if let Some(end) = link.find('>') {
                                    next_url = Some(link[start + 1..end].to_string());
                                }
                            }
                        }
                    }
                }
            }

            if let Some(next) = next_url {
                url = next;
            } else {
                break;
            }
        }

        save_cached_repos(&all_repos);
        Ok(all_repos)
    }

    pub async fn create_repo(&self, name: &str, private: bool) -> Result<(), Box<dyn std::error::Error>> {
        let url = "https://api.github.com/user/repos";
        let payload = serde_json::json!({
            "name": name,
            "private": private
        });
        
        let res = self.client.post(url).json(&payload).send().await?;
        if !res.status().is_success() {
            let err_text = res.text().await?;
            return Err(format!("Failed to create repo: {}", err_text).into());
        }
        Ok(())
    }

    pub async fn delete_repo(&self, full_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("https://api.github.com/repos/{}", full_name);
        let res = self.client.delete(&url).send().await?;
        
        if !res.status().is_success() {
            let err_text = res.text().await?;
            return Err(format!("Failed to delete repo: {}", err_text).into());
        }
        
        Ok(())
    }

    pub fn clone_repo(&self, clone_url: &str, path: &str, rename: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Inject the token into the URL so no prompt needed
        let auth_url = if !self.token.is_empty() && clone_url.starts_with("https://") {
            clone_url.replace("https://", &format!("https://{}@", self.token))
        } else {
            clone_url.to_string()
        };

        let base_path = if path.trim().is_empty() { "." } else { path.trim() };
        
        // We do a small hack: git might print to stdout and mess up the terminal
        // We can run it in a subprocess and capture stdout to not break Ratatui.
        let mut cmd = std::process::Command::new("git");
        cmd.arg("clone");
        cmd.arg(&auth_url);
        
        // If the user specified a rename, pass it as the destination folder name.
        // Otherwise, Git will use the repository's original name automatically.
        if !rename.trim().is_empty() && rename.trim() != "." {
            cmd.arg(rename.trim());
        }

        // Set the working directory where git will place the cloned folder
        cmd.current_dir(base_path);

        let output = cmd
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .output()?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            let safe_err = if !self.token.is_empty() {
                err_msg.replace(&self.token, "***")
            } else {
                err_msg.to_string()
            };
            return Err(format!("Git failed: {}", safe_err).into());
        }
        Ok(())
    }
}

pub fn add_remote(remote_name: &str, clone_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("remote");
    cmd.arg("add");
    cmd.arg(remote_name);
    cmd.arg(clone_url);

    let output = cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        if err_msg.to_lowercase().contains("not a git repository") {
            return Err("NOT_A_GIT_REPO".into());
        }
        return Err(format!("Git failed: {}", err_msg).into());
    }
    Ok(())
}

pub fn init_git() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("init");

    let output = cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git init failed: {}", err_msg).into());
    }
    Ok(())
}

pub fn ensure_gitignore_exists() -> Result<bool, Box<dyn std::error::Error>> {
    let path = Path::new(".gitignore");
    let existed = path.exists();

    if !existed {
        fs::write(path, "")?;
    }

    Ok(existed)
}

pub fn read_gitignore_content() -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(".gitignore")?;
    Ok(content)
}

pub fn open_gitignore_in_editor() -> Result<(), Box<dyn std::error::Error>> {
    let editor = std::env::var("VISUAL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            std::env::var("EDITOR")
                .ok()
                .filter(|s| !s.trim().is_empty())
        })
        .unwrap_or_else(|| "nano".to_string());

    let mut parts = editor.split_whitespace();
    let program = parts.next().ok_or("Invalid editor command")?;
    let mut cmd = std::process::Command::new(program);
    cmd.args(parts);
    cmd.arg(".gitignore");

    let status = cmd.status()?;
    if !status.success() {
        return Err(format!("Editor exited with status: {}", status).into());
    }

    Ok(())
}

pub fn git_add_all() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("add");
    cmd.arg(".");

    let output = cmd
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git add failed: {}", err_msg).into());
    }

    Ok(())
}

pub fn is_git_repo() -> bool {
    let output = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output();

    match output {
        Ok(out) => out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "true",
        Err(_) => false,
    }
}
