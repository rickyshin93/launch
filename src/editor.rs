use anyhow::{Context, Result};
use std::process::Command;

use crate::config::EditorConfig;

/// Open editor with the configured folders.
/// Skips if editor section is None or folders is empty.
pub fn open(editor: Option<&EditorConfig>) -> Result<()> {
    let Some(editor) = editor else {
        return Ok(());
    };

    let folders = match &editor.folders {
        Some(f) if !f.is_empty() => f,
        _ => return Ok(()),
    };

    let cmd = editor.cmd.as_deref().unwrap_or("code");

    Command::new(cmd)
        .args(folders)
        .spawn()
        .with_context(|| format!("Failed to launch editor '{cmd}'"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_editor_is_ok() {
        assert!(open(None).is_ok());
    }

    #[test]
    fn empty_folders_is_ok() {
        let editor = EditorConfig {
            cmd: Some("code".to_string()),
            folders: Some(vec![]),
        };
        assert!(open(Some(&editor)).is_ok());
    }

    #[test]
    fn no_folders_field_is_ok() {
        let editor = EditorConfig {
            cmd: Some("code".to_string()),
            folders: None,
        };
        assert!(open(Some(&editor)).is_ok());
    }
}
