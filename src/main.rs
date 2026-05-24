mod editor;
mod storage;
mod templates;
mod tmux;
mod tui;
mod workspace;

use clap::{Parser, Subcommand};

use crate::editor::edit_workspace;
use crate::storage::{
    list_workspaces, load_workspace, normalize_root, print_workspace_list, write_workspace_file,
};
use crate::templates::{Template, build_workspace};
use crate::tmux::start_workspace;
use crate::tui::TuiAction;
use crate::workspace::print_workspace;

#[derive(Parser)]
#[command(name = "tw", bin_name = "tw")]
#[command(version)]
#[command(about = "Launch repeatable tmux workspaces from TOML files")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Create a new workspace TOML file")]
    Init {
        name: String,

        #[arg(long, value_enum, default_value_t = Template::Custom)]
        template: Template,

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
    Start {
        name: String,

        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init {
            name,
            template,
            root,
            edit,
        }) => {
            let root = match normalize_root(&root) {
                Ok(root) => root,
                Err(message) => {
                    println!("{message}");
                    return;
                }
            };

            let workspace = build_workspace(template, name, root);

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

        Some(Commands::List) => {
            let workspaces = match list_workspaces() {
                Ok(workspaces) => workspaces,
                Err(message) => {
                    println!("{message}");
                    return;
                }
            };

            print_workspace_list(&workspaces);
        }

        Some(Commands::Show { name }) => {
            let workspace = match load_workspace(&name) {
                Ok(workspace) => workspace,
                Err(message) => {
                    println!("{message}");
                    return;
                }
            };

            print_workspace(&workspace);
        }

        Some(Commands::Edit { name }) => match edit_workspace(&name) {
            Ok(()) => {}
            Err(message) => {
                println!("{message}");
            }
        },

        Some(Commands::Start { name, dry_run }) => match start_workspace(&name, dry_run) {
            Ok(()) => {}
            Err(message) => {
                println!("{message}");
            }
        },

        None => match tui::run() {
            Ok(TuiAction::Start(name)) => {
                if let Err(message) = start_workspace(&name, false) {
                    println!("{message}");
                }
            }
            Ok(TuiAction::Edit(name)) => {
                if let Err(message) = edit_workspace(&name) {
                    println!("{message}");
                }
            }
            Ok(TuiAction::Quit) => {}
            Err(message) => {
                println!("{message}");
            }
        },
    }
}
