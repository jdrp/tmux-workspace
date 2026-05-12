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
            println!("init");
            println!("name: {name}");
            println!("template: {template}");
            println!("root: {root}");
            println!("edit: {edit}");
        }
        Commands::List => {
            println!("list");
        }
        Commands::Show { name } => {
            println!("show {name}");
        }
    }
}
