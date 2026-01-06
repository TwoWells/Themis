#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::core::orchestrator::Orchestrator;
    use crate::core::traits::FileSystem;
    use crate::adapters::mock::{MockFileSystem, MockTemplateRenderer, MockCommandExecutor};
    
    fn setup() -> (MockFileSystem, Orchestrator<MockFileSystem, MockTemplateRenderer, MockCommandExecutor>) {
        let fs = MockFileSystem::new();
        let tera = MockTemplateRenderer::default();
        let cmd = MockCommandExecutor::default();
        let config_dir = PathBuf::from("/config");
        
        let orchestrator = Orchestrator::new(fs.clone(), tera, cmd, config_dir);
        (fs, orchestrator)
    }

    #[test]
    fn test_profile_inheritance() {
        let (fs, orchestrator) = setup();

        // 1. Setup Files
        // theman.yaml
        fs.add_file("/config/theman.yaml", r##"
            current_profile: nord
            enroll:
              kitty:
                type: template
                input: /config/template.j2
                output: /config/out.conf
        "##);

        // profiles/dark.yaml (Parent)
        fs.add_file("/config/profiles/dark.yaml", r##"
            metadata:
              name: dark
            vars:
              mode: dark
              bg: "#000000"
              font: "Sans"
        "##);

        // profiles/nord.yaml (Child)
        fs.add_file("/config/profiles/nord.yaml", r##"
            metadata:
              name: nord
            extends: dark
            vars:
              bg: "#2E3440" 
        "##);
        
        // Template File
        fs.add_file("/config/template.j2", "mode={{ mode }} bg={{ bg }} font={{ font }}");

        // 2. Run
        let result = orchestrator.load_profile("nord");
        assert!(result.is_ok(), "Failed to load profile: {:?}", result.err());

        // 3. Verify Output
        // The child 'nord' should have:
        // - mode: dark (Inherited)
        // - font: Sans (Inherited)
        // - bg: #2E3440 (Overridden)
        
        let output = fs.read_to_string(&PathBuf::from("/config/out.conf")).unwrap();
        assert_eq!(output, "mode=dark bg=#2E3440 font=Sans");
    }
}
