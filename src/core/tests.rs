#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests assert via unwrap/expect"
    )]
    #![allow(clippy::module_inception, reason = "core test module by convention")]
    #![allow(
        clippy::needless_raw_string_hashes,
        clippy::default_constructed_unit_structs,
        clippy::uninlined_format_args,
        clippy::significant_drop_tightening,
        reason = "test scaffolding: uniform raw YAML fixtures, mock construction, and assert! message args"
    )]

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

        // themis.yaml
        fs.add_file(
            "/config/themis.yaml",
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

        let load_result = orchestrator.load_profile("nord").unwrap();
        assert!(
            load_result.is_ok(),
            "Apps failed: {:?}",
            load_result.failures
        );

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
            "/config/themis.yaml",
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

        let load_result = orchestrator.load_profile("myprofile").unwrap();
        assert!(
            load_result.is_ok(),
            "Apps failed: {:?}",
            load_result.failures
        );

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
            "/config/themis.yaml",
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

        let load_result = orchestrator.load_profile("test").unwrap();
        assert!(
            load_result.is_ok(),
            "Apps failed: {:?}",
            load_result.failures
        );

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
            "/config/themis.yaml",
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
            "/config/themis.yaml",
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
            "/config/themis.yaml",
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

        let load_result = orchestrator.load_profile("test").unwrap();
        assert!(
            load_result.is_ok(),
            "Apps failed: {:?}",
            load_result.failures
        );

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
            "/config/themis.yaml",
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

        let load_result = orchestrator.load_profile("test").unwrap();
        assert!(
            load_result.is_ok(),
            "Apps failed: {:?}",
            load_result.failures
        );

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
            "/config/themis.yaml",
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

        let load_result = orchestrator.load_profile("test").unwrap();
        assert!(
            load_result.is_ok(),
            "Apps failed: {:?}",
            load_result.failures
        );

        let env = cmd.script_env.lock().unwrap();
        assert_eq!(
            env.get("THEMIS_COLORS"),
            Some(&"#111111:#222222:#333333".to_string()),
            "Array should be colon-delimited"
        );
        assert_eq!(
            env.get("THEMIS_SINGLE"),
            Some(&"value".to_string()),
            "String should pass through"
        );
        assert_eq!(
            env.get("THEMIS_NUMBER"),
            Some(&"42".to_string()),
            "Number should stringify"
        );
    }

    #[test]
    fn test_load_result_captures_failures() {
        let (fs, orchestrator) = setup();

        fs.add_file(
            "/config/themis.yaml",
            r##"
            enroll:
              goodapp:
                type: template
                input: /config/good.j2
                output: /config/good.out
              badapp:
                type: template
                input: /config/nonexistent.j2
                output: /config/bad.out
              anotherapp:
                type: template
                input: /config/good.j2
                output: /config/another.out
        "##,
        );

        fs.add_file(
            "/config/profiles/test.yaml",
            r##"
            vars:
              color: "#000000"
        "##,
        );

        // Only add the good template, badapp's template doesn't exist
        fs.add_file("/config/good.j2", "color={{ color }}");

        let load_result = orchestrator.load_profile("test").unwrap();

        // Should have partial success
        assert!(!load_result.is_ok(), "Should have failures");
        assert_eq!(load_result.success_count(), 2, "Two apps should succeed");
        assert_eq!(load_result.failure_count(), 1, "One app should fail");

        // Check failure details
        let failure = &load_result.failures[0];
        assert_eq!(failure.app_name, "badapp");
        assert!(
            failure.error.contains("not found"),
            "Error should mention file not found: {}",
            failure.error
        );
    }

    #[test]
    fn test_load_result_all_succeed() {
        let (fs, orchestrator) = setup();

        fs.add_file(
            "/config/themis.yaml",
            r##"
            enroll:
              app1:
                type: template
                input: /config/template.j2
                output: /config/app1.out
              app2:
                type: template
                input: /config/template.j2
                output: /config/app2.out
        "##,
        );

        fs.add_file(
            "/config/profiles/test.yaml",
            r##"
            vars:
              bg: "#123456"
        "##,
        );

        fs.add_file("/config/template.j2", "bg={{ bg }}");

        let load_result = orchestrator.load_profile("test").unwrap();

        assert!(load_result.is_ok(), "All apps should succeed");
        assert_eq!(load_result.success_count(), 2);
        assert_eq!(load_result.failure_count(), 0);
        assert!(load_result.failures.is_empty());
    }
}
