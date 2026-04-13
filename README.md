# ghcli (GitHubCLI)

A fast, terminal-based user interface (TUI) for managing your GitHub repositories, written in Rust using [Ratatui](https://github.com/ratatui-org/ratatui).

## Features

- **Profile Overview**: View your GitHub user info, bio, followers, and public repository counts.
- **Repository Management**: List, search, and navigate through all your GitHub repositories.
- **Create**: Easily create new public or private repositories directly from the terminal.
- **Clone**: Clone repositories to your local machine with custom directory paths.
- **Git Integration**: Initialize git in the current directory and instantly add your repository as a remote.
- **Browser Integration**: Quickly open a repository in your default web browser.
- **Delete**: Delete repositories securely with built-in confirmation prompts.
- **Search**: Fast repository lookup moving you directly to your matches.

## Prerequisites

- [Rust & Cargo](https://rustup.rs/) (to build from source)
- Git (for cloning and adding remotes)

## Installation

### Using Make

You can quickly build and install `ghcli` to your `~/.local/bin` folder using the provided Makefile. Make sure `~/.local/bin` is in your system's `PATH`.

```bash
git clone https://github.com/rishalsha/GitHubCLI.git
cd GitHubCLI
make install
```

### Using Cargo

Alternatively, you can build it using cargo:

```bash
cargo build --release
```
The compiled binary will be placed in `target/release/ghcli`.

## Authentication

When you first run `ghcli`, it will guide you through an authentication flow to obtain a GitHub Personal Access Token (PAT) and save it locally.

To force re-authentication or provide a new token/cookie:
```bash
ghcli --auth
```

## Keybindings

| Key                 | Action                                      |
|---------------------|---------------------------------------------|
| `↑` / `↓` / `j`/`k` | Navigate repositories list                  |
| `/`                 | Search repositories                         |
| `c`                 | Create a new repository                     |
| `Enter` / `d`       | Clone the selected repository               |
| `r`                 | Add remote (prompts for `git init` if needed)|
| `o` / `b`           | Open selected repository in web browser     |
| `x` / `Del`         | Delete selected repository                  |
| `q`                 | Quit application                            |

## License

This project is licensed under the MIT License.
