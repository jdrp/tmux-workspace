use clap::{Parser, Subcommand};

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

fn build_workspace(template: String, name: String, root: String) -> Result<Workspace, String> {
    match template.as_str() {
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
    
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            name,
            template,
            root,
            edit,
        } => {
            let workspace = match build_workspace(template, name, root) {
                Ok(workspace) => workspace,
                Err(message) => {
                    println!("{message}");
                    return;
                }
            };

            println!("init");
            print_workspace(&workspace);
            println!("edit: {edit}");
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
