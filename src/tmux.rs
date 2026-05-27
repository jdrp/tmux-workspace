use std::process::{Command, Output};

use crate::storage::{load_workspace, record_workspace_usage};
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

pub fn tmux_session_exists(name: &str) -> Result<bool, String> {
    let output = Command::new("tmux")
        .arg("has-session")
        .arg("-t")
        .arg(name)
        .output()
        .map_err(|error| format!("failed to check tmux session: {error}"))?;

    Ok(output.status.success())
}

fn primary_window_command(window: &Window) -> Option<&str> {
    if let Some(command) = window.command.as_deref() {
        return Some(command);
    }

    if let Some(first_pane) = window.panes.first() {
        return Some(first_pane.command.as_str());
    }

    None
}

fn pane_id_from_output(output: &Output, context: &str) -> Result<String, String> {
    if !output.status.success() {
        return Err(format!(
            "{context} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let pane_id = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if pane_id.is_empty() {
        return Err(format!("{context} did not return a pane id"));
    }

    Ok(pane_id)
}

fn send_command_to_pane(pane_id: &str, command: &str) -> Result<(), String> {
    let output = Command::new("tmux")
        .arg("send-keys")
        .arg("-t")
        .arg(pane_id)
        .arg("-l")
        .arg(command)
        .output()
        .map_err(|error| format!("failed to send command to tmux pane: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "tmux send-keys failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let output = Command::new("tmux")
        .arg("send-keys")
        .arg("-t")
        .arg(pane_id)
        .arg("C-m")
        .output()
        .map_err(|error| format!("failed to press enter in tmux pane: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "tmux send-keys enter failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn create_tmux_panes(session_name: &str, root: &str, window: &Window) -> Result<(), String> {
    let start_index = if window.command.is_some() { 0 } else { 1 };

    for pane in window.panes.iter().skip(start_index) {
        let output = Command::new("tmux")
            .arg("split-window")
            .arg("-P")
            .arg("-F")
            .arg("#{pane_id}")
            .arg("-t")
            .arg(format!("{}:{}", session_name, window.name))
            .arg("-c")
            .arg(root)
            .output()
            .map_err(|error| format!("failed to create tmux pane: {error}"))?;

        let pane_id = pane_id_from_output(&output, "tmux split-window")?;
        send_command_to_pane(&pane_id, &pane.command)?;
    }

    Ok(())
}

fn select_tmux_layout(session_name: &str, window: &Window) -> Result<(), String> {
    let Some(layout) = window.layout else {
        return Ok(());
    };

    let output = Command::new("tmux")
        .arg("select-layout")
        .arg("-t")
        .arg(format!("{}:{}", session_name, window.name))
        .arg(layout.tmux_name())
        .output()
        .map_err(|error| format!("failed to select tmux layout: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "tmux select-layout failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

fn create_tmux_window(session_name: &str, root: &str, window: &Window) -> Result<(), String> {
    let command = primary_window_command(window);

    let output = Command::new("tmux")
        .arg("new-window")
        .arg("-P")
        .arg("-F")
        .arg("#{pane_id}")
        .arg("-t")
        .arg(session_name)
        .arg("-c")
        .arg(root)
        .arg("-n")
        .arg(&window.name)
        .output()
        .map_err(|error| format!("failed to create tmux window: {error}"))?;

    let pane_id = pane_id_from_output(&output, "tmux new-window")?;

    if let Some(command) = command {
        send_command_to_pane(&pane_id, command)?;
    }

    create_tmux_panes(session_name, root, window)?;
    select_tmux_layout(session_name, window)?;

    Ok(())
}

fn create_tmux_session(workspace: &Workspace) -> Result<(), String> {
    let first_window = workspace
        .windows
        .first()
        .ok_or_else(|| String::from("workspace has no windows"))?;

    let first_command = primary_window_command(first_window);

    let output = Command::new("tmux")
        .arg("new-session")
        .arg("-d")
        .arg("-P")
        .arg("-F")
        .arg("#{pane_id}")
        .arg("-s")
        .arg(&workspace.name)
        .arg("-c")
        .arg(&workspace.root)
        .arg("-n")
        .arg(&first_window.name)
        .output()
        .map_err(|error| format!("failed to create tmux session: {error}"))?;

    let first_pane_id = pane_id_from_output(&output, "tmux new-session")?;

    if let Some(first_command) = first_command {
        send_command_to_pane(&first_pane_id, first_command)?;
    }

    create_tmux_panes(&workspace.name, &workspace.root, first_window)?;
    select_tmux_layout(&workspace.name, first_window)?;

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

fn print_send_command_plan(target: &str, command: &str) {
    println!("  tmux send-keys -t {target} -l '{command}'");
    println!("  tmux send-keys -t {target} C-m");
}

fn print_pane_plan(session_name: &str, root: &str, window: &Window) {
    let start_index = if window.command.is_some() { 0 } else { 1 };

    for pane in window.panes.iter().skip(start_index) {
        println!(
            "  tmux split-window -P -F '#{{pane_id}}' -t {}:{} -c {}",
            session_name, window.name, root,
        );
        print_send_command_plan("<new-pane-id>", &pane.command);
    }

    if let Some(layout) = window.layout {
        println!(
            "  tmux select-layout -t {}:{} {}",
            session_name,
            window.name,
            layout.tmux_name()
        );
    }
}

fn print_start_plan(workspace: &Workspace) {
    println!("Would start workspace: {}", workspace.name);
    println!("Root: {}", workspace.root);
    println!();
    println!("Commands:");

    if let Some(first_window) = workspace.windows.first() {
        println!(
            "  tmux new-session -d -P -F '#{{pane_id}}' -s {} -c {} -n {}",
            workspace.name, workspace.root, first_window.name
        );

        if let Some(command) = primary_window_command(first_window) {
            print_send_command_plan("<main-pane-id>", command);
        }

        print_pane_plan(&workspace.name, &workspace.root, first_window);
    }

    for window in workspace.windows.iter().skip(1) {
        println!(
            "  tmux new-window -P -F '#{{pane_id}}' -t {} -c {} -n {}",
            workspace.name, workspace.root, window.name
        );

        if let Some(command) = primary_window_command(window) {
            print_send_command_plan("<main-pane-id>", command);
        }

        print_pane_plan(&workspace.name, &workspace.root, window);
    }

    println!("  tmux attach-session -t {}", workspace.name);
}

pub fn start_workspace(name: &str, dry_run: bool) -> Result<(), String> {
    let workspace = load_workspace(name)?;

    if dry_run {
        print_start_plan(&workspace);
        return Ok(());
    }

    record_workspace_usage(&workspace.name)?;

    check_tmux_exists()?;

    let session_exists = tmux_session_exists(&workspace.name)?;

    if !session_exists {
        create_tmux_session(&workspace)?;
    }

    attach_tmux_session(&workspace.name)?;

    Ok(())
}
