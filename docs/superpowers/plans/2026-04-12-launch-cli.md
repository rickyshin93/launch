# Launch CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a macOS Rust CLI tool that restores full dev environments from YAML config — automating iTerm2 panes, editors, browser tabs, port checking, PID tracking, and git status.

**Architecture:** Modular single-binary CLI. `main.rs` handles CLI parsing via clap and routes to `process.rs` which orchestrates the launch/stop flows by calling independent modules (`config`, `iterm`, `editor`, `browser`, `port`, `git`, `state`). Each module wraps a single system concern.

**Tech Stack:** Rust, clap 4, serde/serde_yaml/serde_json, dialoguer (fuzzy-select), colored, shellexpand. System: osascript, open, git, lsof.

---

## File Map

| File | Responsibility |
|------|---------------|
| `Cargo.toml` | Project manifest with all dependencies |
| `src/main.rs` | CLI entry point — clap derive definitions, routes subcommands to `process.rs` |
| `src/config.rs` | `Config`, `ItermConfig`, `PaneConfig`, `EditorConfig` structs + load/save/expand paths + template generation |
| `src/state.rs` | `ProjectState` struct + read/write/delete JSON state files + PID liveness check |
| `src/git.rs` | Check git status for a list of directories, return dirty dirs with file counts |
| `src/port.rs` | Extract ports from URLs/commands, check port occupancy via lsof, kill by PID |
| `src/iterm.rs` | Generate and execute AppleScript for opening tabs/panes (vertical + grid) and closing tabs |
| `src/editor.rs` | Launch editor command with folder args |
| `src/browser.rs` | Open URLs via `open` command |
| `src/process.rs` | Orchestrate `launch`, `stop`, `list`, `edit`, `new` flows by calling other modules |

---

### Task 1: Project Scaffold + Config Module

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/config.rs`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "launch"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1"
dialoguer = { version = "0.11", features = ["fuzzy-select"] }
colored = "2"
shellexpand = "3"
regex = "1"
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 2: Create src/config.rs**

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub name: String,
    pub iterm: Option<ItermConfig>,
    pub editor: Option<EditorConfig>,
    pub browser: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItermConfig {
    pub layout: Option<String>,
    pub panes: Vec<PaneConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaneConfig {
    pub name: String,
    pub dir: String,
    pub cmd: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
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
    launch_dir().join(format!("{}.yaml", name))
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
            "Config file not found: {}\nRun `launch new {}` to create one.",
            path.display(),
            name
        ));
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    let mut config: Config = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;
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
    ensure_dirs().map_err(|e| format!("Failed to create directories: {}", e))?;
    let path = config_path(name);
    if path.exists() {
        return Err(format!("Config already exists: {}", path.display()));
    }
    let template = format!(
        r#"name: {}
iterm:
  # layout: vertical  # vertical(default) | grid
  panes:
    - name: dev
      dir: ~/projects/{}
      cmd: echo "hello"
editor:
  # cmd: code  # default: code
  folders:
    - ~/projects/{}
# browser:
#   - http://localhost:3000
"#,
        name, name, name
    );
    fs::write(&path, &template)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    Ok(path)
}
```

- [ ] **Step 3: Create minimal src/main.rs to verify compilation**

```rust
mod config;

fn main() {
    config::ensure_dirs().expect("Failed to create launch directories");
    println!("launch CLI - scaffold OK");
}
```

- [ ] **Step 4: Add `dirs` dependency to Cargo.toml**

Add `dirs = "6"` to `[dependencies]` in Cargo.toml (needed by `config.rs` for `dirs::home_dir()`).

- [ ] **Step 5: Build and verify**

Run: `cargo build`
Expected: Compiles successfully with no errors.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs src/config.rs
git commit -m "feat: scaffold project with config module"
```

---

### Task 2: State Module

**Files:**
- Create: `src/state.rs`
- Modify: `src/main.rs` (add `mod state`)

- [ ] **Step 1: Create src/state.rs**

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::config;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectState {
    pub project: String,
    pub started_at: String,
    pub panes: Vec<PaneState>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaneState {
    pub name: String,
    pub pid: u32,
}

/// Path to state file: ~/.launch/state/<project>.json
pub fn state_path(project: &str) -> PathBuf {
    config::launch_dir().join("state").join(format!("{}.json", project))
}

/// Save project state to JSON
pub fn save(state: &ProjectState) -> Result<(), String> {
    let path = state_path(&state.project);
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize state: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write state {}: {}", path.display(), e))?;
    Ok(())
}

/// Load project state from JSON
pub fn load(project: &str) -> Result<Option<ProjectState>, String> {
    let path = state_path(project);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read state {}: {}", path.display(), e))?;
    let state: ProjectState = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse state {}: {}", path.display(), e))?;
    Ok(Some(state))
}

/// Delete state file
pub fn remove(project: &str) -> Result<(), String> {
    let path = state_path(project);
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Failed to remove state {}: {}", path.display(), e))?;
    }
    Ok(())
}

/// Check if a PID is still alive
pub fn is_pid_alive(pid: u32) -> bool {
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if any PID in the state is still alive
pub fn is_running(project: &str) -> Result<bool, String> {
    match load(project)? {
        None => Ok(false),
        Some(state) => Ok(state.panes.iter().any(|p| is_pid_alive(p.pid))),
    }
}

/// List all projects that have state files
pub fn running_projects() -> Vec<String> {
    let state_dir = config::launch_dir().join("state");
    let mut projects = Vec::new();
    if let Ok(entries) = fs::read_dir(&state_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    projects.push(stem.to_string());
                }
            }
        }
    }
    projects
}
```

- [ ] **Step 2: Add mod declaration in main.rs**

Add `mod state;` after `mod config;` in `src/main.rs`.

- [ ] **Step 3: Build and verify**

Run: `cargo build`
Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add src/state.rs src/main.rs
git commit -m "feat: add state module for PID tracking"
```

---

### Task 3: Git Module

**Files:**
- Create: `src/git.rs`
- Modify: `src/main.rs` (add `mod git`)

- [ ] **Step 1: Create src/git.rs**

```rust
use std::collections::HashSet;
use std::process::Command;

pub struct DirtyRepo {
    pub dir: String,
    pub file_count: usize,
}

/// Check git status for a list of directories (deduplicated).
/// Returns list of directories with uncommitted changes.
pub fn check_status(dirs: &[String]) -> Vec<DirtyRepo> {
    let unique_dirs: HashSet<&String> = dirs.iter().collect();
    let mut dirty = Vec::new();

    for dir in unique_dirs {
        if let Some(count) = get_dirty_count(dir) {
            if count > 0 {
                dirty.push(DirtyRepo {
                    dir: dir.clone(),
                    file_count: count,
                });
            }
        }
    }
    dirty
}

/// Run `git status --porcelain` in a directory and count changed files.
/// Returns None if not a git repo or git not available.
fn get_dirty_count(dir: &str) -> Option<usize> {
    let output = Command::new("git")
        .args(["-C", dir, "status", "--porcelain"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.lines().filter(|l| !l.is_empty()).count();
    Some(count)
}
```

- [ ] **Step 2: Add mod declaration in main.rs**

Add `mod git;` after the existing mod declarations in `src/main.rs`.

- [ ] **Step 3: Build and verify**

Run: `cargo build`
Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add src/git.rs src/main.rs
git commit -m "feat: add git status checking module"
```

---

### Task 4: Port Module

**Files:**
- Create: `src/port.rs`
- Modify: `src/main.rs` (add `mod port`)

- [ ] **Step 1: Create src/port.rs**

```rust
use regex::Regex;
use std::collections::HashSet;
use std::process::Command;

pub struct PortConflict {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
}

/// Extract ports from browser URLs and pane commands.
pub fn extract_ports(urls: &[String], cmds: &[String]) -> Vec<u16> {
    let mut ports = HashSet::new();

    // From browser URLs: localhost:<port> or 127.0.0.1:<port>
    let url_re = Regex::new(r"(?:localhost|127\.0\.0\.1):(\d+)").unwrap();
    for url in urls {
        for cap in url_re.captures_iter(url) {
            if let Ok(port) = cap[1].parse::<u16>() {
                ports.insert(port);
            }
        }
    }

    // From commands: --port <N>, --port=<N>, -p <N>, -p<N>
    let cmd_re = Regex::new(r"(?:--port[=\s]|(?:^|\s)-p\s?)(\d+)").unwrap();
    for cmd in cmds {
        for cap in cmd_re.captures_iter(cmd) {
            if let Ok(port) = cap[1].parse::<u16>() {
                ports.insert(port);
            }
        }
    }

    let mut result: Vec<u16> = ports.into_iter().collect();
    result.sort();
    result
}

/// Check if a port is in use. Returns conflict info if occupied.
pub fn check_port(port: u16) -> Option<PortConflict> {
    let output = Command::new("lsof")
        .args(["-i", &format!(":{}", port), "-t"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None; // port is free
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pid: u32 = stdout.lines().next()?.trim().parse().ok()?;
    let process_name = get_process_name(pid);

    Some(PortConflict {
        port,
        pid,
        process_name,
    })
}

/// Get process name from PID
fn get_process_name(pid: u32) -> String {
    Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "comm="])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Kill a process by PID
pub fn kill_pid(pid: u32) -> bool {
    Command::new("kill")
        .args(["-9", &pid.to_string()])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
```

- [ ] **Step 2: Add mod declaration in main.rs**

Add `mod port;` after the existing mod declarations in `src/main.rs`.

- [ ] **Step 3: Build and verify**

Run: `cargo build`
Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add src/port.rs src/main.rs
git commit -m "feat: add port detection and conflict resolution module"
```

---

### Task 5: iTerm2 Module

**Files:**
- Create: `src/iterm.rs`
- Modify: `src/main.rs` (add `mod iterm`)

- [ ] **Step 1: Create src/iterm.rs**

```rust
use std::process::Command;

use crate::config::PaneConfig;

/// Open a new iTerm2 tab with panes arranged according to layout.
/// Returns the AppleScript used (for debugging).
/// `project` is used for naming: "[project] pane_name"
pub fn open_panes(project: &str, panes: &[PaneConfig], layout: &str) -> Result<(), String> {
    if panes.is_empty() {
        return Ok(());
    }

    let script = match layout {
        "grid" => build_grid_script(project, panes),
        _ => build_vertical_script(project, panes),
    };

    run_applescript(&script)
}

/// Build AppleScript for vertical (side-by-side) layout
fn build_vertical_script(project: &str, panes: &[PaneConfig]) -> String {
    let n = panes.len();
    let mut script = String::new();

    script.push_str("tell application \"iTerm2\"\n");
    script.push_str("  activate\n");
    script.push_str("  if (count of windows) = 0 then\n");
    script.push_str("    create window with default profile\n");
    script.push_str("    set theTab to current tab of current window\n");
    script.push_str("  else\n");
    script.push_str("    tell current window\n");
    script.push_str("      set theTab to (create tab with default profile)\n");
    script.push_str("    end tell\n");
    script.push_str("  end if\n");
    script.push_str("  tell current window\n");

    // Split N-1 times on the first session
    for _ in 1..n {
        script.push_str("    tell first session of theTab\n");
        script.push_str("      split vertically with default profile\n");
        script.push_str("    end tell\n");
    }

    // Configure each session
    for (i, pane) in panes.iter().enumerate() {
        let title = format!("[{}] {}", project, pane.name);
        let cmd = build_pane_command(project, pane);
        script.push_str(&format!(
            "    tell item {} of sessions of theTab\n",
            i + 1
        ));
        script.push_str(&format!("      set name to \"{}\"\n", title));
        script.push_str(&format!(
            "      write text \"{}\"\n",
            cmd.replace('\\', "\\\\").replace('"', "\\\"")
        ));
        script.push_str("    end tell\n");
    }

    script.push_str("  end tell\n");
    script.push_str("end tell\n");
    script
}

/// Build AppleScript for grid (2x2) layout
fn build_grid_script(project: &str, panes: &[PaneConfig]) -> String {
    let n = panes.len();

    if n <= 2 {
        // 2 or fewer: just split vertically
        return build_vertical_script(project, panes);
    }

    let mut script = String::new();

    script.push_str("tell application \"iTerm2\"\n");
    script.push_str("  activate\n");
    script.push_str("  if (count of windows) = 0 then\n");
    script.push_str("    create window with default profile\n");
    script.push_str("    set theTab to current tab of current window\n");
    script.push_str("  else\n");
    script.push_str("    tell current window\n");
    script.push_str("      set theTab to (create tab with default profile)\n");
    script.push_str("    end tell\n");
    script.push_str("  end if\n");
    script.push_str("  tell current window\n");

    // Step 1: split vertically (left | right)
    script.push_str("    tell first session of theTab\n");
    script.push_str("      split vertically with default profile\n");
    script.push_str("    end tell\n");

    // Step 2: split left horizontally (top-left | bottom-left)
    script.push_str("    tell first session of theTab\n");
    script.push_str("      split horizontally with default profile\n");
    script.push_str("    end tell\n");

    // Step 3: if 4+ panes, split right horizontally (top-right | bottom-right)
    if n >= 4 {
        // After step 2, sessions are: [top-left, bottom-left, right]
        // We need to split the right (last) session
        script.push_str("    tell last session of theTab\n");
        script.push_str("      split horizontally with default profile\n");
        script.push_str("    end tell\n");
    }

    // Step 4: if 5+ panes, add extras as vertical splits on the last session
    for _ in 4..n {
        script.push_str("    tell last session of theTab\n");
        script.push_str("      split vertically with default profile\n");
        script.push_str("    end tell\n");
    }

    // Configure each session
    for (i, pane) in panes.iter().enumerate() {
        let title = format!("[{}] {}", project, pane.name);
        let cmd = build_pane_command(project, pane);
        script.push_str(&format!(
            "    tell item {} of sessions of theTab\n",
            i + 1
        ));
        script.push_str(&format!("      set name to \"{}\"\n", title));
        script.push_str(&format!(
            "      write text \"{}\"\n",
            cmd.replace('\\', "\\\\").replace('"', "\\\"")
        ));
        script.push_str("    end tell\n");
    }

    script.push_str("  end tell\n");
    script.push_str("end tell\n");
    script
}

/// Build the shell command string for a pane.
/// If cmd is set, wraps with PID tracking: echo $$ > pidfile && exec <cmd>
/// If cmd is None, just cd to directory
fn build_pane_command(project: &str, pane: &PaneConfig) -> String {
    match &pane.cmd {
        Some(cmd) => {
            let pid_file = format!("/tmp/.launch_{}_{}.pid", project, pane.name);
            format!(
                "cd {} && echo $$ > {} && exec {}",
                pane.dir, pid_file, cmd
            )
        }
        None => format!("cd {}", pane.dir),
    }
}

/// Close iTerm2 tabs whose sessions have names starting with "[project]"
pub fn close_tabs(project: &str) -> Result<(), String> {
    let prefix = format!("[{}]", project);
    let script = format!(
        r#"tell application "iTerm2"
  if (count of windows) > 0 then
    tell current window
      set tabsToClose to {{}}
      repeat with t in tabs
        repeat with s in sessions of t
          if name of s starts with "{}" then
            set end of tabsToClose to t
            exit repeat
          end if
        end repeat
      end repeat
      repeat with t in tabsToClose
        close t
      end repeat
    end tell
  end if
end tell"#,
        prefix
    );

    // Ignore errors - tab may already be closed
    let _ = run_applescript(&script);
    Ok(())
}

/// Execute an AppleScript string via osascript
fn run_applescript(script: &str) -> Result<(), String> {
    let output = Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("Failed to run osascript: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("AppleScript error: {}", stderr));
    }
    Ok(())
}
```

- [ ] **Step 2: Add mod declaration in main.rs**

Add `mod iterm;` after the existing mod declarations in `src/main.rs`.

- [ ] **Step 3: Build and verify**

Run: `cargo build`
Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add src/iterm.rs src/main.rs
git commit -m "feat: add iTerm2 AppleScript control module"
```

---

### Task 6: Editor + Browser Modules

**Files:**
- Create: `src/editor.rs`
- Create: `src/browser.rs`
- Modify: `src/main.rs` (add mods)

- [ ] **Step 1: Create src/editor.rs**

```rust
use std::process::Command;

use crate::config::EditorConfig;

/// Open editor with the configured folders.
/// Skips if editor section is None or folders is empty.
pub fn open(editor: &Option<EditorConfig>) -> Result<(), String> {
    let editor = match editor {
        Some(e) => e,
        None => return Ok(()),
    };

    let folders = match &editor.folders {
        Some(f) if !f.is_empty() => f,
        _ => return Ok(()),
    };

    let cmd = editor.cmd.as_deref().unwrap_or("code");

    Command::new(cmd)
        .args(folders)
        .spawn()
        .map_err(|e| format!("Failed to launch editor '{}': {}", cmd, e))?;

    Ok(())
}
```

- [ ] **Step 2: Create src/browser.rs**

```rust
use std::process::Command;

/// Open a list of URLs in the default browser using `open` command.
pub fn open(urls: &Option<Vec<String>>) -> Result<(), String> {
    let urls = match urls {
        Some(u) if !u.is_empty() => u,
        _ => return Ok(()),
    };

    for url in urls {
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL '{}': {}", url, e))?;
    }

    Ok(())
}
```

- [ ] **Step 3: Add mod declarations in main.rs**

Add `mod editor;` and `mod browser;` after the existing mod declarations in `src/main.rs`.

- [ ] **Step 4: Build and verify**

Run: `cargo build`
Expected: Compiles successfully.

- [ ] **Step 5: Commit**

```bash
git add src/editor.rs src/browser.rs src/main.rs
git commit -m "feat: add editor and browser launch modules"
```

---

### Task 7: Process Orchestration Module

**Files:**
- Create: `src/process.rs`
- Modify: `src/main.rs` (add `mod process`)

- [ ] **Step 1: Create src/process.rs**

```rust
use std::io::{self, Write};
use std::process::Command;
use std::thread;
use std::time::Duration;

use colored::Colorize;

use crate::{browser, config, editor, git, iterm, port, state};

/// Main launch flow for a project
pub fn launch(name: &str) -> Result<(), String> {
    config::ensure_dirs().map_err(|e| e.to_string())?;
    let cfg = config::load(name)?;

    // Check if already running
    if state::is_running(name)? {
        println!(
            "{}",
            format!("Project '{}' is already running.", name).yellow()
        );
        print!("Restart? [Y/n] ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();
        if input == "n" || input == "no" {
            println!("Aborted.");
            return Ok(());
        }
        stop(name)?;
    }

    // Git status check
    if let Some(ref iterm) = cfg.iterm {
        let dirs: Vec<String> = iterm.panes.iter().map(|p| p.dir.clone()).collect();
        let dirty = git::check_status(&dirs);
        if !dirty.is_empty() {
            for d in &dirty {
                println!(
                    "{}",
                    format!("  {} has {} uncommitted file(s)", d.dir, d.file_count).yellow()
                );
            }
            print!("Continue? [Y/n] ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();
            if input == "n" || input == "no" {
                println!("Aborted.");
                return Ok(());
            }
        }
    }

    // Port conflict check
    let mut urls: Vec<String> = cfg.browser.clone().unwrap_or_default();
    let cmds: Vec<String> = cfg
        .iterm
        .as_ref()
        .map(|i| i.panes.iter().filter_map(|p| p.cmd.clone()).collect())
        .unwrap_or_default();
    let ports = port::extract_ports(&urls, &cmds);

    for p in &ports {
        if let Some(conflict) = port::check_port(*p) {
            println!(
                "{}",
                format!(
                    "  Port {} is occupied (process: {}, PID: {})",
                    conflict.port, conflict.process_name, conflict.pid
                )
                .yellow()
            );
            print!("[K]ill / [S]kip / [A]bort? ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            match input.trim().to_lowercase().as_str() {
                "k" | "kill" => {
                    port::kill_pid(conflict.pid);
                    println!("  Killed PID {}", conflict.pid);
                }
                "a" | "abort" => {
                    println!("Aborted.");
                    return Ok(());
                }
                _ => {
                    println!("  Skipped port {}", p);
                }
            }
        }
    }

    // Open iTerm2 panes
    if let Some(ref iterm_cfg) = cfg.iterm {
        let layout = iterm_cfg.layout.as_deref().unwrap_or("vertical");
        iterm::open_panes(name, &iterm_cfg.panes, layout)?;

        // Collect PIDs from pid files
        let pane_states = collect_pids(name, &iterm_cfg.panes);
        if !pane_states.is_empty() {
            let project_state = state::ProjectState {
                project: name.to_string(),
                started_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                panes: pane_states,
            };
            state::save(&project_state)?;
        }
    }

    // Open editor
    editor::open(&cfg.editor)?;

    // Open browser
    browser::open(&cfg.browser)?;

    println!("{}", format!("Project '{}' launched!", name).green());
    Ok(())
}

/// Poll for PID files after iTerm2 panes are opened
fn collect_pids(project: &str, panes: &[config::PaneConfig]) -> Vec<state::PaneState> {
    let mut results = Vec::new();

    for pane in panes {
        if pane.cmd.is_none() {
            continue;
        }
        let pid_file = format!("/tmp/.launch_{}_{}.pid", project, pane.name);
        let pid = poll_pid_file(&pid_file);
        if let Some(pid) = pid {
            results.push(state::PaneState {
                name: pane.name.clone(),
                pid,
            });
        }
    }

    results
}

/// Poll for a PID file, checking every 100ms for up to 3 seconds
fn poll_pid_file(path: &str) -> Option<u32> {
    for _ in 0..30 {
        if let Ok(content) = std::fs::read_to_string(path) {
            let pid = content.trim().parse::<u32>().ok();
            if pid.is_some() {
                let _ = std::fs::remove_file(path);
                return pid;
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    None
}

/// Stop a project: kill processes, close iTerm2 tabs, remove state
pub fn stop(name: &str) -> Result<(), String> {
    let state = state::load(name)?;
    match state {
        None => {
            println!("Project '{}' is not running.", name);
            return Ok(());
        }
        Some(s) => {
            for pane in &s.panes {
                kill_process_group(pane.pid);
            }
        }
    }

    iterm::close_tabs(name)?;
    state::remove(name)?;
    println!("{}", format!("Project '{}' stopped.", name).green());
    Ok(())
}

/// Stop all running projects
pub fn stop_all() -> Result<(), String> {
    let projects = state::running_projects();
    if projects.is_empty() {
        println!("No projects are running.");
        return Ok(());
    }
    for project in &projects {
        stop(project)?;
    }
    Ok(())
}

/// Kill a process group: SIGTERM first, wait 3s, then SIGKILL if needed
fn kill_process_group(pid: u32) {
    // Get PGID
    let pgid = Command::new("ps")
        .args(["-o", "pgid=", "-p", &pid.to_string()])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse::<u32>()
                    .ok()
            } else {
                None
            }
        });

    let target = match pgid {
        Some(g) => format!("-{}", g),
        None => pid.to_string(),
    };

    // SIGTERM
    let _ = Command::new("kill")
        .args(["--", &target])
        .output();

    thread::sleep(Duration::from_secs(3));

    // Check if still alive, SIGKILL if needed
    if state::is_pid_alive(pid) {
        let _ = Command::new("kill")
            .args(["-9", "--", &target])
            .output();
    }
}

/// List all projects and their status
pub fn list() -> Result<(), String> {
    config::ensure_dirs().map_err(|e| e.to_string())?;
    let projects = config::list_projects();

    if projects.is_empty() {
        println!("No projects configured. Run `launch new <name>` to create one.");
        return Ok(());
    }

    println!("{:<20} {:<12} {}", "Project", "Status", "Panes");
    println!("{}", "-".repeat(50));

    for project in &projects {
        let (status, pane_names) = match state::load(project)? {
            Some(s) => {
                let alive: Vec<&str> = s
                    .panes
                    .iter()
                    .filter(|p| state::is_pid_alive(p.pid))
                    .map(|p| p.name.as_str())
                    .collect();
                if alive.is_empty() {
                    ("stopped".to_string(), "-".to_string())
                } else {
                    ("running".green().to_string(), alive.join(", "))
                }
            }
            None => ("stopped".to_string(), "-".to_string()),
        };
        println!("{:<20} {:<12} {}", project, status, pane_names);
    }
    Ok(())
}

/// Edit a project config in $EDITOR
pub fn edit(name: &str) -> Result<(), String> {
    let path = config::config_path(name);
    if !path.exists() {
        return Err(format!(
            "Config not found: {}\nRun `launch new {}` to create one.",
            path.display(),
            name
        ));
    }

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    Command::new(&editor)
        .arg(path.to_str().unwrap())
        .status()
        .map_err(|e| format!("Failed to open editor '{}': {}", editor, e))?;
    Ok(())
}

/// Create a new project config from template
pub fn new_project(name: &str) -> Result<(), String> {
    let path = config::create_template(name)?;
    println!(
        "{}",
        format!("Created config: {}", path.display()).green()
    );

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    Command::new(&editor)
        .arg(path.to_str().unwrap())
        .status()
        .map_err(|e| format!("Failed to open editor '{}': {}", editor, e))?;
    Ok(())
}
```

- [ ] **Step 2: Add mod declaration in main.rs**

Add `mod process;` after the existing mod declarations in `src/main.rs`.

- [ ] **Step 3: Build and verify**

Run: `cargo build`
Expected: Compiles successfully.

- [ ] **Step 4: Commit**

```bash
git add src/process.rs src/main.rs
git commit -m "feat: add process orchestration module"
```

---

### Task 8: CLI Entry Point (main.rs with clap)

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Rewrite src/main.rs with full clap CLI**

```rust
mod browser;
mod config;
mod editor;
mod git;
mod iterm;
mod port;
mod process;
mod state;

use clap::{Parser, Subcommand};
use dialoguer::FuzzySelect;

#[derive(Parser)]
#[command(name = "launch", about = "One-command dev environment launcher")]
struct Cli {
    /// Project name to launch (or use subcommand)
    project: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Stop project services
    Stop {
        /// Project name to stop
        project: Option<String>,
        /// Stop all running projects
        #[arg(long)]
        all: bool,
    },
    /// List all projects and their status
    List,
    /// Edit project config in $EDITOR
    Edit {
        /// Project name
        project: String,
    },
    /// Create new project config from template
    New {
        /// Project name
        project: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Stop { project, all }) => {
            if all {
                process::stop_all()
            } else if let Some(name) = project {
                process::stop(&name)
            } else {
                Err("Usage: launch stop <project> or launch stop --all".to_string())
            }
        }
        Some(Commands::List) => process::list(),
        Some(Commands::Edit { project }) => process::edit(&project),
        Some(Commands::New { project }) => process::new_project(&project),
        None => {
            if let Some(name) = cli.project {
                process::launch(&name)
            } else {
                // No args: fuzzy select
                fuzzy_select()
            }
        }
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn fuzzy_select() -> Result<(), String> {
    config::ensure_dirs().map_err(|e| e.to_string())?;
    let projects = config::list_projects();
    if projects.is_empty() {
        return Err("No projects configured. Run `launch new <name>` to create one.".to_string());
    }

    let selection = FuzzySelect::new()
        .with_prompt("Select project")
        .items(&projects)
        .interact_opt()
        .map_err(|e| format!("Selection error: {}", e))?;

    match selection {
        Some(idx) => process::launch(&projects[idx]),
        None => {
            println!("Cancelled.");
            Ok(())
        }
    }
}
```

- [ ] **Step 2: Build and verify**

Run: `cargo build`
Expected: Compiles successfully.

- [ ] **Step 3: Test basic CLI help**

Run: `cargo run -- --help`
Expected: Shows help with commands: stop, list, edit, new, and positional project arg.

- [ ] **Step 4: Test subcommand help**

Run: `cargo run -- stop --help`
Expected: Shows stop subcommand help with --all flag and project argument.

- [ ] **Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: add clap CLI entry point with all subcommands"
```

---

### Task 9: Integration Test — End-to-End Smoke Test

**Files:**
- Create: `tests/integration.rs`

- [ ] **Step 1: Create tests/integration.rs**

This tests the config module and port extraction logic (things we can test without iTerm2).

```rust
use std::fs;
use std::path::PathBuf;

fn test_launch_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("launch_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("state")).unwrap();
    dir
}

#[test]
fn test_yaml_roundtrip() {
    let yaml = r#"
name: testproject
iterm:
  layout: vertical
  panes:
    - name: frontend
      dir: /tmp/test
      cmd: echo hello
    - name: backend
      dir: /tmp/test2
editor:
  cmd: code
  folders:
    - /tmp/test
    - /tmp/test2
browser:
  - http://localhost:3000
  - https://github.com/test
"#;

    let config: launch::config::Config = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.name, "testproject");

    let iterm = config.iterm.unwrap();
    assert_eq!(iterm.layout, Some("vertical".to_string()));
    assert_eq!(iterm.panes.len(), 2);
    assert_eq!(iterm.panes[0].name, "frontend");
    assert_eq!(iterm.panes[0].cmd, Some("echo hello".to_string()));
    assert_eq!(iterm.panes[1].cmd, None);

    let editor = config.editor.unwrap();
    assert_eq!(editor.cmd, Some("code".to_string()));
    assert_eq!(editor.folders.unwrap().len(), 2);

    let browser = config.browser.unwrap();
    assert_eq!(browser.len(), 2);
}

#[test]
fn test_port_extraction() {
    let urls = vec![
        "http://localhost:3000".to_string(),
        "https://github.com/test".to_string(),
        "http://127.0.0.1:8080/api".to_string(),
    ];
    let cmds = vec![
        "npm run dev".to_string(),
        "python main.py --port 5000".to_string(),
        "server --port=9090".to_string(),
        "redis-server -p 6379".to_string(),
    ];

    let ports = launch::port::extract_ports(&urls, &cmds);
    assert!(ports.contains(&3000));
    assert!(ports.contains(&8080));
    assert!(ports.contains(&5000));
    assert!(ports.contains(&9090));
    assert!(ports.contains(&6379));
    // github URL has no port
    assert_eq!(ports.len(), 5);
}

#[test]
fn test_state_roundtrip() {
    let dir = test_launch_dir();
    let state_file = dir.join("state").join("test.json");

    let state = launch::state::ProjectState {
        project: "test".to_string(),
        started_at: "2026-04-12T10:00:00".to_string(),
        panes: vec![launch::state::PaneState {
            name: "dev".to_string(),
            pid: 99999,
        }],
    };

    let json = serde_json::to_string_pretty(&state).unwrap();
    fs::write(&state_file, &json).unwrap();

    let content = fs::read_to_string(&state_file).unwrap();
    let loaded: launch::state::ProjectState = serde_json::from_str(&content).unwrap();
    assert_eq!(loaded.project, "test");
    assert_eq!(loaded.panes.len(), 1);
    assert_eq!(loaded.panes[0].pid, 99999);

    fs::remove_dir_all(&dir).unwrap();
}
```

- [ ] **Step 2: Update Cargo.toml to expose library for integration tests**

Add a `[lib]` section to `Cargo.toml`:

```toml
[lib]
name = "launch"
path = "src/lib.rs"

[[bin]]
name = "launch"
path = "src/main.rs"
```

- [ ] **Step 3: Create src/lib.rs to re-export modules**

```rust
pub mod config;
pub mod state;
pub mod git;
pub mod port;
pub mod iterm;
pub mod editor;
pub mod browser;
pub mod process;
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: All 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs tests/integration.rs
git commit -m "test: add integration tests for config, port, and state"
```

---

### Task 10: README + Final Polish

**Files:**
- Create: `README.md`

- [ ] **Step 1: Create README.md**

```markdown
# launch

A macOS CLI tool to restore your full dev environment with one command.

## Install

```bash
cargo install --path .
```

## Quick Start

```bash
# Create a project config
launch new myproject

# Edit the config
launch edit myproject

# Launch the project
launch myproject

# See all projects
launch list

# Stop the project
launch stop myproject

# Stop all projects
launch stop --all

# Fuzzy select (no args)
launch
```

## Configuration

Configs live in `~/.launch/<project>.yaml`:

```yaml
name: myproject
iterm:
  layout: vertical  # vertical (default) | grid
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
  cmd: cursor  # default: code
  folders:
    - ~/projects/myproject/frontend
    - ~/projects/myproject/backend
browser:
  - http://localhost:3000
  - https://github.com/me/myproject
```

## Features

- **iTerm2 Panes** — Opens a tab per project, splits panes with auto-naming `[project] pane`
- **Layouts** — `vertical` (side-by-side, default) or `grid` (2x2)
- **Editor** — Opens configured editor with project folders
- **Browser** — Opens URLs in default browser
- **Port Conflict Detection** — Auto-detects ports from URLs/commands, warns on conflicts
- **Git Status** — Warns about uncommitted changes before launch
- **Process Tracking** — Tracks PIDs for clean `launch stop`
- **Fuzzy Select** — Run `launch` with no args to pick a project

## Requirements

- macOS
- iTerm2
- Rust (for building)
```

- [ ] **Step 2: Build release binary**

Run: `cargo build --release`
Expected: Compiles successfully. Binary at `target/release/launch`.

- [ ] **Step 3: Run all tests one final time**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs: add README with install and usage instructions"
```
