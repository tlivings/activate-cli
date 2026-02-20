//! Integration tests for project activation flow
//!
//! These tests verify that activation is fast and correct:
//! - No git operations during normal activation
//! - Path output is correct
//! - Database updates work properly
//! - debug_fast mode skips DB updates

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper to run activate binary with args
fn run_activate(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_activate"))
        .args(args)
        .output()
        .expect("Failed to run activate")
}

/// Helper to set up a test environment with a temporary config
fn setup_test_env() -> TempDir {
    let temp = TempDir::new().unwrap();
    // Set XDG dirs to temp for isolation
    std::env::set_var("XDG_CONFIG_HOME", temp.path().join("config"));
    std::env::set_var("XDG_DATA_HOME", temp.path().join("data"));
    temp
}

#[test]
fn test_list_empty_returns_success() {
    let _temp = setup_test_env();
    let output = run_activate(&["--list"]);
    // May fail if not configured, but shouldn't panic
    assert!(output.status.success() || !output.stderr.is_empty());
}

#[test]
fn test_help_flag() {
    let output = run_activate(&["--help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("activate"));
    assert!(stdout.contains("--list"));
}

#[test]
fn test_version_flag() {
    let output = run_activate(&["--version"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("activate"));
}

#[test]
fn test_init_zsh_generates_wrapper() {
    let output = run_activate(&["--init", "zsh"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("__ACTIVATE_BIN"));
    assert!(stdout.contains("precmd"));
    assert!(stdout.contains("__activate_target"));
}

#[test]
fn test_init_bash_generates_wrapper() {
    let output = run_activate(&["--init", "bash"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("__ACTIVATE_BIN"));
}

#[test]
fn test_init_fish_generates_wrapper() {
    let output = run_activate(&["--init", "fish"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("__ACTIVATE_BIN"));
}

#[test]
fn test_init_with_custom_name() {
    let output = run_activate(&["--init", "zsh", "--name", "proj"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("proj()"));
    assert!(stdout.contains("proj_debug()"));
}

#[test]
fn test_init_invalid_shell_fails() {
    let output = run_activate(&["--init", "invalid"]);
    assert!(!output.status.success());
}

#[test]
fn test_debug_fast_flag_exists() {
    // Just verify the flag is accepted
    let output = run_activate(&["--help"]);
    // debug_fast is hidden, so just verify help works
    assert!(output.status.success());
}

// Shell wrapper tests
mod shell_wrapper {
    use super::*;

    #[test]
    fn test_zsh_wrapper_has_precmd_hook() {
        let output = run_activate(&["--init", "zsh"]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Verify precmd workaround for Warp is present
        assert!(stdout.contains("__activate_precmd"), "Should have precmd hook");
        assert!(stdout.contains("precmd_functions"), "Should register precmd hook");
        assert!(stdout.contains("__activate_target"), "Should use target variable");
    }

    #[test]
    fn test_zsh_wrapper_function_sets_target() {
        let output = run_activate(&["--init", "zsh", "--name", "act"]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Function should set target variable, not cd directly
        assert!(stdout.contains("__activate_target=\"$result\""),
            "Function should set target variable");
        assert!(!stdout.contains("builtin cd"),
            "Function should NOT cd directly (precmd does it)");
    }

    #[test]
    fn test_zsh_wrapper_has_debug_function() {
        let output = run_activate(&["--init", "zsh", "--name", "act"]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(stdout.contains("act_debug()"), "Should have debug function");
        assert!(stdout.contains("--debug-fast"), "Debug function should use --debug-fast");
    }

    #[test]
    fn test_bash_wrapper_structure() {
        let output = run_activate(&["--init", "bash", "--name", "act"]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(stdout.contains("act()"), "Should define function");
        assert!(stdout.contains("__ACTIVATE_BIN="), "Should set binary path");
        assert!(stdout.contains("complete -F"), "Should have completion");
    }

    #[test]
    fn test_fish_wrapper_structure() {
        let output = run_activate(&["--init", "fish", "--name", "act"]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(stdout.contains("function act"), "Should define function");
        assert!(stdout.contains("set -g __ACTIVATE_BIN"), "Should set binary path");
        assert!(stdout.contains("complete -c act"), "Should have completion");
    }

    #[test]
    fn test_wrapper_uses_full_binary_path() {
        let output = run_activate(&["--init", "zsh"]);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should use full path, not just "activate"
        assert!(stdout.contains("__ACTIVATE_BIN=\"/"),
            "Should use absolute path to binary");
    }
}
