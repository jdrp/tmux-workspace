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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_serializes_to_toml() {
        let workspace = Workspace {
            name: String::from("demo"),
            template: String::from("rust"),
            root: String::from("/home/test/demo"),
            windows: vec![
                Window {
                    name: String::from("editor"),
                    command: String::from("nvim ."),
                },
                Window {
                    name: String::from("test"),
                    command: String::from("zsh"),
                },
            ],
        };

        let toml = workspace_to_toml(&workspace).expect("workspace should serialize to TOML");

        assert!(toml.contains(r#"name = "demo""#));
        assert!(toml.contains(r#"template = "rust""#));
        assert!(toml.contains(r#"root = "/home/test/demo""#));
        assert!(toml.contains("[[windows]]"));
        assert!(toml.contains(r#"command = "nvim .""#));
    }

    #[test]
    fn workspace_round_trips_through_toml() {
        let original = Workspace {
            name: String::from("demo"),
            template: String::from("rust"),
            root: String::from("/home/test/demo"),
            windows: vec![
                Window {
                    name: String::from("editor"),
                    command: String::from("nvim ."),
                },
                Window {
                    name: String::from("git"),
                    command: String::from("lazygit"),
                },
            ],
        };

        let toml = workspace_to_toml(&original).expect("workspace should serialize to TOML");

        let parsed = toml::from_str::<Workspace>(&toml).expect("workspace TOML should parse back");

        assert_eq!(parsed.name, original.name);
        assert_eq!(parsed.template, original.template);
        assert_eq!(parsed.root, original.root);

        assert_eq!(parsed.windows.len(), 2);
        assert_eq!(parsed.windows[0].name, "editor");
        assert_eq!(parsed.windows[0].command, "nvim .");
        assert_eq!(parsed.windows[1].name, "git");
        assert_eq!(parsed.windows[1].command, "lazygit");
    }
}
