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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn clean_repo_returns_empty() {
        // Use a temp dir that is a git repo with no changes
        let tmp = std::env::temp_dir().join("_launch_test_git_clean");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        Command::new("git")
            .args(["init"])
            .current_dir(&tmp)
            .output()
            .unwrap();

        // Create and commit a file so the repo is clean
        fs::write(tmp.join("hello.txt"), "hello").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(&tmp)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(&tmp)
            .output()
            .unwrap();

        let dirs = vec![tmp.to_string_lossy().to_string()];
        let dirty = check_status(&dirs);
        assert!(dirty.is_empty());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn dirty_repo_returns_count() {
        let tmp = std::env::temp_dir().join("_launch_test_git_dirty");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        Command::new("git")
            .args(["init"])
            .current_dir(&tmp)
            .output()
            .unwrap();

        // Create uncommitted files
        fs::write(tmp.join("a.txt"), "a").unwrap();
        fs::write(tmp.join("b.txt"), "b").unwrap();

        let dirs = vec![tmp.to_string_lossy().to_string()];
        let dirty = check_status(&dirs);
        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0].file_count, 2);

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn non_git_dir_ignored() {
        let tmp = std::env::temp_dir().join("_launch_test_git_nongit");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let dirs = vec![tmp.to_string_lossy().to_string()];
        let dirty = check_status(&dirs);
        assert!(dirty.is_empty());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn deduplicates_dirs() {
        let tmp = std::env::temp_dir().join("_launch_test_git_dedup");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        Command::new("git")
            .args(["init"])
            .current_dir(&tmp)
            .output()
            .unwrap();
        fs::write(tmp.join("a.txt"), "a").unwrap();

        let dir = tmp.to_string_lossy().to_string();
        let dirs = vec![dir.clone(), dir.clone(), dir];
        let dirty = check_status(&dirs);
        // Should only check once, not 3 times
        assert_eq!(dirty.len(), 1);

        let _ = fs::remove_dir_all(&tmp);
    }
}
