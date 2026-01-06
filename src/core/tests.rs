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
        let system_dir = PathBuf::from("/system");

        let orchestrator =
            Orchestrator::with_system_dir(fs.clone(), tera, cmd, config_dir, system_dir);
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
        let system_dir = PathBuf::from("/system");

        let orchestrator =
            Orchestrator::with_system_dir(fs.clone(), tera, cmd.clone(), config_dir, system_dir);
        (fs, cmd, orchestrator)
    }

    #[test]
    fn test_profile_includes_palette() {
        let (fs, orchestrator) = setup();

        // theman.yaml
        fs.add_file(
            "/config/theman.yaml",
            r##"
            enroll:
              kitty:
                type: template
                input: /config/template.j2
                output: /config/out.conf
        "##,
        );

        // System palette: dark (base colors)
        fs.add_file(
            "/system/palettes/dark.yaml",
            r##"
            vars:
              mode: dark
              bg: "#000000"
              font: "Sans"
        "##,
        );

        // User profile: nord (includes dark palette, overrides bg)
        fs.add_file(
            "/config/profiles/nord.yaml",
            r##"
            include: dark
            vars:
              bg: "#2E3440"
        "##,
        );

        // Template File
        fs.add_file(
            "/config/template.j2",
            "mode={{ mode }} bg={{ bg }} font={{ font }}",
        );

        let result = orchestrator.load_profile("nord");
        assert!(result.is_ok(), "Failed to load profile: {:?}", result.err());

        // nord should have:
        // - mode: dark (from palette)
        // - font: Sans (from palette)
        // - bg: #2E3440 (overridden in profile)
        let output = fs
            .read_to_string(&PathBuf::from("/config/out.conf"))
            .unwrap();
        assert_eq!(output, "mode=dark bg=#2E3440 font=Sans");
    }

    #[test]
    fn test_palette_inheritance() {
        let (fs, orchestrator) = setup();

        fs.add_file(
            "/config/theman.yaml",
            r##"
            enroll:
              kitty:
                type: template
                input: /config/template.j2
                output: /config/out.conf
        "##,
        );

        // System palette: base
        fs.add_file(
            "/system/palettes/base.yaml",
            r##"
            vars:
              font: "Monospace"
              size: 12
        "##,
        );

        // System palette: nord (includes base)
        fs.add_file(
            "/system/palettes/nord.yaml",
            r##"
            include: base
            vars:
              bg: "#2E3440"
              fg: "#D8DEE9"
        "##,
        );

        // User profile includes nord
        fs.add_file(
            "/config/profiles/myprofile.yaml",
            r##"
            include: nord
            vars:
              font: "JetBrains Mono"
        "##,
        );

        fs.add_file(
            "/config/template.j2",
            "bg={{ bg }} font={{ font }} size={{ size }}",
        );

        let result = orchestrator.load_profile("myprofile");
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let output = fs
            .read_to_string(&PathBuf::from("/config/out.conf"))
            .unwrap();
        // base: font=Monospace, size=12
        // nord: bg=#2E3440, fg=#D8DEE9 (inherits font, size from base)
        // myprofile: font=JetBrains Mono (overrides)
        assert_eq!(output, "bg=#2E3440 font=JetBrains Mono size=12");
    }

    #[test]
    fn test_user_palette_overrides_system() {
        let (fs, orchestrator) = setup();

        fs.add_file(
            "/config/theman.yaml",
            r##"
            enroll:
              kitty:
                type: template
                input: /config/template.j2
                output: /config/out.conf
        "##,
        );

        // System palette: nord
        fs.add_file(
            "/system/palettes/nord.yaml",
            r##"
            vars:
              bg: "#2E3440"
        "##,
        );

        // User palette: nord (overrides system)
        fs.add_file(
            "/config/palettes/nord.yaml",
            r##"
            vars:
              bg: "#1a1a1a"
        "##,
        );

        fs.add_file(
            "/config/profiles/test.yaml",
            r##"
            include: nord
        "##,
        );

        fs.add_file("/config/template.j2", "bg={{ bg }}");

        let result = orchestrator.load_profile("test");
        assert!(result.is_ok(), "Failed: {:?}", result.err());

        let output = fs
            .read_to_string(&PathBuf::from("/config/out.conf"))
            .unwrap();
        // User palette should take precedence
        assert_eq!(output, "bg=#1a1a1a");
    }

    #[test]
    fn test_circular_include_detected() {
        let (fs, orchestrator) = setup();

        fs.add_file(
            "/config/theman.yaml",
            r##"
            enroll: {}
        "##,
        );

        // Palette a includes b
        fs.add_file(
            "/config/palettes/a.yaml",
            r##"
            include: b
            vars: {}
        "##,
        );

        // Palette b includes a (circular!)
        fs.add_file(
            "/config/palettes/b.yaml",
            r##"
            include: a
            vars: {}
        "##,
        );

        // Profile includes a
        fs.add_file(
            "/config/profiles/test.yaml",
            r##"
            include: a
        "##,
        );

        let result = orchestrator.load_profile("test");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Circular include detected"),
            "Expected circular include error, got: {}",
            err
        );
    }

    #[test]
    fn test_self_referential_include_detected() {
        let (fs, orchestrator) = setup();

        fs.add_file(
            "/config/theman.yaml",
            r##"
            enroll: {}
        "##,
        );

        // Palette includes itself
        fs.add_file(
            "/config/palettes/self_ref.yaml",
            r##"
            include: self_ref
            vars: {}
        "##,
        );

        fs.add_file(
            "/config/profiles/test.yaml",
            r##"
            include: self_ref
        "##,
        );

        let result = orchestrator.load_profile("test");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Circular include detected"),
            "Expected circular include error, got: {}",
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
