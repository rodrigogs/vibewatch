use assert_fs::TempDir;
use assert_fs::prelude::*;
use std::time::Duration;

/// Time allowed for watcher to start and initialize (6 seconds for CI stability)
pub const WATCHER_STARTUP_TIME: Duration = Duration::from_millis(6000);

/// Time allowed for watcher to detect and process filesystem events (6 seconds for CI)
pub const EVENT_DETECTION_TIME: Duration = Duration::from_millis(6000);

/// Timeout for command execution
/// Maximum time to wait for a command to complete
pub const COMMAND_EXECUTION_TIME: Duration = Duration::from_millis(500);

/// Creates a temporary directory for testing
///
/// This directory will be automatically cleaned up when dropped
pub fn setup_test_dir() -> TempDir {
    TempDir::new().unwrap()
}

/// Helper to create a test file with content
///
/// # Arguments
/// * `dir` - The temporary directory to create the file in
/// * `name` - The name/path of the file relative to the directory
/// * `content` - The content to write to the file
pub fn create_test_file(dir: &TempDir, name: &str, content: &str) {
    dir.child(name).write_str(content).unwrap();
}

/// Helper to create multiple test files at once
///
/// # Arguments
/// * `dir` - The temporary directory to create files in
/// * `files` - Slice of (name, content) tuples
pub fn create_test_files(dir: &TempDir, files: &[(&str, &str)]) {
    for (name, content) in files {
        create_test_file(dir, name, content);
    }
}

/// Helper to modify a test file
///
/// # Arguments
/// * `dir` - The temporary directory containing the file
/// * `name` - The name/path of the file relative to the directory
/// * `new_content` - The new content to write to the file
pub fn modify_test_file(dir: &TempDir, name: &str, new_content: &str) {
    dir.child(name).write_str(new_content).unwrap();
}

/// Helper to delete a test file
///
/// # Arguments
/// * `dir` - The temporary directory containing the file
/// * `name` - The name/path of the file relative to the directory
pub fn delete_test_file(dir: &TempDir, name: &str) {
    std::fs::remove_file(dir.child(name).path()).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_test_dir() {
        let dir = setup_test_dir();
        assert!(dir.path().exists());
    }

    #[test]
    fn test_create_test_file() {
        let dir = setup_test_dir();
        create_test_file(&dir, "test.txt", "Hello, World!");

        let content = std::fs::read_to_string(dir.child("test.txt").path()).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_create_multiple_files() {
        let dir = setup_test_dir();
        create_test_files(
            &dir,
            &[("file1.txt", "content1"), ("file2.txt", "content2")],
        );

        assert!(dir.child("file1.txt").path().exists());
        assert!(dir.child("file2.txt").path().exists());
    }

    #[test]
    fn test_modify_test_file() {
        let dir = setup_test_dir();
        create_test_file(&dir, "test.txt", "Initial");
        modify_test_file(&dir, "test.txt", "Modified");

        let content = std::fs::read_to_string(dir.child("test.txt").path()).unwrap();
        assert_eq!(content, "Modified");
    }

    #[test]
    fn test_delete_test_file() {
        let dir = setup_test_dir();
        create_test_file(&dir, "test.txt", "Delete me");
        assert!(dir.child("test.txt").path().exists());

        delete_test_file(&dir, "test.txt");
        assert!(!dir.child("test.txt").path().exists());
    }
}
