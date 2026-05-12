use clap::{Parser, Subcommand, builder::Str};

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

struct Workspace {
    name: String,
    template: String,
    root: String,
    windows: Vec<Window>,
}

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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            name,
            template,
            root,
            edit,
        } => {

            let workspace = match template.as_str() {
                "rust" => rust_workspace(name, root),
                _ => {
                    println!("template is not implemented yet: {template}");
                    return;
                }
            };

            println!("init");
            println!("name: {}", workspace.name);
            println!("template: {}", workspace.template);
            println!("root: {}", workspace.root);
            println!("edit: {edit}");
            println!("windows:");

            for window in &workspace.windows {
                println!("  {}: {}", window.name, window.command);
            }
        }
        Commands::List => {
            println!("list");
        }
        Commands::Show { name } => {
            println!("show {name}");
        }
        Commands::Edit { name } => {
            println!("edit {name}");
        }
        Commands::Start { name } => {
            println!("start {name}");
        }
    }
}
