use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::env;

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

fn client() -> reqwest::Client {
    let token = env::var("GITHUB_TOKEN").unwrap_or_default();
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("GitHubCli-Ratatui-Rust"));
    if !token.is_empty() {
        // Standard PAT bearer token
        if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", token)) {
            headers.insert(AUTHORIZATION, val);
        }
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
}

pub async fn get_user_info() -> Result<User, Box<dyn std::error::Error>> {
    let url = "https://api.github.com/user"; // Requires token to fetch
    let res = client().get(url).send().await?;
    
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
    Ok(user)
}

pub async fn list_repos() -> Result<Vec<Repo>, Box<dyn std::error::Error>> {
    let url = "https://api.github.com/user/repos?sort=updated&per_page=100";
    let res = client().get(url).send().await?;

    if res.status() != 200 {
        return Ok(vec![]);
    }

    let repos: Vec<Repo> = res.json().await?;
    Ok(repos)
}

pub async fn create_repo(name: &str, private: bool) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.github.com/user/repos";
    let payload = serde_json::json!({
        "name": name,
        "private": private
    });
    
    let res = client().post(url).json(&payload).send().await?;
    if !res.status().is_success() {
        let err_text = res.text().await?;
        return Err(format!("Failed to create repo: {}", err_text).into());
    }
    Ok(())
}

pub async fn delete_repo(full_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{}", full_name);
    let res = client().delete(&url).send().await?;
    
    if !res.status().is_success() {
        let err_text = res.text().await?;
        return Err(format!("Failed to delete repo: {}", err_text).into());
    }
    
    Ok(())
}

pub fn clone_repo(clone_url: &str, path: &str, rename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Inject the token into the URL so no prompt needed
    let token = env::var("GITHUB_TOKEN").unwrap_or_default();
    
    let auth_url = if !token.is_empty() && clone_url.starts_with("https://") {
        clone_url.replace("https://", &format!("https://{}@", token))
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
        let safe_err = if !token.is_empty() {
            err_msg.replace(&token, "***")
        } else {
            err_msg.to_string()
        };
        return Err(format!("Git failed: {}", safe_err).into());
    }
    Ok(())
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
