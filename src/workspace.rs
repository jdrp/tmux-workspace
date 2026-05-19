use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Workspace {
    pub name: String,
    pub template: String,
    pub root: String,
    pub windows: Vec<Window>,
}

#[derive(Deserialize, Serialize)]
pub struct Window {
    pub name: String,
    pub command: String,
}

pub fn print_workspace(workspace: &Workspace) {
    println!("name: {}", workspace.name);
    println!("template: {}", workspace.template);
    println!("root: {}", workspace.root);
    println!("windows:");

    for window in &workspace.windows {
        println!("  {}: {}", window.name, window.command);
    }
}

pub fn workspace_to_toml(workspace: &Workspace) -> Result<String, toml::ser::Error> {
    toml::to_string_pretty(workspace)
}
