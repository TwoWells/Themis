#[cfg(test)]
mod tests {
    use crate::adapters::mock::{MockCommandExecutor, MockFileSystem, MockTemplateRenderer};
    use crate::core::orchestrator::Orchestrator;
    use crate::core::traits::FileSystem;
    use std::path::PathBuf;

    fn setup() -> (
        MockFileSystem,
        Orchestrator<MockFileSystem, MockTemplateRenderer, MockCommandExecutor>,
    ) {
        let fs = MockFileSystem::new();
        let tera = MockTemplateRenderer::default();
        let cmd = MockCommandExecutor::default();
        let config_dir = PathBuf::from("/config");

        let orchestrator = Orchestrator::new(fs.clone(), tera, cmd, config_dir);
        (fs, orchestrator)
    }

    fn setup_with_executor() -> (
        MockFileSystem,
        MockCommandExecutor,
        Orchestrator<MockFileSystem, MockTemplateRenderer, MockCommandExecutor>,
    ) {
        let fs = MockFileSystem::new();
        let tera = MockTemplateRenderer::default();
        let cmd = MockCommandExecutor::default();
        let config_dir = PathBuf::from("/config");

        let orchestrator = Orchestrator::new(fs.clone(), tera, cmd.clone(), config_dir);
        (fs, cmd, orchestrator)
    }

    #[test]
    fn test_profile_inheritance() {
        let (fs, orchestrator) = setup();

        // 1. Setup Files
        // theman.yaml
        fs.add_file(
            "/config/theman.yaml",
            r##"
            current_profile: nord
            enroll:
              kitty:
                type: template
                input: /config/template.j2
                output: /config/out.conf
        "##,
        );

        // profiles/dark.yaml (Parent)
        fs.add_file(
            "/config/profiles/dark.yaml",
            r##"
            metadata:
              name: dark
            vars:
              mode: dark
              bg: "#000000"
              font: "Sans"
        "##,
        );

        // profiles/nord.yaml (Child)
        fs.add_file(
            "/config/profiles/nord.yaml",
            r##"
            metadata:
              name: nord
            extends: dark
            vars:
              bg: "#2E3440" 
        "##,
        );

        // Template File
        fs.add_file(
            "/config/template.j2",
            "mode={{ mode }} bg={{ bg }} font={{ font }}",
        );

        // 2. Run
        let result = orchestrator.load_profile("nord");
        assert!(result.is_ok(), "Failed to load profile: {:?}", result.err());

        // 3. Verify Output
        // The child 'nord' should have:
        // - mode: dark (Inherited)
        // - font: Sans (Inherited)
        // - bg: #2E3440 (Overridden)

        let output = fs
            .read_to_string(&PathBuf::from("/config/out.conf"))
            .unwrap();
        assert_eq!(output, "mode=dark bg=#2E3440 font=Sans");
    }

    #[test]
    fn test_circular_inheritance_detected() {
        let (fs, orchestrator) = setup();

        // theman.yaml
        fs.add_file(
            "/config/theman.yaml",
            r##"
            current_profile: a
            enroll: {}
        "##,
        );

        // profiles/a.yaml extends b
        fs.add_file(
            "/config/profiles/a.yaml",
            r##"
            metadata:
              name: a
            extends: b
            vars: {}
        "##,
        );

        // profiles/b.yaml extends a (circular!)
        fs.add_file(
            "/config/profiles/b.yaml",
            r##"
            metadata:
              name: b
            extends: a
            vars: {}
        "##,
        );

        let result = orchestrator.load_profile("a");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Circular profile inheritance detected"),
            "Expected circular inheritance error, got: {}",
            err
        );
    }

    #[test]
    fn test_self_referential_inheritance_detected() {
        let (fs, orchestrator) = setup();

        fs.add_file(
            "/config/theman.yaml",
            r##"
            current_profile: self_ref
            enroll: {}
        "##,
        );

        // Profile extends itself
        fs.add_file(
            "/config/profiles/self_ref.yaml",
            r##"
            metadata:
              name: self_ref
            extends: self_ref
            vars: {}
        "##,
        );

        let result = orchestrator.load_profile("self_ref");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Circular profile inheritance detected"),
            "Expected circular inheritance error, got: {}",
            err
        );
    }

    #[test]
    fn test_reload_signal_sends_pkill() {
        let (fs, cmd, orchestrator) = setup_with_executor();

        fs.add_file(
            "/config/theman.yaml",
            r##"
            enroll:
              waybar:
                type: template
                input: /config/template.j2
                output: /config/out.conf
                reload_signal: SIGUSR2
        "##,
        );

        fs.add_file(
            "/config/profiles/test.yaml",
            r##"
            metadata:
              name: test
            vars:
              color: "#000000"
        "##,
        );

        fs.add_file("/config/template.j2", "color={{ color }}");

        let result = orchestrator.load_profile("test");
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let executed = cmd.executed.lock().unwrap();
        assert!(
            executed.iter().any(|c| c == "pkill -USR2 waybar"),
            "Expected pkill command, got: {:?}",
            *executed
        );
    }

    #[test]
    fn test_reload_signal_without_sig_prefix() {
        let (fs, cmd, orchestrator) = setup_with_executor();

        fs.add_file(
            "/config/theman.yaml",
            r##"
            enroll:
              kitty:
                type: template
                input: /config/template.j2
                output: /config/out.conf
                reload_signal: USR1
        "##,
        );

        fs.add_file(
            "/config/profiles/test.yaml",
            r##"
            metadata:
              name: test
            vars: {}
        "##,
        );

        fs.add_file("/config/template.j2", "test");

        let result = orchestrator.load_profile("test");
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let executed = cmd.executed.lock().unwrap();
        assert!(
            executed.iter().any(|c| c == "pkill -USR1 kitty"),
            "Expected pkill command, got: {:?}",
            *executed
        );
    }

    #[test]
    fn test_script_env_arrays_are_colon_delimited() {
        let (fs, cmd, orchestrator) = setup_with_executor();

        fs.add_file(
            "/config/theman.yaml",
            r##"
            enroll:
              myapp:
                type: script
                path: /usr/bin/theme-script
        "##,
        );

        fs.add_file(
            "/config/profiles/test.yaml",
            r##"
            metadata:
              name: test
            vars:
              colors: ["#111111", "#222222", "#333333"]
              single: "value"
              number: 42
        "##,
        );

        let result = orchestrator.load_profile("test");
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let env = cmd.script_env.lock().unwrap();
        assert_eq!(
            env.get("THEMAN_COLORS"),
            Some(&"#111111:#222222:#333333".to_string()),
            "Array should be colon-delimited"
        );
        assert_eq!(
            env.get("THEMAN_SINGLE"),
            Some(&"value".to_string()),
            "String should pass through"
        );
        assert_eq!(
            env.get("THEMAN_NUMBER"),
            Some(&"42".to_string()),
            "Number should stringify"
        );
    }
}
