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

Current development usage is still through Cargo:

```bash
cargo run -- init demo --template rust --root .
cargo run -- start demo
```

Installing or aliasing the daily command as `tw` is still pending.

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

Currently implemented built-in templates:

- blank
- rust
- python
- web

Templates create workspace TOML files. They do not automatically create the actual project unless a future explicit bootstrap command is added.

### Start

Starting a workspace means reading its TOML file and creating or attaching to the corresponding tmux session.

Current behavior:

- if the tmux session already exists, `start` enters it
- if the session does not exist, `start` creates it first
- when running inside tmux, `start` uses `tmux switch-client`
- when running outside tmux, `start` uses `tmux attach-session`

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

### Current implementation status

The current implementation supports the first GitHub-worthy milestone when run through Cargo:

```bash
cargo run -- init demo --template rust --root .
cargo run -- start demo
```

The first smaller functional milestone is also complete:

```bash
cargo run -- init demo --template rust --root .
cargo run -- list
cargo run -- show demo
```

Implemented so far:

- CLI parsing with `clap`
- subcommands: `init`, `list`, `show`, `edit`, `start`
- built-in templates: `blank`, `rust`, `python`, `web`
- workspace model: `Workspace` and `Window`
- TOML serialization and deserialization with `serde` and `toml`
- writing workspace files to `~/.config/tmux-workspace/workspaces/`
- refusing to overwrite existing workspace files
- listing available workspace TOML files
- showing a parsed workspace without running tmux
- opening a workspace TOML with `$EDITOR`
- falling back to `nvim` when `$EDITOR` is not set
- checking whether `tmux` exists
- checking whether a tmux session already exists
- creating detached tmux sessions
- creating tmux windows from the workspace TOML
- running tmux windows in the configured root directory
- attaching to an existing or newly created tmux session
- using `switch-client` instead of nested `attach` when already inside tmux

Still pending:

- `--edit` opening the TOML immediately after `init`
- installing or aliasing the daily command as `tw` outside `cargo run --`
- expanding `~` in paths
- validating workspace files more strictly
- adding tests
- adding `--dry-run` for `start`
- improving error handling with `anyhow` or `thiserror`

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

Current development usage:

```bash
cargo run -- init tmux-workspace --template rust --root .
```

Current notes:

- `init` writes a TOML file to `~/.config/tmux-workspace/workspaces/`
- existing workspace files are not overwritten
- `--edit` is parsed but does not yet open the file after creation

#### `tw list`

Lists available workspaces.

Example output:

```text
tmux-workspace    rust      ~/projects/tmux-workspace
dotfiles          blank     ~/dotfiles
portfolio         web       ~/projects/portfolio
```

Current implementation reads `.toml` files from:

```text
~/.config/tmux-workspace/workspaces/
```

It parses each workspace file and prints:

```text
name    template    root
```

#### `tw show`

Shows the parsed workspace in a human-readable format without running tmux.

Example:

```bash
tw show tmux-workspace
```

Current output shape:

```text
name: tmux-workspace
template: rust
root: ~/projects/tmux-workspace
windows:
  editor: nvim .
  test: zsh
  git: lazygit
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

Current status: implemented.

Behavior:

- opens the workspace TOML with `$EDITOR`
- falls back to `nvim` if `$EDITOR` is not set
- returns a readable error if the workspace does not exist

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

Current status: implemented.

Behavior:

- reads the workspace TOML
- checks that `tmux` exists
- checks whether the target session already exists
- creates the session if needed
- creates the configured windows
- uses the workspace root as the tmux working directory
- switches to the session when already inside tmux
- attaches to the session when outside tmux

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

- [x] Create Rust project with Cargo
- [x] Add README
- [x] Add initial Git commit
- [x] Create development tmux session
- [x] Use Neovim for all editing

### Phase 1: CLI skeleton

- [x] Add `clap`
- [x] Create `tw --help`
- [x] Add subcommands:
  - [x] `init`
  - [x] `list`
  - [x] `show`
  - [x] `edit`
  - [x] `start`

### Phase 2: data model

- [ ] Define Rust structs:
  - [x] `Workspace`
  - [x] `Window`
  - [ ] `Template` enum or dedicated type (templates are currently internal functions)
- [x] Parse TOML with `serde`
- [x] Serialize TOML with `toml`
- [ ] Expand `~` in paths
- [ ] Validate required fields

### Phase 3: init and templates

- [x] Implement blank template
- [x] Implement Rust template
- [x] Implement Python template
- [x] Implement Web template
- [x] Write workspace TOML files to config directory
- [x] Add `--root`
- [x] Parse `--edit` flag
- [ ] Open TOML after `init` when `--edit` is passed
- [x] Avoid overwriting existing workspace files unless `--force` is added later

### Phase 4: list, show, edit

- [x] List workspace TOML files
- [x] Parse each workspace for display
- [x] Pretty-print a workspace
- [x] Open workspace file using `$EDITOR`
- [x] Fall back to `nvim` if `$EDITOR` is not set

### Phase 5: start tmux

- [x] Check whether `tmux` exists
- [x] Check whether a session already exists
- [x] Create detached tmux session
- [x] Create windows
- [x] Run commands in the correct root directory
- [x] Attach to the session
- [x] Give readable errors if tmux fails

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

Topics learned or touched so far:

- Cargo project structure
- structs and enums
- `Result`
- error propagation with `?`
- ownership and borrowing
- `String` vs `&str`
- `PathBuf`
- `std::process::Command`
- reading and writing files
- parsing TOML
- serializing TOML
- CLI design with `clap`
- formatting with `cargo fmt`

Topics still to practice more deeply:

- modules
- custom error types
- `Path` vs `PathBuf`
- testing
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
  cargo run -- list
  cargo run -- show test
  cargo run -- edit test
  cargo run -- start test

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

Status: completed at the functional level through `cargo run --`.

No tmux launching is required for the first milestone.

## First GitHub-worthy milestone

The first GitHub-worthy milestone is:

```bash
tw init demo --template rust --root .
tw start demo
```

Status: completed at the functional level through `cargo run --`.

This creates or attaches to a tmux session with the windows defined in the generated TOML.

Still pending before daily usage feels complete:

- install or alias the binary as `tw`
- decide whether to keep both `tmux-workspace` and `tw` binary names
- improve path handling for `.` and `~`
- add tests

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
