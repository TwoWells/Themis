use theman::core::orchestrator::Orchestrator;
use theman::adapters::filesystem::RealFileSystem;
use theman::adapters::template::TeraAdapter;
use theman::adapters::command::RealCommandExecutor;
use std::fs;
use tempfile::TempDir;

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
    let config_content = format!(r##"
        current_profile: test-dark
        enroll:
          test_app:
            type: template
            input: "{}/test.j2"
            output: "{}"
    "##, templates_dir.display(), output_file.display());
    
    fs::write(config_dir.join("theman.yaml"), config_content).unwrap();
    
    // profiles/test-dark.yaml
    fs::write(profiles_dir.join("test-dark.yaml"), r##"
        metadata:
          name: test-dark
        vars:
          bg: "#123456"
    "##).unwrap();
    
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
