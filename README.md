# tmux-workspace

`tmux-workspace` is a small Rust CLI for creating and launching repeatable tmux workspaces from TOML files.

Define a workspace once, then reopen the same tmux session layout whenever you need it.

```bash
tw init my-project --template rust --root .
tw start my-project
```

Workspace files are stored under:

```text
~/.config/tmux-workspace/workspaces/
```

## Features

- Create workspace TOML files from built-in templates.
- List available workspaces.
- Show a workspace without starting tmux.
- Edit a workspace TOML file with `$EDITOR`, falling back to `nvim`.
- Start or attach to a tmux session from a workspace file.
- Create tmux windows and panes from TOML.
- Apply tmux layouts such as `tiled`, `main-vertical`, `main-horizontal`, `even-horizontal`, and `even-vertical`.
- Send configured commands into interactive shells using `tmux send-keys`.
- Keep panes open after commands finish.
- Use `tmux switch-client` when already inside tmux to avoid nested sessions.
- Use `tmux attach-session` when outside tmux.
- Preview tmux actions with `--dry-run`.
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

- template: `custom`
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
custom
rust
python
web
```

#### `custom`

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
notes             custom     /home/user/notes
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
- If it does not exist, `tw` creates the session from the TOML file.
- If running inside tmux, `tw` uses `tmux switch-client`.
- If running outside tmux, `tw` uses `tmux attach-session`.

Example:

```bash
tw start tmux-workspace
```

### Dry run

Preview what `start` would do without creating or attaching to a tmux session:

```bash
tw start NAME --dry-run
```

## Workspace TOML

A workspace file defines a tmux session.

Simple workspace:

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

### Panes

Windows can also define panes.

Pane support is useful for long-running development processes, such as a full-stack app with an editor, backend server, frontend server, and Git UI.

```toml
name = "fullstack-app"
template = "custom"
root = "/home/user/dev/fullstack-app"

[[windows]]
name = "dev"
layout = "main-vertical"

[[windows.panes]]
command = "nvim ."

[[windows.panes]]
command = "cd backend; cargo run"

[[windows.panes]]
command = "cd frontend; npm run dev"

[[windows]]
name = "git"
command = "lazygit"
```

This opens Neovim in the `dev` window, starts backend and frontend dev servers in panes, and opens LazyGit in a separate `git` window.

Configured commands are sent into interactive shells using `tmux send-keys`. This means that when a command exits, the pane remains open and returns to the shell.

Because commands are sent to a shell, normal shell syntax works:

```toml
command = "cd backend; cargo run"
```

### Layouts

A window can optionally request a tmux layout:

```toml
[[windows]]
name = "dev"
layout = "tiled"
```

Supported layouts:

```text
even-horizontal
even-vertical
main-horizontal
main-vertical
tiled
```

Layouts are applied with `tmux select-layout` after panes are created.

For example, a four-pane dashboard can use `layout = "tiled"`:

```toml
name = "dashboard"
template = "custom"
root = "/home/user/dev/app"

[[windows]]
name = "dev"
layout = "tiled"

[[windows.panes]]
command = "nvim ."

[[windows.panes]]
command = "cd backend; cargo run"

[[windows.panes]]
command = "cd frontend; npm run dev"

[[windows.panes]]
command = "lazygit"
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

Implemented:

- `init`
- `list`
- `show`
- `edit`
- `start`
- built-in templates
- TOML serialization and parsing
- tmux session creation and attachment
- tmux window support
- tmux pane support
- optional tmux layouts
- `start --dry-run`

The project is still early and mainly built for personal workflow and Rust learning.

## Roadmap

Planned or possible improvements:

- Text user interface for browsing and starting workspaces
- Better error types with `anyhow` or `thiserror`
- More validation for workspace files
- More tests for storage, path handling, and tmux command planning
- Shell completions
- Project-specific environment variables
- Workspace groups
- Import layout from an existing tmux session
- User-defined templates
- Optional Git branch or status display in `tw list`
- More deterministic pane layout support
- Bootstrap commands for creating projects, explicitly kept out of the MVP

## Non-goals for the MVP

The MVP does not aim to include:

- project bootstrap commands
- plugin systems
- remote sessions
- automatic GitHub repository creation
- cross-platform support beyond Unix-like systems

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
