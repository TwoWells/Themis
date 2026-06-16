// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
//! End-to-end integration tests exercising the real adapters against a
//! temporary config directory.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests assert via unwrap/expect"
)]
#![allow(
    clippy::needless_raw_string_hashes,
    clippy::uninlined_format_args,
    clippy::redundant_clone,
    reason = "test scaffolding: uniform raw YAML fixtures and explicit setup"
)]

use std::fs;
use tempfile::TempDir;
use themis::adapters::command::RealCommandExecutor;
use themis::adapters::dryrun::{DryRunCommandExecutor, DryRunFileSystem};
use themis::adapters::filesystem::RealFileSystem;
use themis::adapters::template::TeraAdapter;
use themis::commands::{init, verify};
use themis::core::orchestrator::Orchestrator;

mod common;

#[test]
fn test_end_to_end_flow() {
    // 1. Setup Temp Dir
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let config_dir = root.join("config");
    let templates_dir = config_dir.join("templates");
    let profiles_dir = config_dir.join("profiles");
    let output_file = root.join("output.conf");

    fs::create_dir_all(&templates_dir).unwrap();
    fs::create_dir_all(&profiles_dir).unwrap();

    // 2. Write Artifacts

    // themis.yaml
    let config_content = format!(
        r##"
        current_profile: test-dark
        enroll:
          test_app:
            type: template
            input: "{}/test.j2"
            output: "{}"
    "##,
        templates_dir.display(),
        output_file.display()
    );

    fs::write(config_dir.join("themis.yaml"), config_content).unwrap();

    // profiles/test-dark.yaml
    fs::write(
        profiles_dir.join("test-dark.yaml"),
        r##"
        metadata:
          name: test-dark
        vars:
          bg: "#123456"
    "##,
    )
    .unwrap();

    // templates/test.j2
    fs::write(templates_dir.join("test.j2"), "Background: {{ bg }}").unwrap();

    // 3. Initialize Orchestrator
    let fs_adapter = RealFileSystem;
    let tera_adapter = TeraAdapter::new();
    let cmd_adapter = RealCommandExecutor;

    let orchestrator = Orchestrator::new(fs_adapter, tera_adapter, cmd_adapter, config_dir.clone());

    // 4. Run Load
    let load_result = orchestrator.load_profile("test-dark").unwrap();
    assert!(
        load_result.is_ok(),
        "Apps failed: {:?}",
        load_result.failures
    );

    // 5. Verify Output
    let output_content = fs::read_to_string(&output_file).expect("Output file not found");
    assert_eq!(output_content, "Background: #123456");
}

#[test]
fn test_dry_run_does_not_write_files() {
    // 1. Setup Temp Dir
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let config_dir = root.join("config");
    let templates_dir = config_dir.join("templates");
    let profiles_dir = config_dir.join("profiles");
    let output_file = root.join("output.conf");

    fs::create_dir_all(&templates_dir).unwrap();
    fs::create_dir_all(&profiles_dir).unwrap();

    // 2. Write Artifacts
    let config_content = format!(
        r##"
        current_profile: test-dark
        enroll:
          test_app:
            type: template
            input: "{}/test.j2"
            output: "{}"
            reload_cmd: "echo reloaded"
    "##,
        templates_dir.display(),
        output_file.display()
    );

    fs::write(config_dir.join("themis.yaml"), config_content).unwrap();

    fs::write(
        profiles_dir.join("test-dark.yaml"),
        r##"
        metadata:
          name: test-dark
        vars:
          bg: "#123456"
    "##,
    )
    .unwrap();

    fs::write(templates_dir.join("test.j2"), "Background: {{ bg }}").unwrap();

    // 3. Initialize Orchestrator with DRY-RUN adapters
    let orchestrator = Orchestrator::new(
        DryRunFileSystem,
        TeraAdapter::new(),
        DryRunCommandExecutor,
        config_dir,
    );

    // 4. Run Load
    let load_result = orchestrator.load_profile("test-dark").unwrap();
    assert!(
        load_result.is_ok(),
        "Apps failed: {:?}",
        load_result.failures
    );

    // 5. Verify output file was NOT created
    assert!(
        !output_file.exists(),
        "Output file should not exist in dry-run mode"
    );
}

#[test]
fn test_init_creates_config_structure() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("themis");

    // Run init
    let result = init::run(&config_dir);
    assert!(result.is_ok(), "Init failed: {:?}", result.err());

    // Verify directories created
    assert!(config_dir.join("profiles").is_dir());
    assert!(config_dir.join("palettes").is_dir());
    assert!(config_dir.join("templates").is_dir());

    // Verify files created
    assert!(config_dir.join("themis.yaml").is_file());
    assert!(config_dir.join("profiles/example.yaml").is_file());

    // Verify config is valid YAML
    let config_content = fs::read_to_string(config_dir.join("themis.yaml")).unwrap();
    assert!(config_content.contains("enroll:"));

    // Verify profile is valid YAML
    let profile_content = fs::read_to_string(config_dir.join("profiles/example.yaml")).unwrap();
    assert!(profile_content.contains("vars:"));
}

#[test]
fn test_init_is_idempotent() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("themis");

    // Run init twice
    init::run(&config_dir).unwrap();
    let result = init::run(&config_dir);

    // Second run should succeed without error (idempotent)
    assert!(
        result.is_ok(),
        "Second init should not fail: {:?}",
        result.err()
    );

    // Files should still exist
    assert!(config_dir.join("themis.yaml").is_file());
}

#[test]
fn test_verify_valid_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let system_dir = temp_dir.path().join("system");

    // Create directories
    fs::create_dir_all(config_dir.join("profiles")).unwrap();
    fs::create_dir_all(config_dir.join("templates")).unwrap();
    fs::create_dir_all(system_dir.join("palettes")).unwrap();

    // Create valid config
    fs::write(
        config_dir.join("themis.yaml"),
        r#"
enroll:
  kitty:
    type: template
    input: "~/.config/themis/templates/kitty.j2"
    output: "~/.config/kitty/.themis.conf"
"#,
    )
    .unwrap();

    // Create template (so verify passes)
    let home = std::env::var("HOME").unwrap();
    let template_dir = format!("{}/.config/themis/templates", home);
    fs::create_dir_all(&template_dir).ok();
    fs::write(format!("{}/kitty.j2", template_dir), "test").ok();

    // Create valid profile
    fs::write(
        config_dir.join("profiles/test.yaml"),
        r##"
vars:
  bg: "#000000"
"##,
    )
    .unwrap();

    let result = verify::run(&config_dir, &system_dir).unwrap();
    // May have warnings about template not existing, but should not error on YAML validity
    assert!(
        result.errors.iter().all(|e| !e.contains("invalid YAML")),
        "Should not have YAML errors: {:?}",
        result.errors
    );
}

#[test]
fn test_verify_detects_invalid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let system_dir = temp_dir.path().join("system");

    fs::create_dir_all(config_dir.join("profiles")).unwrap();
    fs::create_dir_all(&system_dir).unwrap();

    // Create valid config
    fs::write(config_dir.join("themis.yaml"), "enroll: {}").unwrap();

    // Create invalid profile YAML
    fs::write(
        config_dir.join("profiles/bad.yaml"),
        "this is not: valid: yaml: [",
    )
    .unwrap();

    let result = verify::run(&config_dir, &system_dir).unwrap();
    assert!(
        result.errors.iter().any(|e| e.contains("invalid YAML")),
        "Should detect invalid YAML: {:?}",
        result.errors
    );
}

#[test]
fn test_verify_detects_missing_palette() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let system_dir = temp_dir.path().join("system");

    fs::create_dir_all(config_dir.join("profiles")).unwrap();
    fs::create_dir_all(&system_dir).unwrap();

    // Create valid config
    fs::write(config_dir.join("themis.yaml"), "enroll: {}").unwrap();

    // Create profile that includes non-existent palette
    fs::write(
        config_dir.join("profiles/test.yaml"),
        r#"
include: nonexistent
vars: {}
"#,
    )
    .unwrap();

    let result = verify::run(&config_dir, &system_dir).unwrap();
    assert!(
        result.errors.iter().any(|e| e.contains("doesn't exist")),
        "Should detect missing palette: {:?}",
        result.errors
    );
}

#[test]
fn test_doctor_detects_missing_include() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");

    fs::create_dir_all(&config_dir).unwrap();

    // Create config enrolling kitty
    fs::write(
        config_dir.join("themis.yaml"),
        r#"
enroll:
  kitty:
    type: template
    input: "~/.config/themis/templates/kitty.j2"
    output: "~/.config/kitty/.themis.conf"
"#,
    )
    .unwrap();

    // Create kitty config WITHOUT the include pattern
    let dot_config_kitty = temp_dir.path().join(".config/kitty");
    fs::create_dir_all(&dot_config_kitty).unwrap();
    fs::write(
        dot_config_kitty.join("kitty.conf"),
        "# Kitty config\nfont_size 12\n",
    )
    .unwrap();

    // Run doctor in an isolated subprocess
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_themis"));
    cmd.args(["--config", config_dir.to_str().unwrap(), "doctor"]);
    common::isolate_env(&mut cmd, temp_dir.path());
    let output = cmd.output().unwrap();

    // Should exit with failure (missing include pattern)
    assert!(!output.status.success(), "Should fail when include missing");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    assert!(
        combined.contains("kitty"),
        "Should mention kitty in output: {}",
        combined
    );
}

#[test]
fn test_doctor_reports_ok_with_include() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");

    fs::create_dir_all(&config_dir).unwrap();

    // Create config enrolling kitty
    fs::write(
        config_dir.join("themis.yaml"),
        r#"
enroll:
  kitty:
    type: template
    input: "~/.config/themis/templates/kitty.j2"
    output: "~/.config/kitty/.themis.conf"
"#,
    )
    .unwrap();

    // Create kitty config WITH the include pattern
    let dot_config_kitty = temp_dir.path().join(".config/kitty");
    fs::create_dir_all(&dot_config_kitty).unwrap();
    fs::write(
        dot_config_kitty.join("kitty.conf"),
        "# Kitty config\ninclude .themis.conf\nfont_size 12\n",
    )
    .unwrap();

    // Run doctor in an isolated subprocess
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_themis"));
    cmd.args(["--config", config_dir.to_str().unwrap(), "doctor"]);
    common::isolate_env(&mut cmd, temp_dir.path());
    let output = cmd.output().unwrap();

    assert!(
        output.status.success(),
        "Should succeed when include present: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_doctor_skips_unknown_apps() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");

    fs::create_dir_all(&config_dir).unwrap();

    // Create config enrolling gtk (which has no check defined)
    fs::write(
        config_dir.join("themis.yaml"),
        r#"
enroll:
  gtk:
    type: command
    commands:
      - "gsettings set org.gnome.desktop.interface color-scheme prefer-dark"
"#,
    )
    .unwrap();

    // Run doctor in an isolated subprocess
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_themis"));
    cmd.args(["--config", config_dir.to_str().unwrap(), "doctor"]);
    common::isolate_env(&mut cmd, temp_dir.path());
    let output = cmd.output().unwrap();

    // Should succeed (unknown apps are skipped, not failures)
    assert!(
        output.status.success(),
        "Should succeed when only unknown apps: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_status_shows_no_state_initially() {
    let temp_dir = TempDir::new().unwrap();

    // Run status in an isolated subprocess (no state file exists)
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_themis"));
    cmd.arg("status");
    common::isolate_env(&mut cmd, temp_dir.path());
    let output = cmd.output().unwrap();

    assert!(output.status.success(), "Status should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No state found"),
        "Should indicate no state: {}",
        stdout
    );
}

#[test]
fn test_status_shows_loaded_profile() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/themis");

    // Create minimal config and profile
    fs::create_dir_all(config_dir.join("profiles")).unwrap();
    fs::create_dir_all(config_dir.join("templates")).unwrap();

    fs::write(config_dir.join("themis.yaml"), "enroll: {}").unwrap();
    fs::write(
        config_dir.join("profiles/test-profile.yaml"),
        "vars:\n  bg: \"#000000\"",
    )
    .unwrap();

    // Run load first
    let mut load_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_themis"));
    load_cmd.args([
        "--config",
        config_dir.to_str().unwrap(),
        "load",
        "test-profile",
    ]);
    common::isolate_env(&mut load_cmd, temp_dir.path());
    let load_output = load_cmd.output().unwrap();

    assert!(
        load_output.status.success(),
        "Load should succeed: {}",
        String::from_utf8_lossy(&load_output.stderr)
    );

    // Now run status
    let mut status_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_themis"));
    status_cmd.arg("status");
    common::isolate_env(&mut status_cmd, temp_dir.path());
    let status_output = status_cmd.output().unwrap();

    assert!(status_output.status.success(), "Status should succeed");
    let stdout = String::from_utf8_lossy(&status_output.stdout);
    assert!(
        stdout.contains("test-profile"),
        "Should show loaded profile name: {}",
        stdout
    );
}

#[test]
fn test_dry_run_does_not_save_state() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/themis");

    // Create minimal config and profile
    fs::create_dir_all(config_dir.join("profiles")).unwrap();

    fs::write(config_dir.join("themis.yaml"), "enroll: {}").unwrap();
    fs::write(
        config_dir.join("profiles/test-profile.yaml"),
        "vars:\n  bg: \"#000000\"",
    )
    .unwrap();

    // Run load with --dry-run
    let mut load_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_themis"));
    load_cmd.args([
        "--config",
        config_dir.to_str().unwrap(),
        "load",
        "test-profile",
        "--dry-run",
    ]);
    common::isolate_env(&mut load_cmd, temp_dir.path());
    let load_output = load_cmd.output().unwrap();

    assert!(
        load_output.status.success(),
        "Dry-run load should succeed: {}",
        String::from_utf8_lossy(&load_output.stderr)
    );

    // Now run status - should show no state
    let mut status_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_themis"));
    status_cmd.arg("status");
    common::isolate_env(&mut status_cmd, temp_dir.path());
    let status_output = status_cmd.output().unwrap();

    assert!(status_output.status.success(), "Status should succeed");
    let stdout = String::from_utf8_lossy(&status_output.stdout);
    assert!(
        stdout.contains("No state found"),
        "Dry-run should not save state: {}",
        stdout
    );
}

#[test]
fn test_status_respects_xdg_state_home() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".config/themis");
    let custom_state_dir = temp_dir.path().join("custom-state");

    // Create minimal config and profile
    fs::create_dir_all(config_dir.join("profiles")).unwrap();
    fs::create_dir_all(&custom_state_dir).unwrap();

    fs::write(config_dir.join("themis.yaml"), "enroll: {}").unwrap();
    fs::write(
        config_dir.join("profiles/xdg-test.yaml"),
        "vars:\n  bg: \"#000000\"",
    )
    .unwrap();

    // Run load with a custom XDG_STATE_HOME. Isolate first, then override
    // XDG_STATE_HOME so this test still verifies the custom-dir behavior.
    let mut load_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_themis"));
    load_cmd.args(["--config", config_dir.to_str().unwrap(), "load", "xdg-test"]);
    common::isolate_env(&mut load_cmd, temp_dir.path());
    load_cmd.env("XDG_STATE_HOME", &custom_state_dir);
    let load_output = load_cmd.output().unwrap();

    assert!(
        load_output.status.success(),
        "Load should succeed: {}",
        String::from_utf8_lossy(&load_output.stderr)
    );

    // Verify state was saved to custom location
    let state_file = custom_state_dir.join("themis/state.json");
    assert!(
        state_file.exists(),
        "State should be saved to XDG_STATE_HOME"
    );

    // Verify status reads from custom location
    let mut status_cmd = std::process::Command::new(env!("CARGO_BIN_EXE_themis"));
    status_cmd.arg("status");
    common::isolate_env(&mut status_cmd, temp_dir.path());
    status_cmd.env("XDG_STATE_HOME", &custom_state_dir);
    let status_output = status_cmd.output().unwrap();

    let stdout = String::from_utf8_lossy(&status_output.stdout);
    assert!(
        stdout.contains("xdg-test"),
        "Status should read from XDG_STATE_HOME: {}",
        stdout
    );
}

#[test]
fn test_completions_bash() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_themis"))
        .args(["completions", "bash"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Completions command should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("_themis()"),
        "Bash completions should contain function: {}",
        stdout
    );
}

#[test]
fn test_completions_zsh() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_themis"))
        .args(["completions", "zsh"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Completions command should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("#compdef themis"),
        "Zsh completions should contain compdef: {}",
        stdout
    );
}

#[test]
fn test_completions_fish() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_themis"))
        .args(["completions", "fish"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Completions command should succeed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("complete -c themis"),
        "Fish completions should contain complete command: {}",
        stdout
    );
}
