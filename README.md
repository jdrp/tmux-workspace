# tmux-workspace

`tmux-workspace` is a small Rust CLI for creating and launching repeatable tmux workspaces from TOML files.

Define a workspace once, then reopen the same tmux session layout whenever you need it.

```bash
tw init my-project --template rust --root .
tw start my-project
```

## What it does

`tmux-workspace` stores workspace recipes as TOML files under:

```text
~/.config/tmux-workspace/workspaces/
```

Each recipe defines:

- a workspace name
- a root directory
- a template name
- one or more tmux windows
- the command to run in each window

For example:

```toml
name = "tmux-workspace"
template = "rust"
root = "/home/user/dev/tmux-workspace"

[[windows]]
name = "editor"
command = "nvim ."

[[windows]]
name = "test"
command = "zsh"

[[windows]]
name = "git"
command = "lazygit"
```

Running `tw start tmux-workspace` creates or switches to a tmux session with those windows.

## Features

- Create workspace TOML files from built-in templates.
- List available workspaces.
- Show a workspace without starting tmux.
- Edit a workspace TOML file with `$EDITOR`, falling back to `nvim`.
- Start or attach to a tmux session from a workspace file.
- Use `tmux switch-client` when already inside tmux to avoid nested sessions.
- Use `tmux attach-session` when outside tmux.
- Preview tmux commands with `--dry-run`.
- Refuse to overwrite existing workspace files.
- Store workspace roots as absolute paths.

## Requirements

- Linux or another Unix-like environment
- Rust and Cargo
- tmux
- Optional: Neovim, LazyGit, or any tools referenced by your workspace commands

## Installation

### From a local clone

```bash
git clone <repo-url>
cd tmux-workspace
cargo install --path .
```

This installs the `tw` binary to Cargo's bin directory, usually:

```text
~/.cargo/bin/tw
```

Make sure Cargo's bin directory is in your `PATH`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Then check:

```bash
tw --help
tw --version
```

### During development

You can also run the CLI without installing it:

```bash
cargo run --bin tw -- --help
cargo run --bin tw -- list
```

## Usage

### Create a workspace

```bash
tw init NAME
```

By default, this uses:

- template: `blank`
- root: current directory
- edit: false

Examples:

```bash
tw init notes
tw init tmux-workspace --template rust --root .
tw init scripts --template python --root ~/dev/scripts
tw init portfolio --template web --root ~/dev/portfolio --edit
```

`--root` is resolved to an absolute path when the workspace is created.

Existing workspace files are not overwritten.

### Available templates

```text
blank
rust
python
web
```

#### `blank`

```text
shell    zsh
```

#### `rust`

```text
editor   nvim .
test     zsh
git      lazygit
```

#### `python`

```text
editor   nvim .
run      zsh
git      lazygit
```

#### `web`

```text
editor   nvim .
server   npm run dev
git      lazygit
```

### List workspaces

```bash
tw list
```

Example output:

```text
tmux-workspace    rust      /home/user/dev/tmux-workspace
notes             blank     /home/user/notes
portfolio         web       /home/user/dev/portfolio
```

### Show a workspace

```bash
tw show NAME
```

Example:

```bash
tw show tmux-workspace
```

Output:

```text
name: tmux-workspace
template: rust
root: /home/user/dev/tmux-workspace
windows:
  editor: nvim .
  test: zsh
  git: lazygit
```

### Edit a workspace

```bash
tw edit NAME
```

This opens:

```text
~/.config/tmux-workspace/workspaces/NAME.toml
```

The editor is selected from `$EDITOR`. If `$EDITOR` is not set, `nvim` is used.

You can also open the file immediately after creating it:

```bash
tw init NAME --template rust --root . --edit
```

### Start a workspace

```bash
tw start NAME
```

Behavior:

- If the tmux session already exists, `tw` switches or attaches to it.
- If it does not exist, `tw` creates the session and windows from the TOML file.
- If running inside tmux, `tw` uses `tmux switch-client`.
- If running outside tmux, `tw` uses `tmux attach-session`.

Example:

```bash
tw start tmux-workspace
```

Equivalent shell idea:

```bash
tmux new-session -d -s tmux-workspace -c /home/user/dev/tmux-workspace -n editor 'nvim .'
tmux new-window -t tmux-workspace -c /home/user/dev/tmux-workspace -n test 'zsh'
tmux new-window -t tmux-workspace -c /home/user/dev/tmux-workspace -n git 'lazygit'
tmux attach-session -t tmux-workspace
```

### Dry run

Preview what `start` would do without creating or attaching to a tmux session:

```bash
tw start NAME --dry-run
```

Example output:

```text
Would start workspace: tmux-workspace
Root: /home/user/dev/tmux-workspace

Commands:
  tmux new-session -d -s tmux-workspace -c /home/user/dev/tmux-workspace -n editor 'nvim .'
  tmux new-window -t tmux-workspace -c /home/user/dev/tmux-workspace -n test 'zsh'
  tmux new-window -t tmux-workspace -c /home/user/dev/tmux-workspace -n git 'lazygit'
  tmux attach-session -t tmux-workspace
```

## Commands

```text
tw init NAME [--template TEMPLATE] [--root PATH] [--edit]
tw list
tw show NAME
tw edit NAME
tw start NAME [--dry-run]
```

## Development

Run checks:

```bash
cargo fmt
cargo check
cargo test
cargo clippy -- -D warnings
```

Run locally:

```bash
cargo run --bin tw -- --help
cargo run --bin tw -- init demo --template rust --root .
cargo run --bin tw -- list
cargo run --bin tw -- show demo
cargo run --bin tw -- start demo --dry-run
```

Install the current local build:

```bash
cargo install --path .
```

## Project status

The core MVP is implemented:

- `init`
- `list`
- `show`
- `edit`
- `start`
- built-in templates
- TOML serialization and parsing
- tmux session creation and attachment
- `start --dry-run`

The project is still early and mainly built for personal workflow and Rust learning.

## Roadmap

Planned or possible improvements:

- Better error types with `anyhow` or `thiserror`
- More validation for workspace files
- More tests for storage, path handling, and tmux command planning
- Shell completions
- Pane support in TOML
- Project-specific environment variables
- Workspace groups
- Import layout from an existing tmux session
- User-defined templates
- Optional Git branch or status display in `tw list`
- Bootstrap commands for creating projects, explicitly kept out of the MVP

## Non-goals for the MVP

The MVP does not aim to include:

- project bootstrap commands
- pane layouts
- plugin systems
- remote sessions
- a TUI
- automatic GitHub repository creation
- cross-platform support beyond Unix-like systems

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

