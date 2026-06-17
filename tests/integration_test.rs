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
fn test_symlink_integration_with_real_adapters() {
    // Exercises RealFileSystem::create_symlink end-to-end: the source path is
    // rendered as a template ({{ mode }}), `~`-expanded, then linked at target.
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let config_dir = root.join("config");
    let profiles_dir = config_dir.join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    // A real source file that the symlink should point at, named by `mode`.
    let source_file = root.join("dark.conf");
    fs::write(&source_file, "real source contents").unwrap();

    // Target lives under a directory that does NOT exist yet, so the link
    // creation must also create the parent directory.
    let target_link = root.join("nested/link.conf");

    let config_content = format!(
        r##"
        enroll:
          linked_app:
            type: symlink
            source: "{}/{{{{ mode }}}}.conf"
            target: "{}"
    "##,
        root.display(),
        target_link.display()
    );
    fs::write(config_dir.join("themis.yaml"), config_content).unwrap();

    fs::write(
        profiles_dir.join("test.yaml"),
        r##"
        vars:
          mode: dark
    "##,
    )
    .unwrap();

    let orchestrator = Orchestrator::new(
        RealFileSystem,
        TeraAdapter::new(),
        RealCommandExecutor,
        config_dir,
    );

    let load_result = orchestrator.load_profile("test").unwrap();
    assert!(
        load_result.is_ok(),
        "Apps failed: {:?}",
        load_result.failures
    );

    // The link must exist and resolve to the source file's contents.
    assert!(target_link.exists(), "Symlink target should exist");
    let linked_contents = fs::read_to_string(&target_link).unwrap();
    assert_eq!(
        linked_contents, "real source contents",
        "Symlink should resolve to the rendered source file"
    );
}

#[test]
fn test_symlink_integration_overwrites_existing_target() {
    // RealFileSystem::create_symlink force-overwrites an existing target. This
    // pins the `target.exists() || target.is_symlink()` guard: the target here
    // is a pre-existing *regular* file, so only the `exists()` half is true.
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let config_dir = root.join("config");
    let profiles_dir = config_dir.join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    let source_file = root.join("source.conf");
    fs::write(&source_file, "linked contents").unwrap();

    // Pre-existing regular file at the target path; the link must replace it.
    let target_link = root.join("target.conf");
    fs::write(&target_link, "stale contents that must be replaced").unwrap();

    let config_content = format!(
        r##"
        enroll:
          linked_app:
            type: symlink
            source: "{}"
            target: "{}"
    "##,
        source_file.display(),
        target_link.display()
    );
    fs::write(config_dir.join("themis.yaml"), config_content).unwrap();

    fs::write(profiles_dir.join("test.yaml"), "vars: {}\n").unwrap();

    let orchestrator = Orchestrator::new(
        RealFileSystem,
        TeraAdapter::new(),
        RealCommandExecutor,
        config_dir,
    );

    let load_result = orchestrator.load_profile("test").unwrap();
    assert!(
        load_result.is_ok(),
        "Linking over an existing file should succeed: {:?}",
        load_result.failures
    );

    // The target must now resolve to the source's contents, not the stale data.
    let linked_contents = fs::read_to_string(&target_link).unwrap();
    assert_eq!(
        linked_contents, "linked contents",
        "Existing target should have been replaced by the new symlink"
    );
}

#[test]
fn test_command_integration_with_real_adapters() {
    // Exercises RealCommandExecutor::run_command end-to-end: a rendered shell
    // command runs and its side effect (a written file) is observable.
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let config_dir = root.join("config");
    let profiles_dir = config_dir.join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    let marker = root.join("marker.txt");

    let config_content = format!(
        r##"
        enroll:
          cmd_app:
            type: command
            commands:
              - "printf '%s' {{{{ mode }}}} > {}"
    "##,
        marker.display()
    );
    fs::write(config_dir.join("themis.yaml"), config_content).unwrap();

    fs::write(
        profiles_dir.join("test.yaml"),
        r##"
        vars:
          mode: dark
    "##,
    )
    .unwrap();

    let orchestrator = Orchestrator::new(
        RealFileSystem,
        TeraAdapter::new(),
        RealCommandExecutor,
        config_dir,
    );

    let load_result = orchestrator.load_profile("test").unwrap();
    assert!(
        load_result.is_ok(),
        "Apps failed: {:?}",
        load_result.failures
    );

    let written = fs::read_to_string(&marker).unwrap();
    assert_eq!(
        written, "dark",
        "The rendered shell command should have run and written the marker"
    );
}

#[test]
fn test_command_integration_reports_failure_with_real_adapters() {
    // A command that exits non-zero must surface as an app failure, pinning
    // RealCommandExecutor's `!output.status.success()` error path.
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let config_dir = root.join("config");
    let profiles_dir = config_dir.join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    fs::write(
        config_dir.join("themis.yaml"),
        r##"
        enroll:
          failing_app:
            type: command
            commands:
              - "exit 3"
    "##,
    )
    .unwrap();

    fs::write(
        profiles_dir.join("test.yaml"),
        r##"
        vars: {}
    "##,
    )
    .unwrap();

    let orchestrator = Orchestrator::new(
        RealFileSystem,
        TeraAdapter::new(),
        RealCommandExecutor,
        config_dir,
    );

    let load_result = orchestrator.load_profile("test").unwrap();
    assert!(
        !load_result.is_ok(),
        "A non-zero command should produce a failure"
    );
    assert_eq!(load_result.failure_count(), 1);
    assert_eq!(load_result.failures[0].app_name, "failing_app");
}

#[test]
fn test_script_integration_with_real_adapters() {
    // Exercises RealCommandExecutor::run_script end-to-end with a real
    // executable, asserting both the success path and that a THEMIS_* env var
    // reaches the script.
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let config_dir = root.join("config");
    let profiles_dir = config_dir.join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    let marker = root.join("script-marker.txt");
    let script_path = root.join("theme.sh");
    fs::write(
        &script_path,
        format!(
            "#!/bin/sh\nprintf '%s' \"$THEMIS_MODE\" > {}\n",
            marker.display()
        ),
    )
    .unwrap();
    let mut perms = fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).unwrap();

    let config_content = format!(
        r##"
        enroll:
          scripted_app:
            type: script
            path: "{}"
    "##,
        script_path.display()
    );
    fs::write(config_dir.join("themis.yaml"), config_content).unwrap();

    fs::write(
        profiles_dir.join("test.yaml"),
        r##"
        vars:
          mode: dark
    "##,
    )
    .unwrap();

    let orchestrator = Orchestrator::new(
        RealFileSystem,
        TeraAdapter::new(),
        RealCommandExecutor,
        config_dir,
    );

    let load_result = orchestrator.load_profile("test").unwrap();
    assert!(
        load_result.is_ok(),
        "Apps failed: {:?}",
        load_result.failures
    );

    let written = fs::read_to_string(&marker).unwrap();
    assert_eq!(
        written, "dark",
        "The script should have run with THEMIS_MODE in its environment"
    );
}

#[test]
fn test_script_integration_reports_failure_with_real_adapters() {
    // A script that exits non-zero must surface as an app failure, pinning
    // RealCommandExecutor::run_script's `!output.status.success()` error path.
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let config_dir = root.join("config");
    let profiles_dir = config_dir.join("profiles");
    fs::create_dir_all(&profiles_dir).unwrap();

    let script_path = root.join("failing.sh");
    fs::write(&script_path, "#!/bin/sh\nexit 4\n").unwrap();
    let mut perms = fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script_path, perms).unwrap();

    let config_content = format!(
        r##"
        enroll:
          scripted_app:
            type: script
            path: "{}"
    "##,
        script_path.display()
    );
    fs::write(config_dir.join("themis.yaml"), config_content).unwrap();

    fs::write(
        profiles_dir.join("test.yaml"),
        r##"
        vars: {}
    "##,
    )
    .unwrap();

    let orchestrator = Orchestrator::new(
        RealFileSystem,
        TeraAdapter::new(),
        RealCommandExecutor,
        config_dir,
    );

    let load_result = orchestrator.load_profile("test").unwrap();
    assert!(
        !load_result.is_ok(),
        "A non-zero script should produce a failure"
    );
    assert_eq!(load_result.failure_count(), 1);
    assert_eq!(load_result.failures[0].app_name, "scripted_app");
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
fn test_verify_detects_missing_template() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let system_dir = temp_dir.path().join("system");

    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&system_dir).unwrap();

    // Enroll an app whose template file does not exist.
    fs::write(
        config_dir.join("themis.yaml"),
        format!(
            r#"
enroll:
  kitty:
    type: template
    input: "{}/does-not-exist.j2"
    output: "{}/out/kitty.conf"
"#,
            config_dir.display(),
            config_dir.display()
        ),
    )
    .unwrap();

    let result = verify::run(&config_dir, &system_dir).unwrap();
    assert!(
        !result.is_ok(),
        "Missing template should make verify fail: {:?}",
        result.errors
    );
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("Template not found") && e.contains("kitty")),
        "Expected a template-not-found error: {:?}",
        result.errors
    );
}

#[test]
fn test_verify_detects_missing_script() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let system_dir = temp_dir.path().join("system");

    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&system_dir).unwrap();

    fs::write(
        config_dir.join("themis.yaml"),
        format!(
            r#"
enroll:
  myapp:
    type: script
    path: "{}/no-such-script.sh"
"#,
            config_dir.display()
        ),
    )
    .unwrap();

    let result = verify::run(&config_dir, &system_dir).unwrap();
    assert!(
        !result.is_ok(),
        "Missing script should make verify fail: {:?}",
        result.errors
    );
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("Script not found") && e.contains("myapp")),
        "Expected a script-not-found error: {:?}",
        result.errors
    );
}

#[test]
fn test_verify_warns_on_missing_output_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let system_dir = temp_dir.path().join("system");

    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&system_dir).unwrap();

    // Template exists, but the output parent directory does not.
    let template_path = config_dir.join("kitty.j2");
    fs::write(&template_path, "test").unwrap();

    fs::write(
        config_dir.join("themis.yaml"),
        format!(
            r#"
enroll:
  kitty:
    type: template
    input: "{}"
    output: "{}/nonexistent-dir/kitty.conf"
"#,
            template_path.display(),
            config_dir.display()
        ),
    )
    .unwrap();

    let result = verify::run(&config_dir, &system_dir).unwrap();
    // No errors: the template exists and the YAML is valid.
    assert!(
        result.is_ok(),
        "Verify should pass (only a warning expected): {:?}",
        result.errors
    );
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.contains("Output directory doesn't exist")),
        "Expected a missing-output-directory warning: {:?}",
        result.warnings
    );
}

#[test]
fn test_verify_resolves_palette_from_user_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let system_dir = temp_dir.path().join("system");

    fs::create_dir_all(config_dir.join("profiles")).unwrap();
    fs::create_dir_all(config_dir.join("palettes")).unwrap();
    fs::create_dir_all(system_dir.join("palettes")).unwrap();

    fs::write(config_dir.join("themis.yaml"), "enroll: {}").unwrap();

    // Profile includes a palette that exists ONLY in the user palettes dir.
    fs::write(
        config_dir.join("profiles/test.yaml"),
        r#"
include: user-only
vars: {}
"#,
    )
    .unwrap();
    fs::write(
        config_dir.join("palettes/user-only.yaml"),
        "vars:\n  bg: \"#000000\"",
    )
    .unwrap();

    let result = verify::run(&config_dir, &system_dir).unwrap();
    assert!(
        result.is_ok(),
        "A palette present only in the user dir should resolve: {:?}",
        result.errors
    );
    assert!(
        !result.errors.iter().any(|e| e.contains("doesn't exist")),
        "Should not report the user palette as missing: {:?}",
        result.errors
    );
}

#[test]
fn test_verify_resolves_palette_from_system_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let system_dir = temp_dir.path().join("system");

    fs::create_dir_all(config_dir.join("profiles")).unwrap();
    fs::create_dir_all(system_dir.join("palettes")).unwrap();

    fs::write(config_dir.join("themis.yaml"), "enroll: {}").unwrap();

    // Profile includes a palette that exists ONLY in the system palettes dir.
    fs::write(
        config_dir.join("profiles/test.yaml"),
        r#"
include: system-only
vars: {}
"#,
    )
    .unwrap();
    fs::write(
        system_dir.join("palettes/system-only.yaml"),
        "vars:\n  bg: \"#000000\"",
    )
    .unwrap();

    let result = verify::run(&config_dir, &system_dir).unwrap();
    assert!(
        result.is_ok(),
        "A palette present only in the system dir should resolve: {:?}",
        result.errors
    );
    assert!(
        !result.errors.iter().any(|e| e.contains("doesn't exist")),
        "Should not report the system palette as missing: {:?}",
        result.errors
    );
}

#[test]
fn test_verify_warns_on_missing_symlink_target_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let system_dir = temp_dir.path().join("system");

    fs::create_dir_all(&config_dir).unwrap();
    fs::create_dir_all(&system_dir).unwrap();

    // A symlink integration whose target lives in a directory that does not
    // exist yet must produce a warning (not an error).
    fs::write(
        config_dir.join("themis.yaml"),
        format!(
            r#"
enroll:
  linked:
    type: symlink
    source: "{}/src.conf"
    target: "{}/missing-dir/link.conf"
"#,
            config_dir.display(),
            config_dir.display()
        ),
    )
    .unwrap();

    let result = verify::run(&config_dir, &system_dir).unwrap();
    assert!(
        result.is_ok(),
        "Symlink with a missing target dir is a warning, not an error: {:?}",
        result.errors
    );
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.contains("Symlink target directory doesn't exist")),
        "Expected a missing-symlink-target-dir warning: {:?}",
        result.warnings
    );
}

#[test]
fn test_verify_ignores_non_yaml_files_in_profiles() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let system_dir = temp_dir.path().join("system");

    fs::create_dir_all(config_dir.join("profiles")).unwrap();
    fs::create_dir_all(&system_dir).unwrap();

    fs::write(config_dir.join("themis.yaml"), "enroll: {}").unwrap();

    // A non-YAML file in the profiles dir must be ignored entirely — even
    // though its contents are not valid YAML, verify must not try to parse it.
    fs::write(
        config_dir.join("profiles/README.txt"),
        "this is not: valid: yaml: [ and must be ignored",
    )
    .unwrap();

    let result = verify::run(&config_dir, &system_dir).unwrap();
    assert!(
        result.is_ok(),
        "A .txt file in profiles/ must be skipped, not parsed: {:?}",
        result.errors
    );
    assert!(
        result.errors.is_empty(),
        "No errors expected from a non-YAML profile file: {:?}",
        result.errors
    );
}

#[test]
fn test_verify_processes_yml_extension_profiles() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let system_dir = temp_dir.path().join("system");

    fs::create_dir_all(config_dir.join("profiles")).unwrap();
    fs::create_dir_all(&system_dir).unwrap();

    fs::write(config_dir.join("themis.yaml"), "enroll: {}").unwrap();

    // A profile using the `.yml` (not `.yaml`) extension must still be
    // validated: this one includes a missing palette, which must be reported.
    fs::write(
        config_dir.join("profiles/legacy.yml"),
        r#"
include: nonexistent-palette
vars: {}
"#,
    )
    .unwrap();

    let result = verify::run(&config_dir, &system_dir).unwrap();
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("legacy") && e.contains("doesn't exist")),
        "A .yml profile should be validated like .yaml: {:?}",
        result.errors
    );
}

#[test]
fn test_verify_detects_palette_with_missing_include() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let system_dir = temp_dir.path().join("system");

    fs::create_dir_all(config_dir.join("palettes")).unwrap();
    fs::create_dir_all(&system_dir).unwrap();

    fs::write(config_dir.join("themis.yaml"), "enroll: {}").unwrap();

    // A palette that includes a parent palette which does not exist must be
    // reported as an error.
    fs::write(
        config_dir.join("palettes/child.yaml"),
        r#"
include: missing-parent
vars: {}
"#,
    )
    .unwrap();

    let result = verify::run(&config_dir, &system_dir).unwrap();
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("child") && e.contains("doesn't exist")),
        "A palette including a missing parent should error: {:?}",
        result.errors
    );
}

#[test]
fn test_verify_processes_yml_extension_palettes() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let system_dir = temp_dir.path().join("system");

    fs::create_dir_all(config_dir.join("palettes")).unwrap();
    fs::create_dir_all(&system_dir).unwrap();

    fs::write(config_dir.join("themis.yaml"), "enroll: {}").unwrap();

    // A palette using the `.yml` (not `.yaml`) extension must still be
    // validated: this one includes a missing parent, which must be reported.
    fs::write(
        config_dir.join("palettes/legacy.yml"),
        r#"
include: missing-parent
vars: {}
"#,
    )
    .unwrap();

    let result = verify::run(&config_dir, &system_dir).unwrap();
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("legacy") && e.contains("doesn't exist")),
        "A .yml palette should be validated like .yaml: {:?}",
        result.errors
    );
}

#[test]
fn test_verify_detects_invalid_palette_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    let system_dir = temp_dir.path().join("system");

    fs::create_dir_all(config_dir.join("palettes")).unwrap();
    fs::create_dir_all(&system_dir).unwrap();

    fs::write(config_dir.join("themis.yaml"), "enroll: {}").unwrap();

    // Invalid palette YAML (a .yaml file, so it passes the extension filter).
    fs::write(
        config_dir.join("palettes/bad.yaml"),
        "this is not: valid: yaml: [",
    )
    .unwrap();

    let result = verify::run(&config_dir, &system_dir).unwrap();
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.contains("Palette 'bad'") && e.contains("invalid YAML")),
        "Should detect invalid palette YAML: {:?}",
        result.errors
    );
}

#[test]
fn test_verify_cli_exits_nonzero_on_error() {
    // Drives the `verify` subcommand through main() so the `if !result.is_ok()`
    // exit branch is exercised: a config enrolling a template that doesn't
    // exist must make the process exit non-zero.
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    fs::write(
        config_dir.join("themis.yaml"),
        format!(
            r#"
enroll:
  kitty:
    type: template
    input: "{}/missing.j2"
    output: "{}/out/kitty.conf"
"#,
            config_dir.display(),
            config_dir.display()
        ),
    )
    .unwrap();

    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_themis"));
    cmd.args(["--config", config_dir.to_str().unwrap(), "verify"]);
    common::isolate_env(&mut cmd, temp_dir.path());
    let output = cmd.output().unwrap();

    assert!(
        !output.status.success(),
        "verify should exit non-zero when a template is missing"
    );
}

#[test]
fn test_verify_cli_exits_zero_on_valid_config() {
    // The complementary success branch: a valid config makes `verify` exit 0.
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");
    fs::create_dir_all(&config_dir).unwrap();

    // Empty enrollment is valid; nothing to fail on.
    fs::write(config_dir.join("themis.yaml"), "enroll: {}").unwrap();

    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_themis"));
    cmd.args(["--config", config_dir.to_str().unwrap(), "verify"]);
    common::isolate_env(&mut cmd, temp_dir.path());
    let output = cmd.output().unwrap();

    assert!(
        output.status.success(),
        "verify should exit zero on a valid config: {}",
        String::from_utf8_lossy(&output.stderr)
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
fn test_doctor_counts_multiple_skipped_apps() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");

    fs::create_dir_all(&config_dir).unwrap();

    // Enroll two apps with no doctor check defined: both are skipped. Using
    // two (not one) makes the skipped counter's increment observable — a
    // mutant that turns `+= 1` into `*= 1` leaves the count stuck below 2.
    fs::write(
        config_dir.join("themis.yaml"),
        r#"
enroll:
  gtk:
    type: command
    commands:
      - "gsettings set org.gnome.desktop.interface color-scheme prefer-dark"
  custom-thing:
    type: command
    commands:
      - "true"
"#,
    )
    .unwrap();

    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_themis"));
    cmd.args(["--config", config_dir.to_str().unwrap(), "doctor"]);
    common::isolate_env(&mut cmd, temp_dir.path());
    let output = cmd.output().unwrap();

    assert!(output.status.success(), "Should succeed when all skipped");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("2 skipped"),
        "Should report two skipped apps: {}",
        combined
    );
}

#[test]
fn test_doctor_counts_multiple_ok_apps() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("config");

    fs::create_dir_all(&config_dir).unwrap();

    // Enroll two known apps (kitty, foot) whose configs each carry the
    // expected include line. Using two makes the OK counter's increment
    // observable: a `+= 1` -> `*= 1` mutant cannot reach 2.
    fs::write(
        config_dir.join("themis.yaml"),
        r#"
enroll:
  kitty:
    type: template
    input: "~/.config/themis/templates/kitty.j2"
    output: "~/.config/kitty/.themis.conf"
  foot:
    type: template
    input: "~/.config/themis/templates/foot.j2"
    output: "~/.config/foot/themis.ini"
"#,
    )
    .unwrap();

    let kitty_dir = temp_dir.path().join(".config/kitty");
    fs::create_dir_all(&kitty_dir).unwrap();
    fs::write(
        kitty_dir.join("kitty.conf"),
        "# Kitty config\ninclude .themis.conf\n",
    )
    .unwrap();

    let foot_dir = temp_dir.path().join(".config/foot");
    fs::create_dir_all(&foot_dir).unwrap();
    fs::write(
        foot_dir.join("foot.ini"),
        "include=~/.config/foot/themis.ini\n",
    )
    .unwrap();

    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_themis"));
    cmd.args(["--config", config_dir.to_str().unwrap(), "doctor"]);
    common::isolate_env(&mut cmd, temp_dir.path());
    let output = cmd.output().unwrap();

    assert!(
        output.status.success(),
        "Should succeed when both includes present: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("2 OK"),
        "Should report two OK apps: {}",
        combined
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
