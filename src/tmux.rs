use std::process::Command;

use crate::storage::load_workspace;
use crate::workspace::{Window, Workspace};

fn check_tmux_exists() -> Result<(), String> {
    let output = Command::new("tmux")
        .arg("-V")
        .output()
        .map_err(|error| format!("failed to run tmux: {error}"))?;

    if !output.status.success() {
        return Err(format!("tmux exited with status: {}", output.status));
    }

    Ok(())
}

fn tmux_session_exists(name: &str) -> Result<bool, String> {
    let output = Command::new("tmux")
        .arg("has-session")
        .arg("-t")
        .arg(name)
        .output()
        .map_err(|error| format!("failed to check tmux session: {error}"))?;

    Ok(output.status.success())
}

fn create_tmux_window(session_name: &str, root: &str, window: &Window) -> Result<(), String> {
    let output = Command::new("tmux")
        .arg("new-window")
        .arg("-t")
        .arg(session_name)
        .arg("-c")
        .arg(root)
        .arg("-n")
        .arg(&window.name)
        .arg(&window.command)
        .output()
        .map_err(|error| format!("failed to create tmux window: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "tmux new-window failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn create_tmux_session(workspace: &Workspace) -> Result<(), String> {
    let first_window = workspace
        .windows
        .first()
        .ok_or_else(|| String::from("workspace has no windows"))?;

    let output = Command::new("tmux")
        .arg("new-session")
        .arg("-d")
        .arg("-s")
        .arg(&workspace.name)
        .arg("-c")
        .arg(&workspace.root)
        .arg("-n")
        .arg(&first_window.name)
        .arg(&first_window.command)
        .output()
        .map_err(|error| format!("failed to create tmux session: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "tmux new-session failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    for window in workspace.windows.iter().skip(1) {
        create_tmux_window(&workspace.name, &workspace.root, window)?;
    }

    Ok(())
}

fn attach_tmux_session(session_name: &str) -> Result<(), String> {
    let inside_tmux = std::env::var("TMUX").is_ok();

    let status = if inside_tmux {
        Command::new("tmux")
            .arg("switch-client")
            .arg("-t")
            .arg(session_name)
            .status()
            .map_err(|error| format!("failed to switch tmux client: {error}"))?
    } else {
        Command::new("tmux")
            .arg("attach-session")
            .arg("-t")
            .arg(session_name)
            .status()
            .map_err(|error| format!("failed to attach tmux session: {error}"))?
    };

    if !status.success() {
        return Err(format!("tmux attach failed with status: {status}"));
    }

    Ok(())
}

pub fn start_workspace(name: &str) -> Result<(), String> {
    let workspace = load_workspace(name)?;

    check_tmux_exists()?;

    let session_exists = tmux_session_exists(&workspace.name)?;

    if !session_exists {
        create_tmux_session(&workspace)?;
    }

    attach_tmux_session(&workspace.name)?;

    Ok(())
}
