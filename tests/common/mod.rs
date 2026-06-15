// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
//! Shared integration test utilities.
//!
//! Each integration test file is a separate compilation unit, so
//! `mod common;` imports this module to share [`isolate_env`] and the
//! `xdg_*` path helpers without copy-pasting.

#![allow(dead_code, reason = "each test crate compiles common separately")]

use std::path::{Path, PathBuf};
use std::process::Command;

// ── Environment isolation ────────────────────────────────────────────

/// Isolates a subprocess from the user's environment.
///
/// Points each XDG base dir at a *distinct* subdir of the given root —
/// `XDG_CONFIG_HOME` → `<root>/config`, `XDG_STATE_HOME` → `<root>/state`,
/// `XDG_DATA_HOME` → `<root>/data`, `XDG_CACHE_HOME` → `<root>/cache` — so
/// the process uses the test's tempdir instead of `~/.config`,
/// `~/.local/state`, `~/.local/share`, or `~/.cache`. Keeping the bases
/// distinct makes `isolate_env` a mislocation detector: code that writes
/// under the *wrong* base no longer silently lands in one shared directory.
///
/// Themis resolves its state path from `XDG_STATE_HOME` ahead of `HOME`
/// (see `State::state_path`) and its config dir via the `directories`
/// crate (which honors `XDG_CONFIG_HOME`), so setting `HOME` alone does
/// *not* isolate a subprocess — on a machine where `XDG_STATE_HOME` is
/// exported it would read and write the user's real
/// `~/.local/state/themis/state.json`. Pointing the XDG bases at the
/// tempdir closes that hole. `HOME` is also pointed at `root` so any
/// `~`-expansion (e.g. `doctor`'s `shellexpand::tilde` on app config
/// paths) and `directories` `HOME` fallbacks resolve inside the tempdir.
///
/// Clears every inherited `THEMIS_*` env var (e.g. `THEMIS_CONFIG_DIR`,
/// the `--config` flag's env var, plus any `THEMIS_<VAR>` runtime vars)
/// so the user's shell can't leak settings into the subprocess. It's
/// prefix-based rather than a hand-maintained list, so a newly-added var
/// is covered for free. Callers re-set specific vars (e.g. a custom
/// `XDG_STATE_HOME`) explicitly *after* this call.
///
/// `PATH` is intentionally left untouched: Themis's command/script
/// integrations need real binaries, and the tests using this helper
/// don't spawn any.
///
/// Test-side code that resolves a path the subprocess reads or writes
/// (config, state) must derive it through [`xdg_config_home`] /
/// [`xdg_state_home`] so both sides agree on the split layout.
pub fn isolate_env(cmd: &mut Command, root: impl AsRef<Path>) {
    let root = root.as_ref();
    cmd.env("HOME", root);
    cmd.env("XDG_CONFIG_HOME", xdg_config_home(root));
    cmd.env("XDG_STATE_HOME", xdg_state_home(root));
    cmd.env("XDG_DATA_HOME", xdg_data_home(root));
    cmd.env("XDG_CACHE_HOME", xdg_cache_home(root));
    // Clear every inherited `THEMIS_*` var so the user's shell can't leak
    // settings (config dir, runtime vars, …) into the subprocess. Prefix-based,
    // not a hand-maintained list, so a newly-added var is covered for free.
    for (key, _) in std::env::vars_os() {
        if key.to_str().is_some_and(|k| k.starts_with("THEMIS_")) {
            cmd.env_remove(&key);
        }
    }
}

/// The `XDG_CONFIG_HOME` subdir [`isolate_env`] configures under `root`.
///
/// The `directories` crate resolves user config at
/// `$XDG_CONFIG_HOME/themis/`, so a test writing a config the subprocess
/// must read writes under this path.
pub fn xdg_config_home(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join("config")
}

/// The `XDG_STATE_HOME` subdir [`isolate_env`] configures under `root`.
///
/// `State::state_path()` resolves to `$XDG_STATE_HOME/themis/state.json`,
/// so test-side code computing the state path must resolve through this
/// helper.
pub fn xdg_state_home(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join("state")
}

/// The `XDG_DATA_HOME` subdir [`isolate_env`] configures under `root`.
pub fn xdg_data_home(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join("data")
}

/// The `XDG_CACHE_HOME` subdir [`isolate_env`] configures under `root`.
pub fn xdg_cache_home(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join("cache")
}
