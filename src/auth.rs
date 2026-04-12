use dialoguer::Input;
use dialoguer::theme::ColorfulTheme;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitHubCredentials {
    pub token: String,
}

impl GitHubCredentials {
    pub fn get_config_path() -> Option<PathBuf> {
        let dirs = ProjectDirs::from("com", "githubcli", "ghcli")?;
        Some(dirs.config_dir().join("credentials.json"))
    }

    pub fn load() -> Option<Self> {
        let config_path = Self::get_config_path()?;
        let file_contents = std::fs::read_to_string(config_path).ok()?;
        serde_json::from_str(&file_contents).ok()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let config_path = Self::get_config_path().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine config directory",
            )
        })?;

        // Ensure the parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(config_path, json)?;
        Ok(())
    }
}

/// Handles prompting the user to paste their PAT manually
pub fn manual_auth_flow() -> Result<GitHubCredentials, String> {
    println!("\nPlease create a Personal Access Token on GitHub.");
    println!("(Settings -> Developer Settings -> Personal access tokens)\n");
    println!("\nPlease create a Personal Access Token on GitHub.");
    println!("(Settings -> Developer Settings -> Personal access tokens)\n");

    let token: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter GITHUB_TOKEN (paste and press enter)")
        .interact_text()
        .map_err(|e| e.to_string())?;

    Ok(GitHubCredentials { token })
}
