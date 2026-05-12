# tmux-workspace

`tmux-workspace` is a Rust CLI for launching repeatable tmux workspaces from TOML files.

The goal is simple: define a project once, then open the same tmux session layout every time.

This project is also a learning project. It is designed to help me learn Rust while building a tool I will actually use in my daily terminal workflow with tmux and Neovim.

## Why

When working on a project, I usually want the same terminal setup:

- one window for editing with Neovim
- one window for running commands, tests, or development servers
- one window for Git or LazyGit
- sometimes extra windows or panes for logs, docs, shells, or experiments

Instead of recreating that by hand every time, `tmux-workspace` reads a TOML file and starts the tmux session for me.

## Project goals

The first goal is not to build a huge project generator.

The first goal is to build a small, reliable workspace launcher:

```bash
tw init my-project --template rust --root .
tw show my-project
tw start my-project
```

Later, the project can grow into a more complete workspace manager.

## Command name

The package/project name is:

```bash
tmux-workspace
```

The preferred command alias is:

```bash
tw
```

The long binary name can remain available, but daily usage should be short and comfortable.

## Core concepts

### Workspace

A workspace is a TOML file describing a tmux session.

It has:

- a name
- a root directory
- one or more tmux windows
- optional commands to run in those windows
- later, optional panes

### Template

A template is a predefined TOML shape for common project types.

Examples:

- blank
- rust
- python
- web
- dotfiles

Templates create workspace TOML files. They do not automatically create the actual project unless a future explicit bootstrap command is added.

### Start

Starting a workspace means reading its TOML file and creating or attaching to the corresponding tmux session.

## MVP

The MVP should stay focused.

### MVP commands

```bash
tw init NAME
tw init NAME --template TEMPLATE
tw init NAME --root PATH
tw init NAME --template TEMPLATE --root PATH --edit

tw list
tw show NAME
tw edit NAME
tw start NAME
```

### MVP behavior

#### `tw init`

Creates a new workspace TOML file.

Default behavior:

```bash
tw init my-project
```

Equivalent intent:

- template: `blank`
- root: current directory
- edit: false

Examples:

```bash
tw init tmux-workspace --template rust --root .
tw init notes --template blank --root ~/notes
tw init portfolio --template web --root ~/projects/portfolio --edit
```

#### `tw list`

Lists available workspaces.

Example output:

```text
tmux-workspace    rust      ~/projects/tmux-workspace
dotfiles          blank     ~/dotfiles
portfolio         web       ~/projects/portfolio
```

#### `tw show`

Shows the parsed workspace in a human-readable format without running tmux.

Example:

```bash
tw show tmux-workspace
```

Example output:

```text
Workspace: tmux-workspace
Template: rust
Root: ~/projects/tmux-workspace

Windows:
  1. editor    nvim .
  2. test      zsh
  3. git       lazygit
```

#### `tw edit`

Opens the workspace TOML file in `$EDITOR`.

Example:

```bash
tw edit tmux-workspace
```

Expected behavior:

```bash
$EDITOR ~/.config/tmux-workspace/workspaces/tmux-workspace.toml
```

#### `tw start`

Starts or attaches to a tmux session based on the workspace TOML.

Example:

```bash
tw start tmux-workspace
```

Equivalent shell idea:

```bash
tmux new-session -d -s tmux-workspace -c ~/projects/tmux-workspace -n editor 'nvim .'
tmux new-window -t tmux-workspace -c ~/projects/tmux-workspace -n test 'zsh'
tmux new-window -t tmux-workspace -c ~/projects/tmux-workspace -n git 'lazygit'
tmux attach -t tmux-workspace
```

The Rust program should not rely on a single shell string if it can avoid it. It should call `tmux` through `std::process::Command`.

## Proposed config directory

Workspace files should live in:

```text
~/.config/tmux-workspace/workspaces/
```

Example:

```text
~/.config/tmux-workspace/workspaces/tmux-workspace.toml
~/.config/tmux-workspace/workspaces/dotfiles.toml
~/.config/tmux-workspace/workspaces/portfolio.toml
```

## Example TOML files

### Blank workspace

```toml
name = "notes"
template = "blank"
root = "~/notes"

[[windows]]
name = "shell"
command = "zsh"
```

### Rust workspace

```toml
name = "tmux-workspace"
template = "rust"
root = "~/projects/tmux-workspace"

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

### Python workspace

```toml
name = "data-scripts"
template = "python"
root = "~/projects/data-scripts"

[[windows]]
name = "editor"
command = "nvim ."

[[windows]]
name = "run"
command = "zsh"

[[windows]]
name = "git"
command = "lazygit"
```

### Web workspace

```toml
name = "portfolio"
template = "web"
root = "~/projects/portfolio"

[[windows]]
name = "editor"
command = "nvim ."

[[windows]]
name = "server"
command = "npm run dev"

[[windows]]
name = "git"
command = "lazygit"
```

## Future TOML: panes

Panes are not required for the first MVP.

Later, the TOML can support panes inside a window:

```toml
name = "rust-cli"
template = "rust"
root = "~/projects/rust-cli"

[[windows]]
name = "dev"

[[windows.panes]]
command = "nvim ."

[[windows.panes]]
command = "cargo test"

[[windows]]
name = "git"
command = "lazygit"
```

## Bootstrap: future idea, not MVP

A future version may support project creation commands.

This is intentionally not part of the MVP.

Reason: creating projects and launching workspaces are related, but they are different responsibilities. Adding bootstrap too early would make the first version more complex and less focused.

Possible future commands:

```bash
tw bootstrap NAME
tw bootstrap NAME --dry-run
tw init NAME --template rust --root . --bootstrap
```

Possible future TOML:

```toml
[[bootstrap.steps]]
description = "Create Rust project"
command = "cargo init"
creates = "Cargo.toml"

[[bootstrap.steps]]
description = "Create docs folder"
command = "mkdir -p docs examples"
```

Rules for a future bootstrap feature:

- never run by default
- require an explicit command or flag
- support `--dry-run`
- show commands before running them
- make steps idempotent where possible
- support `creates` to skip steps already completed
- treat shell commands as powerful and potentially dangerous

## Roadmap

### Phase 0: project setup

- [ ] Create Rust project with Cargo
- [ ] Add README
- [ ] Add initial Git commit
- [ ] Create development tmux session
- [ ] Use Neovim for all editing

### Phase 1: CLI skeleton

- [ ] Add `clap`
- [ ] Create `tw --help`
- [ ] Add subcommands:
  - [ ] `init`
  - [ ] `list`
  - [ ] `show`
  - [ ] `edit`
  - [ ] `start`

### Phase 2: data model

- [ ] Define Rust structs:
  - [ ] `Workspace`
  - [ ] `Window`
  - [ ] `Template`
- [ ] Parse TOML with `serde`
- [ ] Serialize TOML with `toml`
- [ ] Expand `~` in paths
- [ ] Validate required fields

### Phase 3: init and templates

- [ ] Implement blank template
- [ ] Implement Rust template
- [ ] Implement Python template
- [ ] Implement Web template
- [ ] Write workspace TOML files to config directory
- [ ] Add `--root`
- [ ] Add `--edit`
- [ ] Avoid overwriting existing workspace files unless `--force` is added later

### Phase 4: list, show, edit

- [ ] List workspace TOML files
- [ ] Parse each workspace for display
- [ ] Pretty-print a workspace
- [ ] Open workspace file using `$EDITOR`
- [ ] Fall back to `nvim` if `$EDITOR` is not set

### Phase 5: start tmux

- [ ] Check whether `tmux` exists
- [ ] Check whether a session already exists
- [ ] Create detached tmux session
- [ ] Create windows
- [ ] Run commands in the correct root directory
- [ ] Attach to the session
- [ ] Give readable errors if tmux fails

### Phase 6: polish

- [ ] Add better error messages with `anyhow` or `thiserror`
- [ ] Add tests for TOML parsing
- [ ] Add tests for template generation
- [ ] Add `--dry-run` for `start`
- [ ] Add README examples
- [ ] Add GitHub-ready documentation

### Phase 7: future features

- [ ] Pane support
- [ ] Bootstrap steps
- [ ] Bootstrap `--dry-run`
- [ ] Shell completions
- [ ] Import from existing tmux session
- [ ] Workspace groups
- [ ] Project-specific environment variables
- [ ] Optional Git branch/status display in `tw list`

## Rust learning goals

This project should be used to learn Rust gradually.

Topics to learn while building:

- Cargo project structure
- modules
- structs and enums
- `Result`
- error propagation with `?`
- ownership and borrowing
- `Path` and `PathBuf`
- `std::process::Command`
- reading and writing files
- parsing TOML
- CLI design with `clap`
- testing
- formatting with `cargo fmt`
- linting with `cargo clippy`

## Development workflow

Recommended tmux layout while building this project:

```text
tmux session: tmux-workspace

window 1: editor
  nvim .

window 2: run
  cargo run -- --help
  cargo run -- init test --template rust --root .
  cargo run -- show test

window 3: test
  cargo test
  cargo fmt
  cargo clippy

window 4: git
  lazygit
```

## Personal tmux shortcuts

Current tmux prefix:

```text
Ctrl-a
```

Useful tmux shortcuts:

```text
Ctrl-a d        detach
Ctrl-a n        new window
Ctrl-n          next window
Ctrl-p          previous window
Ctrl-a |        split left/right
Ctrl-a -        split top/bottom
Ctrl-a h/j/k/l  move between panes
Ctrl-a x        close pane
Ctrl-a z        zoom pane
Ctrl-a c        copy mode
```

In copy mode:

```text
v               start selection
y               copy selection using xclip
```

## Personal Neovim shortcuts

Leader is space.

Important shortcuts for this project:

```text
Space Space     find files
Space f         find files
Space g         live grep / search text in project
Space /         search inside current buffer
Space b         buffers
Space o         recent files
Space k         keymaps
Space h         help tags
Space -         open Oil explorer
Space e         open Oil explorer
Space G         LazyGit
Space t         toggle terminal

Space y         copy to clipboard
Space Y         copy entire file to clipboard
Space p         paste below from clipboard
Space P         paste above from clipboard

Space d         cut
Space c         cut and insert
Space x         cut character

Space r         replace word globally

Space lh        LSP hover
Space ld        LSP definition
Space lr        LSP references
Space ln        LSP rename
Space la        LSP code action
Space lf        LSP format buffer
Space le        show diagnostic
Space lj        next diagnostic
Space lk        previous diagnostic
Space lq        diagnostics list

Space wh        split horizontal
Space wv        split vertical
Space ww        next window
Space wo        only window
Space wq        close window

s               Leap forward
S               Leap backward
gs              Leap from window
```

Important custom behavior:

- `d`, `c`, and `x` delete/change without copying into the default register.
- Use `Space d`, `Space c`, and `Space x` when the intention is to cut.
- Use `Space y`, `Space Y`, and `Space p` for system clipboard workflows.

## First milestone

The first milestone is intentionally small:

```bash
tw init demo --template rust --root .
tw list
tw show demo
```

No tmux launching is required for the first milestone.

The first GitHub-worthy milestone is:

```bash
tw init demo --template rust --root .
tw start demo
```

That should create or attach to a tmux session with the windows defined in the generated TOML.

## Non-goals for the MVP

The MVP should not include:

- project bootstrap commands
- panes
- shell completion
- plugin systems
- remote sessions
- cross-platform support beyond Linux
- a TUI
- automatic GitHub repo creation

Those can be considered later.

## License

To be decided.
