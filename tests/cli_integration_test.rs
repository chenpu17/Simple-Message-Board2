//! CLI integration tests for the message board
//!
//! These tests verify the command-line interface works correctly.

use std::process::Command;

/// Helper to get the binary path
fn get_binary_path() -> std::path::PathBuf {
    // Use the debug binary built by cargo test
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test executable name
    path.pop(); // Remove 'deps' directory
    path.push("message_board");
    path
}

/// Test that --help works
#[test]
fn test_cli_help() {
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("start"));
    assert!(stdout.contains("stop"));
    assert!(stdout.contains("restart"));
    assert!(stdout.contains("status"));
    assert!(stdout.contains("logs"));
    assert!(stdout.contains("--port"));
    assert!(stdout.contains("--data-dir"));
    assert!(stdout.contains("--foreground"));
}

/// Test that --version works
#[test]
fn test_cli_version() {
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("message-board"));
}

/// Test start --help
#[test]
fn test_start_help() {
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .args(["start", "--help"])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--port"));
    assert!(stdout.contains("--data-dir"));
    assert!(stdout.contains("--foreground"));
}

/// Test stop --help
#[test]
fn test_stop_help() {
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .args(["stop", "--help"])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--data-dir"));
}

/// Test restart --help
#[test]
fn test_restart_help() {
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .args(["restart", "--help"])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--port"));
    assert!(stdout.contains("--data-dir"));
    assert!(stdout.contains("--foreground"));
}

/// Test status --help
#[test]
fn test_status_help() {
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .args(["status", "--help"])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--data-dir"));
}

/// Test logs --help
#[test]
fn test_logs_help() {
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .args(["logs", "--help"])
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--data-dir"));
    assert!(stdout.contains("--lines"));
}

/// Test status command when service is not running
#[test]
fn test_status_not_running() {
    let binary = get_binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let data_dir = temp_dir.path().to_str().unwrap();

    let output = Command::new(&binary)
        .args(["status", "-d", data_dir])
        .output()
        .expect("Failed to execute binary");

    // Command should succeed even when not running
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("not running") || stdout.contains("Message board is not running"));
}

/// Test logs command when no log file exists
#[test]
fn test_logs_no_file() {
    let binary = get_binary_path();
    let temp_dir = tempfile::tempdir().unwrap();
    let data_dir = temp_dir.path().to_str().unwrap();

    let output = Command::new(&binary)
        .args(["logs", "-d", data_dir])
        .output()
        .expect("Failed to execute binary");

    // Command should succeed
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No log file") || stdout.is_empty());
}

/// Test that invalid command shows error
#[test]
fn test_invalid_command() {
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .arg("invalid_command")
        .output()
        .expect("Failed to execute binary");

    // Should fail
    assert!(!output.status.success());
}

/// Test that invalid option shows error
#[test]
fn test_invalid_option() {
    let binary = get_binary_path();
    let output = Command::new(&binary)
        .args(["start", "--invalid-option"])
        .output()
        .expect("Failed to execute binary");

    // Should fail
    assert!(!output.status.success());
}
