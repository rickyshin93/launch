use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub name: String,
    pub iterm: Option<ItermConfig>,
    pub editor: Option<EditorConfig>,
    pub browser: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ItermConfig {
    pub layout: Option<String>,
    pub panes: Vec<PaneConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaneConfig {
    pub name: String,
    pub dir: String,
    pub cmd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EditorConfig {
    pub cmd: Option<String>,
    pub folders: Option<Vec<String>>,
}

/// Returns the base directory: ~/.launch/
pub fn launch_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Cannot determine home directory");
    home.join(".launch")
}

/// Returns the config file path for a project: ~/.launch/<name>.yaml
pub fn config_path(name: &str) -> PathBuf {
    launch_dir().join(format!("{name}.yaml"))
}

/// Ensure ~/.launch/ and ~/.launch/state/ directories exist
pub fn ensure_dirs() -> std::io::Result<()> {
    let base = launch_dir();
    fs::create_dir_all(&base)?;
    fs::create_dir_all(base.join("state"))?;
    Ok(())
}

/// Load and parse a project config, expanding ~ paths
pub fn load(name: &str) -> Result<Config, String> {
    let path = config_path(name);
    if !path.exists() {
        return Err(format!(
            "Config file not found: {}\nRun `launch new {name}` to create one.",
            path.display(),
        ));
    }
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    let mut config: Config = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {e}", path.display()))?;
    expand_paths(&mut config);
    Ok(config)
}

/// Expand all ~ paths in the config to absolute paths
fn expand_paths(config: &mut Config) {
    if let Some(ref mut iterm) = config.iterm {
        for pane in &mut iterm.panes {
            pane.dir = shellexpand::tilde(&pane.dir).to_string();
        }
    }
    if let Some(ref mut editor) = config.editor {
        if let Some(ref mut folders) = editor.folders {
            for folder in folders.iter_mut() {
                *folder = shellexpand::tilde(folder).to_string();
            }
        }
    }
}

/// List all project names from ~/.launch/*.yaml
pub fn list_projects() -> Vec<String> {
    let dir = launch_dir();
    let mut projects = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    projects.push(stem.to_string());
                }
            }
        }
    }
    projects.sort();
    projects
}

/// Generate a template YAML config for a new project
pub fn create_template(name: &str) -> Result<PathBuf, String> {
    ensure_dirs().map_err(|e| format!("Failed to create directories: {e}"))?;
    let path = config_path(name);
    if path.exists() {
        return Err(format!("Config already exists: {}", path.display()));
    }
    let template = format!(
        r#"name: {name}
iterm:
  # layout: vertical  # vertical(default) | grid
  panes:
    - name: dev
      dir: ~/projects/{name}
      cmd: echo "hello"
editor:
  # cmd: code  # default: code
  folders:
    - ~/projects/{name}
# browser:
#   - http://localhost:3000
"#,
    );
    fs::write(&path, &template).map_err(|e| format!("Failed to write {}: {e}", path.display()))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_yaml() {
        let yaml = r#"
name: myproject
iterm:
  layout: grid
  panes:
    - name: frontend
      dir: ~/projects/myproject/frontend
      cmd: npm run dev
    - name: backend
      dir: ~/projects/myproject/backend
      cmd: watchexec -e py python main.py
    - name: git
      dir: ~/projects/myproject
editor:
  cmd: cursor
  folders:
    - ~/projects/myproject/frontend
    - ~/projects/myproject/backend
browser:
  - http://localhost:3000
  - https://github.com/me/myproject
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.name, "myproject");

        let iterm = config.iterm.unwrap();
        assert_eq!(iterm.layout, Some("grid".to_string()));
        assert_eq!(iterm.panes.len(), 3);
        assert_eq!(iterm.panes[0].cmd, Some("npm run dev".to_string()));
        assert_eq!(iterm.panes[2].cmd, None); // git pane has no cmd

        let editor = config.editor.unwrap();
        assert_eq!(editor.cmd, Some("cursor".to_string()));
        assert_eq!(editor.folders.unwrap().len(), 2);

        assert_eq!(config.browser.unwrap().len(), 2);
    }

    #[test]
    fn parse_minimal_yaml() {
        let yaml = "name: simple\n";
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.name, "simple");
        assert!(config.iterm.is_none());
        assert!(config.editor.is_none());
        assert!(config.browser.is_none());
    }

    #[test]
    fn parse_roundtrip() {
        let yaml = r#"
name: test
iterm:
  panes:
    - name: dev
      dir: /tmp/test
      cmd: echo hi
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        let serialized = serde_yaml::to_string(&config).unwrap();
        let config2: Config = serde_yaml::from_str(&serialized).unwrap();
        assert_eq!(config, config2);
    }

    #[test]
    fn expand_tilde_paths() {
        let mut config = Config {
            name: "test".to_string(),
            iterm: Some(ItermConfig {
                layout: None,
                panes: vec![PaneConfig {
                    name: "dev".to_string(),
                    dir: "~/projects/test".to_string(),
                    cmd: None,
                }],
            }),
            editor: Some(EditorConfig {
                cmd: None,
                folders: Some(vec!["~/projects/test".to_string()]),
            }),
            browser: None,
        };

        expand_paths(&mut config);

        let home = dirs::home_dir().unwrap();
        let expected = home.join("projects/test").to_string_lossy().to_string();
        assert_eq!(config.iterm.unwrap().panes[0].dir, expected);
        assert_eq!(config.editor.unwrap().folders.unwrap()[0], expected);
    }

    #[test]
    fn launch_dir_path() {
        let dir = launch_dir();
        let home = dirs::home_dir().unwrap();
        assert_eq!(dir, home.join(".launch"));
    }

    #[test]
    fn config_path_format() {
        let path = config_path("myproject");
        assert_eq!(path, launch_dir().join("myproject.yaml"));
    }

    #[test]
    fn ensure_dirs_creates_directories() {
        ensure_dirs().unwrap();
        assert!(launch_dir().exists());
        assert!(launch_dir().join("state").exists());
    }

    #[test]
    fn create_and_load_template() {
        let name = "_launch_test_tpl";
        let path = config_path(name);
        let _ = fs::remove_file(&path);

        ensure_dirs().unwrap();
        let created = create_template(name).unwrap();
        assert!(created.exists());

        let config = load(name).unwrap();
        assert_eq!(config.name, name);
        // paths should be expanded
        if let Some(iterm) = &config.iterm {
            for pane in &iterm.panes {
                assert!(!pane.dir.contains('~'));
            }
        }

        // duplicate should fail
        assert!(create_template(name).is_err());

        let _ = fs::remove_file(&path);
    }
}
