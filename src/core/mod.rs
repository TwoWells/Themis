// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
//! Domain logic for Themis.
//!
//! This module holds the orchestration engine and its supporting types:
//!
//! - [`orchestrator`] — loads profiles and applies integrations.
//! - [`config`] — the `themis.yaml` schema (enrolled apps + overrides).
//! - [`profile`] — profiles and palettes with include-based inheritance.
//! - [`integration`] — the four integration kinds (template, symlink,
//!   command, script).
//! - [`paths`] — cross-platform resolution of the config, state, and system
//!   data directories from the XDG environment variables.
//! - [`state`] — persistence of the currently loaded profile.
//! - [`traits`] — the I/O abstractions ([`traits::FileSystem`],
//!   [`traits::TemplateRenderer`], [`traits::CommandExecutor`]) that decouple
//!   the engine from real I/O.
pub mod config;
pub mod integration;
pub mod orchestrator;
pub mod paths;
pub mod profile;
pub mod state;
pub mod traits;

#[cfg(test)]
mod tests;
