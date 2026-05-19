use crate::workspace::{Window, Workspace};

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

pub fn build_workspace(template: &str, name: String, root: String) -> Result<Workspace, String> {
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
