use std::fs;
use tempfile::TempDir;
use theman::adapters::command::RealCommandExecutor;
use theman::adapters::dryrun::{DryRunCommandExecutor, DryRunFileSystem};
use theman::adapters::filesystem::RealFileSystem;
use theman::adapters::template::TeraAdapter;
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
