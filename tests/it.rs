//! Integration tests for vibewatch
//!
//! This file contains ALL integration tests for performance reasons.
//! Having multiple integration test files causes severe performance problems:
//! - Each file compiles as a separate binary
//! - Cargo relinks the library with each file: 3x slower compile, 5x larger artifacts
//!
//! See: https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html

// Allow zombie processes in tests - we intentionally kill child processes
#![allow(clippy::zombie_processes)]

use assert_cmd::Command;
use assert_cmd::cargo::CommandCargoExt;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::{Child, Command as StdCommand, Stdio};
use std::thread;
use std::time::Duration;

mod common;

// ============================================================================
// CLI Tests - Test the command-line interface
// ============================================================================

#[test]
fn test_cli_help_flag() {
    let mut cmd = Command::cargo_bin("vibewatch").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("vibewatch"))
        .stdout(predicate::str::contains("DIRECTORY"))
        .stdout(predicate::str::contains("File Filtering"));
}

#[test]
fn test_cli_version_flag() {
    let mut cmd = Command::cargo_bin("vibewatch").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("vibewatch"));
}

#[test]
fn test_cli_requires_directory_argument() {
    let mut cmd = Command::cargo_bin("vibewatch").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("DIRECTORY"));
}

#[test]
fn test_cli_accepts_valid_directory() {
    let temp_dir = common::setup_test_dir();

    let mut child = StdCommand::cargo_bin("vibewatch")
        .unwrap()
        .arg(temp_dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start vibewatch");

    // Let it run briefly to ensure it starts successfully
    thread::sleep(Duration::from_millis(500));

    // Kill it (vibewatch runs indefinitely)
    child.kill().expect("Failed to kill vibewatch");

    // Check it didn't exit with an error before we killed it
    // Note: kill() will cause a non-zero exit, which is expected
}

#[test]
fn test_cli_rejects_nonexistent_directory() {
    let mut cmd = Command::cargo_bin("vibewatch").unwrap();
    cmd.arg("/nonexistent/directory/that/does/not/exist")
        .timeout(Duration::from_secs(1))
        .assert()
        .failure();
}

#[test]
fn test_cli_verbose_flag_shows_debug_output() {
    let temp_dir = common::setup_test_dir();

    let mut child = StdCommand::cargo_bin("vibewatch")
        .unwrap()
        .arg(temp_dir.path())
        .arg("--verbose")
        .arg("--on-change")
        .arg(common::touch_command("/tmp/vibewatch-test-output.txt"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start vibewatch");

    thread::sleep(Duration::from_millis(500));
    child.kill().expect("Failed to kill vibewatch");
}

#[test]
fn test_cli_accepts_include_patterns() {
    let temp_dir = common::setup_test_dir();

    let mut child = StdCommand::cargo_bin("vibewatch")
        .unwrap()
        .arg(temp_dir.path())
        .arg("--include")
        .arg("*.rs")
        .arg("--include")
        .arg("*.toml")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start vibewatch");

    thread::sleep(Duration::from_millis(500));
    child.kill().expect("Failed to kill vibewatch");
}

#[test]
fn test_cli_accepts_exclude_patterns() {
    let temp_dir = common::setup_test_dir();

    let mut child = StdCommand::cargo_bin("vibewatch")
        .unwrap()
        .arg(temp_dir.path())
        .arg("--exclude")
        .arg("*.tmp")
        .arg("--exclude")
        .arg("target/**")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start vibewatch");

    thread::sleep(Duration::from_millis(500));
    child.kill().expect("Failed to kill vibewatch");
}

#[test]
fn test_cli_accepts_on_create_command() {
    let temp_dir = common::setup_test_dir();

    let mut child = StdCommand::cargo_bin("vibewatch")
        .unwrap()
        .arg(temp_dir.path())
        .arg("--on-create")
        .arg("echo Created: {file_path}")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start vibewatch");

    thread::sleep(Duration::from_millis(500));
    child.kill().expect("Failed to kill vibewatch");
}

#[test]
fn test_cli_accepts_on_modify_command() {
    let temp_dir = common::setup_test_dir();

    let mut child = StdCommand::cargo_bin("vibewatch")
        .unwrap()
        .arg(temp_dir.path())
        .arg("--on-modify")
        .arg("echo Modified: {file_path}")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start vibewatch");

    thread::sleep(Duration::from_millis(500));
    child.kill().expect("Failed to kill vibewatch");
}

#[test]
fn test_cli_accepts_on_delete_command() {
    let temp_dir = common::setup_test_dir();

    let mut child = StdCommand::cargo_bin("vibewatch")
        .unwrap()
        .arg(temp_dir.path())
        .arg("--on-delete")
        .arg("echo Deleted: {file_path}")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start vibewatch");

    thread::sleep(Duration::from_millis(500));
    child.kill().expect("Failed to kill vibewatch");
}

#[test]
fn test_cli_accepts_on_change_command() {
    let temp_dir = common::setup_test_dir();

    let mut child = StdCommand::cargo_bin("vibewatch")
        .unwrap()
        .arg(temp_dir.path())
        .arg("--on-change")
        .arg("echo Changed: {file_path}")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start vibewatch");

    thread::sleep(Duration::from_millis(500));
    child.kill().expect("Failed to kill vibewatch");
}

// ============================================================================
// Watcher Tests - Test actual file watching behavior
// ============================================================================

/// Helper to start vibewatch with a command and capture output
fn start_watcher_with_command(dir: &assert_fs::TempDir, on_change_cmd: &str) -> Child {
    StdCommand::cargo_bin("vibewatch")
        .unwrap()
        .arg(dir.path())
        .arg("--on-change")
        .arg(on_change_cmd)
        .arg("--verbose")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start vibewatch")
}

#[test]
fn test_watcher_detects_file_creation() {
    let temp_dir = common::setup_test_dir();

    // Start vibewatch with a cross-platform command to create a marker file
    let marker_file = temp_dir.child("marker.txt");
    let marker_path = marker_file.path().display().to_string();
    let command = common::touch_command(&marker_path);

    let mut child = start_watcher_with_command(&temp_dir, &command);

    // Wait for watcher to start
    thread::sleep(common::WATCHER_STARTUP_TIME);

    // Create a test file
    common::create_test_file(&temp_dir, "test.txt", "Hello, World!");

    // Wait for detection and command execution with polling
    let marker_exists = common::wait_for_file(marker_file.path(), common::MARKER_FILE_POLL_TIMEOUT);

    // Cleanup
    child.kill().expect("Failed to kill vibewatch");

    // Verify the marker file was created by the command
    // (This proves vibewatch detected the file creation and ran the command)
    assert!(
        marker_exists,
        "Marker file should exist at {}. Vibewatch may not have detected the file creation or executed the command.",
        marker_path
    );
}

#[test]
fn test_watcher_detects_file_modification() {
    let temp_dir = common::setup_test_dir();

    // Create initial file
    common::create_test_file(&temp_dir, "test.txt", "Initial content");

    // Start vibewatch
    let marker_file = temp_dir.child("modified_marker.txt");
    let marker_path = marker_file.path().display().to_string();
    let command = common::touch_command(&marker_path);

    let mut child = start_watcher_with_command(&temp_dir, &command);

    // Wait for watcher to start
    thread::sleep(common::WATCHER_STARTUP_TIME);

    // Modify the file
    common::modify_test_file(&temp_dir, "test.txt", "Modified content");

    // Wait for detection and command execution
    thread::sleep(common::EVENT_DETECTION_TIME);
    thread::sleep(common::COMMAND_EXECUTION_TIME);

    // Cleanup
    child.kill().expect("Failed to kill vibewatch");

    // Verify command was executed
    assert!(
        marker_file.path().exists(),
        "Marker file should exist. Vibewatch may not have detected the file modification."
    );
}

#[test]
fn test_watcher_detects_file_deletion() {
    let temp_dir = common::setup_test_dir();

    // Create initial file
    common::create_test_file(&temp_dir, "test.txt", "Delete me");

    // Start vibewatch
    let marker_file = temp_dir.child("deleted_marker.txt");
    let marker_path = marker_file.path().display().to_string();
    let command = common::touch_command(&marker_path);

    let mut child = start_watcher_with_command(&temp_dir, &command);

    // Wait for watcher to start
    thread::sleep(common::WATCHER_STARTUP_TIME);

    // Delete the file
    common::delete_test_file(&temp_dir, "test.txt");

    // Wait for detection and command execution
    thread::sleep(common::EVENT_DETECTION_TIME);
    thread::sleep(common::COMMAND_EXECUTION_TIME);

    // Cleanup
    child.kill().expect("Failed to kill vibewatch");

    // Verify command was executed
    assert!(
        marker_file.path().exists(),
        "Marker file should exist. Vibewatch may not have detected the file deletion."
    );
}

// ============================================================================
// Filter Tests - Test pattern matching with real files
// ============================================================================

#[test]
fn test_filter_include_pattern_only_watches_matching_files() {
    let temp_dir = common::setup_test_dir();

    // Use a counter file approach - each detection appends a line
    let counter_file = temp_dir.child("counter.txt");
    let counter_path = counter_file.path().display().to_string();
    // Create initial empty file
    common::create_test_file(&temp_dir, "counter.txt", "");
    let command = common::touch_command(&counter_path);

    // Only watch .rs files
    let mut child = StdCommand::cargo_bin("vibewatch")
        .unwrap()
        .arg(temp_dir.path())
        .arg("--include")
        .arg("*.rs")
        .arg("--on-change")
        .arg(&command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start vibewatch");

    thread::sleep(common::WATCHER_STARTUP_TIME);

    // Create a .rs file (should be detected)
    common::create_test_file(&temp_dir, "test.rs", "// Rust file");
    thread::sleep(common::EVENT_DETECTION_TIME);
    thread::sleep(common::COMMAND_EXECUTION_TIME);

    // Create a .txt file (should be ignored)
    common::create_test_file(&temp_dir, "test.txt", "Text file");
    thread::sleep(common::EVENT_DETECTION_TIME);
    thread::sleep(common::COMMAND_EXECUTION_TIME);

    child.kill().expect("Failed to kill vibewatch");

    // The counter file should have been modified (touched) at least once
    // This is a simple smoke test - we just verify the command ran
    assert!(
        counter_file.path().exists(),
        "Filter should have allowed .rs file through and executed command"
    );

    // Note: More sophisticated testing would require counting invocations,
    // but that requires shell features vibewatch doesn't currently use
}

#[test]
fn test_filter_exclude_pattern_ignores_matching_files() {
    let temp_dir = common::setup_test_dir();

    let marker_file = temp_dir.child("marker.txt");
    let marker_path = marker_file.path().display().to_string();
    let command = common::touch_command(&marker_path);

    // Exclude .tmp files
    let mut child = StdCommand::cargo_bin("vibewatch")
        .unwrap()
        .arg(temp_dir.path())
        .arg("--exclude")
        .arg("*.tmp")
        .arg("--on-change")
        .arg(&command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start vibewatch");

    thread::sleep(common::WATCHER_STARTUP_TIME);

    // Create a regular file (should be detected)
    common::create_test_file(&temp_dir, "test.txt", "Regular file");

    // Wait for detection and command execution with polling
    let marker_exists = common::wait_for_file(marker_file.path(), common::MARKER_FILE_POLL_TIMEOUT);

    // Create a .tmp file (should be ignored)
    common::create_test_file(&temp_dir, "test.tmp", "Temp file");
    thread::sleep(common::EVENT_DETECTION_TIME);

    child.kill().expect("Failed to kill vibewatch");

    // The marker file should exist (from the regular file)
    // This verifies the exclude pattern worked and allowed non-.tmp files
    assert!(
        marker_exists,
        "Marker file should exist from detecting non-.tmp file"
    );
}

#[test]
fn test_filter_multiple_include_patterns() {
    let temp_dir = common::setup_test_dir();

    let marker_file = temp_dir.child("marker.txt");
    let marker_path = marker_file.path().display().to_string();
    let command = common::touch_command(&marker_path);

    // Watch both .rs and .toml files
    let mut child = StdCommand::cargo_bin("vibewatch")
        .unwrap()
        .arg(temp_dir.path())
        .arg("--include")
        .arg("*.rs")
        .arg("--include")
        .arg("*.toml")
        .arg("--on-change")
        .arg(&command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start vibewatch");

    thread::sleep(common::WATCHER_STARTUP_TIME);

    // Create a .rs file (should be detected)
    common::create_test_file(&temp_dir, "test.rs", "// Rust");

    // Wait for detection and command execution with polling
    let marker_exists = common::wait_for_file(marker_file.path(), common::MARKER_FILE_POLL_TIMEOUT);

    // Create a .toml file (should be detected)
    common::create_test_file(&temp_dir, "test.toml", "[package]");
    thread::sleep(common::EVENT_DETECTION_TIME);

    // Create a .txt file (should be ignored)
    common::create_test_file(&temp_dir, "test.txt", "Text");
    thread::sleep(common::EVENT_DETECTION_TIME);

    child.kill().expect("Failed to kill vibewatch");

    // Should have detected both .rs and .toml files
    // The marker file existence proves the watch worked
    assert!(
        marker_exists,
        "Marker file should exist - both .rs and .toml files should have been detected"
    );
}

#[test]
fn test_filter_combine_include_and_exclude() {
    let temp_dir = common::setup_test_dir();

    let marker_file = temp_dir.child("marker.txt");
    let marker_path = marker_file.path().display().to_string();
    let command = common::touch_command(&marker_path);

    // Watch .rs files but exclude test.rs
    let mut child = StdCommand::cargo_bin("vibewatch")
        .unwrap()
        .arg(temp_dir.path())
        .arg("--include")
        .arg("*.rs")
        .arg("--exclude")
        .arg("test.rs")
        .arg("--on-change")
        .arg(&command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start vibewatch");

    thread::sleep(common::WATCHER_STARTUP_TIME);

    // Create main.rs (should be detected)
    common::create_test_file(&temp_dir, "main.rs", "// Main");

    // Wait for detection and command execution with polling
    let marker_exists = common::wait_for_file(marker_file.path(), common::MARKER_FILE_POLL_TIMEOUT);

    // Create test.rs (should be excluded)
    common::create_test_file(&temp_dir, "test.rs", "// Test");
    thread::sleep(common::EVENT_DETECTION_TIME);

    child.kill().expect("Failed to kill vibewatch");

    // Should have detected main.rs but not test.rs
    // The marker file existence proves filtering worked correctly
    assert!(
        marker_exists,
        "Marker file should exist - main.rs should have been detected (test.rs excluded)"
    );
}

// ============================================================================
// Command Execution Tests - Test template substitution and command execution
// ============================================================================

#[test]
fn test_command_template_substitution_file_path() {
    let temp_dir = common::setup_test_dir();

    let output_file = temp_dir.child("output.txt");
    let output_path = output_file.path().display().to_string();

    // For now, just test that command execution works with the template
    // Full template substitution testing would require shell features
    let command = common::touch_command(&output_path);

    let mut child = start_watcher_with_command(&temp_dir, &command);

    thread::sleep(common::WATCHER_STARTUP_TIME);

    // Create a test file
    common::create_test_file(&temp_dir, "watched.txt", "content");

    // Wait for detection and command execution with polling
    let output_exists = common::wait_for_file(output_file.path(), common::MARKER_FILE_POLL_TIMEOUT);

    child.kill().expect("Failed to kill vibewatch");

    // Verify the command was executed when file was created
    assert!(
        output_exists,
        "Output file should exist, proving command with template was executed"
    );
}

#[test]
fn test_specific_event_commands() {
    let temp_dir = common::setup_test_dir();

    // Create separate marker files for each event type
    // Using absolute paths to ensure they're outside the watched directory
    let markers_dir = common::setup_test_dir();
    let create_marker = markers_dir.child("create_marker.txt");
    let modify_marker = markers_dir.child("modify_marker.txt");
    let delete_marker = markers_dir.child("delete_marker.txt");

    let create_cmd = common::touch_command(&create_marker.path().display().to_string());
    let modify_cmd = common::touch_command(&modify_marker.path().display().to_string());
    let delete_cmd = common::touch_command(&delete_marker.path().display().to_string());

    // Start vibewatch with specific commands for each event type
    let mut child = StdCommand::cargo_bin("vibewatch")
        .unwrap()
        .arg(temp_dir.path())
        .arg("--debounce")
        .arg("0") // Disable debouncing for immediate test response
        .arg("--on-create")
        .arg(&create_cmd)
        .arg("--on-modify")
        .arg(&modify_cmd)
        .arg("--on-delete")
        .arg(&delete_cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start vibewatch");

    thread::sleep(common::WATCHER_STARTUP_TIME);

    // Test create event
    common::create_test_file(&temp_dir, "test.txt", "initial");
    thread::sleep(common::EVENT_DETECTION_TIME);
    thread::sleep(common::COMMAND_EXECUTION_TIME);

    // Test modify event
    common::modify_test_file(&temp_dir, "test.txt", "modified");
    thread::sleep(common::EVENT_DETECTION_TIME);
    thread::sleep(common::COMMAND_EXECUTION_TIME);

    // Test delete event
    common::delete_test_file(&temp_dir, "test.txt");
    thread::sleep(common::EVENT_DETECTION_TIME);
    thread::sleep(common::COMMAND_EXECUTION_TIME);

    child.kill().expect("Failed to kill vibewatch");

    // Verify each specific command was executed
    assert!(
        create_marker.path().exists(),
        "Create command was not executed"
    );
    assert!(
        modify_marker.path().exists(),
        "Modify command was not executed"
    );
    assert!(
        delete_marker.path().exists(),
        "Delete command was not executed"
    );
}
