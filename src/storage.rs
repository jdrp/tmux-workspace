use std::fs;
use std::path::{Path, PathBuf};

use crate::workspace::{Workspace, workspace_to_toml};

pub fn workspaces_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME environment variable is not set");

    PathBuf::from(home)
        .join(".config")
        .join("tmux-workspace")
        .join("workspaces")
}

pub fn workspace_file_path(name: &str) -> PathBuf {
    workspaces_dir().join(format!("{name}.toml"))
}

pub fn normalize_root(root: &str) -> Result<String, String> {
    let path = if root == "~" {
        let home = std::env::var("HOME")
            .map_err(|_| String::from("HOME environment variable is not set"))?;
        PathBuf::from(home)
    } else if let Some(rest) = root.strip_prefix("~/") {
        let home = std::env::var("HOME")
            .map_err(|_| String::from("HOME environment variable is not set"))?;
        PathBuf::from(home).join(rest)
    } else {
        PathBuf::from(root)
    };

    let absolute_path = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .map_err(|error| format!("failed to read current directory: {error}"))?
            .join(path)
    };

    let canonical_path = absolute_path
        .canonicalize()
        .map_err(|error| format!("failed to resolve root path: {error}"))?;

    Ok(canonical_path.display().to_string())
}

pub fn write_workspace_file(workspace: &Workspace) -> Result<PathBuf, String> {
    let dir = workspaces_dir();
    fs::create_dir_all(&dir)
        .map_err(|error| format!("failed to create config directory: {error}"))?;

    let path = workspace_file_path(&workspace.name);

    if path.exists() {
        return Err(format!("workspace already exists: {}", path.display()));
    }

    let toml = workspace_to_toml(workspace)
        .map_err(|error| format!("failed to serialize workspace: {error}"))?;

    fs::write(&path, toml).map_err(|error| format!("failed to write workspace file: {error}"))?;

    Ok(path)
}

pub fn read_workspace_file(path: &Path) -> Result<Workspace, String> {
    let content =
        fs::read_to_string(path).map_err(|error| format!("failed to read file: {error}"))?;

    toml::from_str::<Workspace>(&content).map_err(|error| format!("failed to parse TOML: {error}"))
}

pub fn delete_workspace_file(name: &str) -> Result<PathBuf, String> {
    let path = workspace_file_path(name);

    if !path.exists() {
        return Err(format!("workspace not found: {}", path.display()));
    }

    fs::remove_file(&path).map_err(|error| format!("failed to delete workspace file: {error}"))?;

    Ok(path)
}

pub fn list_workspaces() -> Result<Vec<Workspace>, String> {
    let dir = workspaces_dir();

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries =
        fs::read_dir(&dir).map_err(|error| format!("failed to read workspaces dir: {error}"))?;

    let mut workspaces = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|error| format!("failed to read directory entry: {error}"))?;
        let path = entry.path();

        if path.extension().and_then(|extension| extension.to_str()) != Some("toml") {
            continue;
        }

        match read_workspace_file(&path) {
            Ok(workspace) => workspaces.push(workspace),
            Err(message) => {
                println!("skipping {}: {message}", path.display());
            }
        }
    }

    Ok(workspaces)
}

pub fn print_workspace_list(workspaces: &[Workspace]) {
    if workspaces.is_empty() {
        println!("no workspaces found");
        return;
    }

    for workspace in workspaces {
        println!(
            "{}\t{}\t{}",
            workspace.name, workspace.template, workspace.root
        );
    }
}

pub fn load_workspace(name: &str) -> Result<Workspace, String> {
    let path = workspace_file_path(name);

    if !path.exists() {
        return Err(format!("workspace not found: {}", path.display()));
    }

    read_workspace_file(&path)
}
