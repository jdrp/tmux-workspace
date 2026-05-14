use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "tw", bin_name = "tw")]
#[command(about = "Launch repeatable tmux workspaces from TOML files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        name: String,

        #[arg(long, default_value = "blank")]
        template: String,

        #[arg(long, default_value = ".")]
        root: String,

        #[arg(long)]
        edit: bool,
    },

    List,

    Show {
        name: String,
    },

    Edit {
        name: String,
    },

    Start {
        name: String,
    },
}

#[derive(Deserialize, Serialize)]
struct Workspace {
    name: String,
    template: String,
    root: String,
    windows: Vec<Window>,
}

#[derive(Deserialize, Serialize)]
struct Window {
    name: String,
    command: String,
}

fn rust_workspace(name: String, root: String) -> Workspace {
    Workspace {
        name,
        template: String::from("rust"),
        root,
        windows: vec![
            Window {
                name: String::from("editor"),
                command: String::from("nvim ."),
            },
            Window {
                name: String::from("test"),
                command: String::from("zsh"),
            },
            Window {
                name: String::from("git"),
                command: String::from("lazygit"),
            },
        ],
    }
}

fn blank_workspace(name: String, root: String) -> Workspace {
    Workspace {
        name,
        template: String::from("blank"),
        root,
        windows: vec![Window {
            name: String::from("shell"),
            command: String::from("zsh"),
        }],
    }
}

fn python_workspace(name: String, root: String) -> Workspace {
    Workspace {
        name,
        template: String::from("python"),
        root,
        windows: vec![
            Window {
                name: String::from("editor"),
                command: String::from("nvim ."),
            },
            Window {
                name: String::from("run"),
                command: String::from("zsh"),
            },
            Window {
                name: String::from("git"),
                command: String::from("lazygit"),
            },
        ],
    }
}

fn web_workspace(name: String, root: String) -> Workspace {
    Workspace {
        name,
        template: String::from("web"),
        root,
        windows: vec![
            Window {
                name: String::from("editor"),
                command: String::from("nvim ."),
            },
            Window {
                name: String::from("server"),
                command: String::from("npm run dev"),
            },
            Window {
                name: String::from("git"),
                command: String::from("lazygit"),
            },
        ],
    }
}

fn build_workspace(template: &str, name: String, root: String) -> Result<Workspace, String> {
    match template {
        "blank" => Ok(blank_workspace(name, root)),
        "rust" => Ok(rust_workspace(name, root)),
        "python" => Ok(python_workspace(name, root)),
        "web" => Ok(web_workspace(name, root)),
        _ => Err(format!(
            "unknown template: {template}\navailable templates: blank, rust, python, web"
        )),
    }
}

fn print_workspace(workspace: &Workspace) {
    println!("name: {}", workspace.name);
    println!("template: {}", workspace.template);
    println!("root: {}", workspace.root);
    println!("windows:");

    for window in &workspace.windows {
        println!("  {}: {}", window.name, window.command);
    }
}

fn workspace_to_toml(workspace: &Workspace) -> Result<String, toml::ser::Error> {
    toml::to_string_pretty(workspace)
}

fn workspaces_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME environment variable is not set");

    PathBuf::from(home)
        .join(".config")
        .join("tmux-workspace")
        .join("workspaces")
}

fn workspace_file_path(name: &str) -> PathBuf {
    workspaces_dir().join(format!("{name}.toml"))
}

fn write_workspace_file(workspace: &Workspace) -> Result<PathBuf, String> {
    let dir = workspaces_dir();
    fs::create_dir_all(&dir)
        .map_err(|error| format!("failed to create config directory: {error}"))?;

    let path = workspace_file_path(&workspace.name);

    if path.exists() {
        return Err(format!("workspace already exists: {}", path.display()));
    }

    let toml = workspace_to_toml(workspace)
        .map_err(|error| format!("failed to serialize workspace: {error}"))?;

    fs::write(&path, toml).map_err(|error| format!("failed to write workspace file: {error}"))?;

    Ok(path)
}

fn read_workspace_file(path: &PathBuf) -> Result<Workspace, String> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("failed to read file: {error}"))?;

    toml::from_str::<Workspace>(&content).map_err(|error| format!("failed to parse TOML: {error}"))
}

fn list_workspaces() -> Result<Vec<Workspace>, String> {
    let dir = workspaces_dir();

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries =
        fs::read_dir(&dir).map_err(|error| format!("failed to read workspaces dir: {error}"))?;

    let mut workspaces = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
        let path = entry.path();

        if path.extension().and_then(|extension| extension.to_str()) != Some("toml") {
            continue;
        }

        match read_workspace_file(&path) {
            Ok(workspace) => workspaces.push(workspace),
            Err(message) => {
                println!("skipping {}: {message}", path.display());
            }
        }
    }

    Ok(workspaces)
}

fn print_workspace_list(workspaces: &[Workspace]) {
    if workspaces.is_empty() {
        println!("no workspaces found");
        return;
    }

    for workspace in workspaces {
        println!(
            "{}\t{}\t{}",
            workspace.name, workspace.template, workspace.root
        );
    }
}

fn load_workspace(name: &str) -> Result<Workspace, String> {
    let path = workspace_file_path(name);

    if !path.exists() {
        return Err(format!("workspace not found: {}", path.display()));
    }

    read_workspace_file(&path)
}

fn editor_command() -> String {
    std::env::var("EDITOR").unwrap_or_else(|_| String::from("nvim"))
}

fn edit_workspace(name: &str) -> Result<(), String> {
    let path = workspace_file_path(name);

    if !path.exists() {
        return Err(format!("workspace not found: {}", path.display()));
    }

    let editor = editor_command();

    let status = Command::new(&editor)
        .arg(&path)
        .status()
        .map_err(|error| format!("failed to open editor '{editor}': {error}"))?;

    if !status.success() {
        return Err(format!("editor exited with status: {status}"));
    }

    Ok(())
}

fn check_tmux_exists() -> Result<(), String> {
    let output = Command::new("tmux")
        .arg("-V")
        .output()
        .map_err(|error| format!("failed to run tmux: {error}"))?;

    if !output.status.success() {
        return Err(format!("tmux exited with status: {}", output.status));
    }

    Ok(())
}

fn tmux_session_exists(name: &str) -> Result<bool, String> {
    let output = Command::new("tmux")
        .arg("has-session")
        .arg("-t")
        .arg(name)
        .output()
        .map_err(|error| format!("failed to check tmux session: {error}"))?;

    Ok(output.status.success())
}

fn create_tmux_window(session_name: &str, root: &str, window: &Window) -> Result<(), String> {
    let output = Command::new("tmux")
        .arg("new-window")
        .arg("-t")
        .arg(session_name)
        .arg("-c")
        .arg(root)
        .arg("-n")
        .arg(&window.name)
        .arg(&window.command)
        .output()
        .map_err(|error| format!("failed to create tmux window: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "tmux new-window failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn create_tmux_session(workspace: &Workspace) -> Result<(), String> {
    let first_window = workspace
        .windows
        .first()
        .ok_or_else(|| String::from("workspace has no windows"))?;

    let output = Command::new("tmux")
        .arg("new-session")
        .arg("-d")
        .arg("-s")
        .arg(&workspace.name)
        .arg("-c")
        .arg(&workspace.root)
        .arg("-n")
        .arg(&first_window.name)
        .arg(&first_window.command)
        .output()
        .map_err(|error| format!("failed to create tmux session: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "tmux new-session failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    for window in workspace.windows.iter().skip(1) {
        create_tmux_window(&workspace.name, &workspace.root, window)?;
    }

    Ok(())
}

fn start_workspace(name: &str) -> Result<(), String> {
    let workspace = load_workspace(name)?;

    check_tmux_exists()?;

    let session_exists = tmux_session_exists(&workspace.name)?;

    if !session_exists {
        create_tmux_session(&workspace)?;
    }

    println!("start");
    print_workspace(&workspace);
    println!("session exists: {session_exists}");

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            name,
            template,
            root,
            edit,
        } => {
            let workspace = match build_workspace(&template, name, root) {
                Ok(workspace) => workspace,
                Err(message) => {
                    println!("{message}");
                    return;
                }
            };

            println!("init");
            print_workspace(&workspace);
            println!("edit: {edit}");

            let path = match write_workspace_file(&workspace) {
                Ok(path) => path,
                Err(message) => {
                    println!("{message}");
                    return;
                }
            };

            println!("created: {}", path.display());
        }
        Commands::List => {
            let workspaces = match list_workspaces() {
                Ok(workspaces) => workspaces,
                Err(message) => {
                    println!("{message}");
                    return;
                }
            };

            print_workspace_list(&workspaces);
        }
        Commands::Show { name } => {
            let workspace = match load_workspace(&name) {
                Ok(workspace) => workspace,
                Err(message) => {
                    println!("{message}");
                    return;
                }
            };

            print_workspace(&workspace);
        }
        Commands::Edit { name } => match edit_workspace(&name) {
            Ok(()) => {}
            Err(message) => {
                println!("{message}");
            }
        },
        Commands::Start { name } => match start_workspace(&name) {
            Ok(()) => {}
            Err(message) => {
                println!("{message}");
            }
        },
    }
}
