use std::process::Command;

use crate::storage::workspace_file_path;

fn editor_command() -> String {
    std::env::var("EDITOR").unwrap_or_else(|_| String::from("nvim"))
}

pub fn edit_workspace(name: &str) -> Result<(), String> {
    let path = workspace_file_path(name);

    if !path.exists() {
        return Err(format!("workspace not found: {}", path.display()));
    }

    let editor = editor_command();

    let status = Command::new(&editor)
        .arg(&path)
        .status()
        .map_err(|error| format!("failed to open editor '{editor}': {error}"))?;

    if !status.success() {
        return Err(format!("editor exited with status: {status}"));
    }

    Ok(())
}
