mod editor;
mod storage;
mod templates;
mod tmux;
mod workspace;

use clap::{Parser, Subcommand};

use crate::editor::edit_workspace;
use crate::storage::{
    list_workspaces, load_workspace, normalize_root, print_workspace_list, write_workspace_file,
};
use crate::templates::build_workspace;
use crate::tmux::start_workspace;
use crate::workspace::print_workspace;

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
