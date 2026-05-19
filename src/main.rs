mod storage;
mod templates;
mod workspace;

use clap::{Parser, Subcommand};
use std::process::Command;

use crate::storage::{
    list_workspaces, load_workspace, normalize_root, print_workspace_list, workspace_file_path,
    write_workspace_file,
};
use crate::templates::build_workspace;
use crate::workspace::{Window, Workspace, print_workspace};

#[derive(Parser)]
#[command(name = "tw", bin_name = "tw")]
#[command(version)]
#[command(about = "Launch repeatable tmux workspaces from TOML files")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Create a new workspace TOML file")]
    Init {
        name: String,

        #[arg(long, default_value = "blank")]
        template: String,

        #[arg(long, default_value = ".")]
        root: String,

        #[arg(long)]
        edit: bool,
    },

    #[command(about = "List available workspaces")]
    List,

    #[command(about = "Show workspace structure without starting tmux")]
    Show { name: String },

    #[command(about = "Open a workspace TOML file in $EDITOR")]
    Edit { name: String },

    #[command(about = "Create or attach to a tmux workspace session")]
    Start { name: String },
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

fn attach_tmux_session(session_name: &str) -> Result<(), String> {
    let inside_tmux = std::env::var("TMUX").is_ok();

    let status = if inside_tmux {
        Command::new("tmux")
            .arg("switch-client")
            .arg("-t")
            .arg(session_name)
            .status()
            .map_err(|error| format!("failed to switch tmux client: {error}"))?
    } else {
        Command::new("tmux")
            .arg("attach-session")
            .arg("-t")
            .arg(session_name)
            .status()
            .map_err(|error| format!("failed to attach tmux session: {error}"))?
    };

    if !status.success() {
        return Err(format!("tmux attach failed with status: {status}"));
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

    attach_tmux_session(&workspace.name)?;

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
            let root = match normalize_root(&root) {
                Ok(root) => root,
                Err(message) => {
                    println!("{message}");
                    return;
                }
            };

            let workspace = match build_workspace(&template, name, root) {
                Ok(workspace) => workspace,
                Err(message) => {
                    println!("{message}");
                    return;
                }
            };

            println!("init");
            print_workspace(&workspace);

            let path = match write_workspace_file(&workspace) {
                Ok(path) => path,
                Err(message) => {
                    println!("{message}");
                    return;
                }
            };

            println!("created: {}", path.display());

            if edit {
                match edit_workspace(&workspace.name) {
                    Ok(()) => {}
                    Err(message) => {
                        println!("{message}");
                    }
                }
            }
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
