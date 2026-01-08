use std::fs;
use tempfile::TempDir;
use theman::adapters::command::RealCommandExecutor;
use theman::adapters::dryrun::{DryRunCommandExecutor, DryRunFileSystem};
use theman::adapters::filesystem::RealFileSystem;
use theman::adapters::template::TeraAdapter;
use theman::commands::{init, verify};
use theman::core::orchestrator::Orchestrator;

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

    // theman.yaml
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

    fs::write(config_dir.join("theman.yaml"), config_content).unwrap();

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
    let result = orchestrator.load_profile("test-dark");
    assert!(result.is_ok(), "Load profile failed: {:?}", result.err());

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

    fs::write(config_dir.join("theman.yaml"), config_content).unwrap();

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
    let result = orchestrator.load_profile("test-dark");
    assert!(result.is_ok(), "Load profile failed: {:?}", result.err());

    // 5. Verify output file was NOT created
    assert!(
        !output_file.exists(),
        "Output file should not exist in dry-run mode"
    );
}

#[test]
fn test_init_creates_config_structure() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("theman");

    // Run init
    let result = init::run(&config_dir);
    assert!(result.is_ok(), "Init failed: {:?}", result.err());

    // Verify directories created
    assert!(config_dir.join("profiles").is_dir());
    assert!(config_dir.join("palettes").is_dir());
    assert!(config_dir.join("templates").is_dir());

    // Verify files created
    assert!(config_dir.join("theman.yaml").is_file());
    assert!(config_dir.join("profiles/example.yaml").is_file());

    // Verify config is valid YAML
    let config_content = fs::read_to_string(config_dir.join("theman.yaml")).unwrap();
    assert!(config_content.contains("enroll:"));

    // Verify profile is valid YAML
    let profile_content = fs::read_to_string(config_dir.join("profiles/example.yaml")).unwrap();
    assert!(profile_content.contains("vars:"));
}

#[test]
fn test_init_is_idempotent() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join("theman");

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
    assert!(config_dir.join("theman.yaml").is_file());
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
        config_dir.join("theman.yaml"),
        r#"
enroll:
  kitty:
    type: template
    input: "~/.config/theman/templates/kitty.j2"
    output: "~/.config/kitty/.theman.conf"
"#,
    )
    .unwrap();

    // Create template (so verify passes)
    let home = std::env::var("HOME").unwrap();
    let template_dir = format!("{}/.config/theman/templates", home);
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
    fs::write(config_dir.join("theman.yaml"), "enroll: {}").unwrap();

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
    fs::write(config_dir.join("theman.yaml"), "enroll: {}").unwrap();

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
