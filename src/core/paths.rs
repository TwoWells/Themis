// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
//! Cross-platform path resolution for Themis.
//!
//! Themis honors the [XDG Base Directory] environment variables on every
//! platform — Linux and macOS alike — with `$HOME`-relative defaults as the
//! fallback. There is deliberately no platform-native strategy (no
//! `~/Library/Application Support` on macOS, no Known Folders on Windows): a
//! config directory ports between platforms unchanged, and `XDG_*_HOME` steers
//! resolution everywhere.
//!
//! - **config** — `$XDG_CONFIG_HOME` else `~/.config`, then `themis`
//! - **state** — `$XDG_STATE_HOME` else `~/.local/state`, then
//!   `themis/state.json`
//! - **data / system palettes** — every entry of `$XDG_DATA_DIRS` (default
//!   `/usr/local/share:/usr/share`), each suffixed with `themis`, plus the
//!   Homebrew prefixes on macOS — returned as an ordered search list.
//!
//! Each public resolver reads the process environment, then delegates to a pure
//! inner function that takes the resolved env values as explicit arguments. The
//! pure functions carry the logic (and the tests), keeping the impure env reads
//! a thin shell — the same split `state.rs` uses for its clock read.
//!
//! [XDG Base Directory]: https://specifications.freedesktop.org/basedir-spec/latest/

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Directory name Themis owns under each base directory.
const THEMIS_DIR: &str = "themis";

/// File name of the persisted state document.
const STATE_FILE: &str = "state.json";

/// The XDG default search path for system data directories.
const XDG_DATA_DIRS_DEFAULT: &str = "/usr/local/share:/usr/share";

/// Resolve a base directory from a pre-read XDG env value, falling back to a
/// `$HOME`-relative default.
///
/// `xdg` is the value of the relevant `XDG_*_HOME` variable (if set), `home`
/// the value of `HOME` (if set). Returns `$xdg` when present, otherwise
/// `$HOME/<home_relative>`.
///
/// # Errors
///
/// Returns an error if `xdg` is `None` and `home` is also `None`.
fn resolve_base(xdg: Option<String>, home: Option<String>, home_relative: &str) -> Result<PathBuf> {
    if let Some(base) = xdg {
        Ok(PathBuf::from(base))
    } else {
        let home = home.context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(home_relative))
    }
}

/// Read an env var, treating an empty value as unset.
fn env_nonempty(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}

/// Resolve the user config directory following the XDG Base Directory spec.
///
/// Returns `$XDG_CONFIG_HOME/themis` if `XDG_CONFIG_HOME` is set, otherwise
/// `~/.config/themis`.
///
/// # Errors
///
/// Returns an error if neither `XDG_CONFIG_HOME` nor `HOME` is set.
pub fn config_dir() -> Result<PathBuf> {
    Ok(resolve_base(
        env_nonempty("XDG_CONFIG_HOME"),
        env_nonempty("HOME"),
        ".config",
    )?
    .join(THEMIS_DIR))
}

/// Resolve the state file path following the XDG Base Directory spec.
///
/// Returns `$XDG_STATE_HOME/themis/state.json` if `XDG_STATE_HOME` is set,
/// otherwise `~/.local/state/themis/state.json`.
///
/// # Errors
///
/// Returns an error if neither `XDG_STATE_HOME` nor `HOME` is set.
pub fn state_file() -> Result<PathBuf> {
    Ok(resolve_base(
        env_nonempty("XDG_STATE_HOME"),
        env_nonempty("HOME"),
        ".local/state",
    )?
    .join(THEMIS_DIR)
    .join(STATE_FILE))
}

/// Resolve the ordered search list of system data directories.
///
/// Each entry is a `themis` directory under a system data root. System
/// palettes live at `<entry>/palettes/<name>.yaml`. The list is searched in
/// order, first match wins, and is consulted only after the user's own data.
///
/// The list is derived from `$XDG_DATA_DIRS` (default
/// `/usr/local/share:/usr/share`), suffixing each entry with `themis`. On
/// macOS the Homebrew prefix share directories (`/opt/homebrew/share`,
/// `/usr/local/share`) are appended as well, so a `brew`-installed palette set
/// resolves. Duplicate directories are removed while preserving order, and the
/// list is never empty even when `XDG_DATA_DIRS` is unset or empty.
#[must_use]
pub fn system_data_dirs() -> Vec<PathBuf> {
    resolve_system_data_dirs(env_nonempty("XDG_DATA_DIRS"), cfg!(target_os = "macos"))
}

/// Pure core of [`system_data_dirs`]: build the search list from a pre-read
/// `XDG_DATA_DIRS` value and a platform flag.
///
/// `xdg_data_dirs` is the value of `XDG_DATA_DIRS` (already treated as `None`
/// when empty); `is_macos` selects the Homebrew prefixes. Taking `is_macos` as
/// a parameter — rather than reading `cfg!` directly — keeps the macOS branch
/// live (and testable) on every platform.
fn resolve_system_data_dirs(xdg_data_dirs: Option<String>, is_macos: bool) -> Vec<PathBuf> {
    let data_dirs = xdg_data_dirs.unwrap_or_else(|| XDG_DATA_DIRS_DEFAULT.to_string());

    let mut roots: Vec<PathBuf> = data_dirs
        .split(':')
        .filter(|entry| !entry.is_empty())
        .map(PathBuf::from)
        .collect();

    // On macOS, also search the Homebrew prefixes so a `brew`-installed palette
    // set resolves. `/opt/homebrew` is the Apple-silicon default; `/usr/local`
    // is the Intel default (and the XDG fallback already, but explicit here so
    // it is searched even when XDG_DATA_DIRS overrides the default).
    if is_macos {
        roots.push(PathBuf::from("/opt/homebrew/share"));
        roots.push(PathBuf::from("/usr/local/share"));
    }

    let mut dirs: Vec<PathBuf> = Vec::new();
    for root in roots {
        let dir = root.join(THEMIS_DIR);
        if !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }
    dirs
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests assert via unwrap/expect"
    )]

    use super::*;

    #[test]
    fn resolve_base_prefers_xdg() {
        let path = resolve_base(
            Some("/xdg/cfg".to_string()),
            Some("/home/t".to_string()),
            ".config",
        )
        .unwrap();
        assert_eq!(path, PathBuf::from("/xdg/cfg"));
    }

    #[test]
    fn resolve_base_falls_back_to_home() {
        let path = resolve_base(None, Some("/home/tester".to_string()), ".config").unwrap();
        assert_eq!(path, PathBuf::from("/home/tester/.config"));
    }

    #[test]
    fn resolve_base_state_relative() {
        let path = resolve_base(None, Some("/home/tester".to_string()), ".local/state").unwrap();
        assert_eq!(path, PathBuf::from("/home/tester/.local/state"));
    }

    #[test]
    fn resolve_base_errors_without_xdg_or_home() {
        assert!(
            resolve_base(None, None, ".config").is_err(),
            "resolve_base must error when both the XDG value and HOME are absent"
        );
    }

    // config_dir / state_file compose resolve_base with the themis suffix; pin
    // the full shape against the resolved bases.
    #[test]
    fn config_path_appends_themis() {
        let base = resolve_base(Some("/xdg/cfg".to_string()), None, ".config").unwrap();
        assert_eq!(base.join(THEMIS_DIR), PathBuf::from("/xdg/cfg/themis"));
    }

    #[test]
    fn state_path_appends_themis_state_json() {
        let base = resolve_base(Some("/xdg/state".to_string()), None, ".local/state").unwrap();
        assert_eq!(
            base.join(THEMIS_DIR).join(STATE_FILE),
            PathBuf::from("/xdg/state/themis/state.json")
        );
    }

    #[test]
    fn system_dirs_uses_xdg_data_dirs() {
        let dirs = resolve_system_data_dirs(Some("/a:/b".to_string()), false);
        assert_eq!(
            dirs,
            vec![PathBuf::from("/a/themis"), PathBuf::from("/b/themis")]
        );
    }

    #[test]
    fn system_dirs_defaults_when_unset() {
        let dirs = resolve_system_data_dirs(None, false);
        assert_eq!(
            dirs,
            vec![
                PathBuf::from("/usr/local/share/themis"),
                PathBuf::from("/usr/share/themis"),
            ]
        );
    }

    #[test]
    fn system_dirs_skips_empty_entries() {
        // Leading/trailing/embedded empty fields (`::`) must not yield a bare
        // `themis` dir.
        let dirs = resolve_system_data_dirs(Some(":/a::/b:".to_string()), false);
        assert_eq!(
            dirs,
            vec![PathBuf::from("/a/themis"), PathBuf::from("/b/themis")]
        );
    }

    #[test]
    fn system_dirs_dedups_preserving_order() {
        let dirs = resolve_system_data_dirs(Some("/dup:/dup:/other".to_string()), false);
        assert_eq!(
            dirs,
            vec![PathBuf::from("/dup/themis"), PathBuf::from("/other/themis")]
        );
    }

    #[test]
    fn system_dirs_macos_appends_homebrew_prefixes() {
        // With the default XDG dirs, macOS adds /opt/homebrew/share and
        // de-dups /usr/local/share (already an XDG default).
        let dirs = resolve_system_data_dirs(None, true);
        assert_eq!(
            dirs,
            vec![
                PathBuf::from("/usr/local/share/themis"),
                PathBuf::from("/usr/share/themis"),
                PathBuf::from("/opt/homebrew/share/themis"),
            ],
            "macOS must search the Homebrew prefixes, de-duplicating /usr/local/share"
        );
    }

    #[test]
    fn system_dirs_macos_homebrew_even_when_xdg_overridden() {
        // When XDG_DATA_DIRS is overridden away from the defaults, the Homebrew
        // prefixes are still searched on macOS.
        let dirs = resolve_system_data_dirs(Some("/custom".to_string()), true);
        assert_eq!(
            dirs,
            vec![
                PathBuf::from("/custom/themis"),
                PathBuf::from("/opt/homebrew/share/themis"),
                PathBuf::from("/usr/local/share/themis"),
            ]
        );
    }

    #[test]
    fn system_dirs_non_macos_omits_homebrew() {
        let dirs = resolve_system_data_dirs(Some("/custom".to_string()), false);
        assert_eq!(dirs, vec![PathBuf::from("/custom/themis")]);
        assert!(
            !dirs.contains(&PathBuf::from("/opt/homebrew/share/themis")),
            "non-macOS must not search the Homebrew prefix"
        );
    }
}
