use clap::ValueEnum;

use crate::workspace::{Window, Workspace};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Template {
    Blank,
    Rust,
    Python,
    Web,
}

impl Template {
    pub fn as_str(self) -> &'static str {
        match self {
            Template::Blank => "blank",
            Template::Rust => "rust",
            Template::Python => "python",
            Template::Web => "web",
        }
    }
}

impl std::fmt::Display for Template {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

fn blank_workspace(name: String, root: String) -> Workspace {
    Workspace {
        name,
        template: Template::Blank.as_str().to_string(),
        root,
        windows: vec![Window {
            name: String::from("shell"),
            command: String::from("zsh"),
        }],
    }
}

fn rust_workspace(name: String, root: String) -> Workspace {
    Workspace {
        name,
        template: Template::Rust.as_str().to_string(),
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

fn python_workspace(name: String, root: String) -> Workspace {
    Workspace {
        name,
        template: Template::Python.as_str().to_string(),
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
        template: Template::Web.as_str().to_string(),
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

pub fn build_workspace(template: Template, name: String, root: String) -> Workspace {
    match template {
        Template::Blank => blank_workspace(name, root),
        Template::Rust => rust_workspace(name, root),
        Template::Python => python_workspace(name, root),
        Template::Web => web_workspace(name, root),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_template_creates_shell_window() {
        let workspace = build_workspace(
            Template::Blank,
            String::from("notes"),
            String::from("/home/test/notes"),
        );

        assert_eq!(workspace.name, "notes");
        assert_eq!(workspace.template, "blank");
        assert_eq!(workspace.root, "/home/test/notes");

        assert_eq!(workspace.windows.len(), 1);
        assert_eq!(workspace.windows[0].name, "shell");
        assert_eq!(workspace.windows[0].command, "zsh");
    }

    #[test]
    fn rust_template_creates_editor_test_and_git_windows() {
        let workspace = build_workspace(
            Template::Rust,
            String::from("tmux-workspace"),
            String::from("/home/test/tmux-workspace"),
        );

        assert_eq!(workspace.name, "tmux-workspace");
        assert_eq!(workspace.template, "rust");
        assert_eq!(workspace.root, "/home/test/tmux-workspace");

        assert_eq!(workspace.windows.len(), 3);

        assert_eq!(workspace.windows[0].name, "editor");
        assert_eq!(workspace.windows[0].command, "nvim .");

        assert_eq!(workspace.windows[1].name, "test");
        assert_eq!(workspace.windows[1].command, "zsh");

        assert_eq!(workspace.windows[2].name, "git");
        assert_eq!(workspace.windows[2].command, "lazygit");
    }
}
