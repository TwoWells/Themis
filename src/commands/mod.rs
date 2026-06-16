// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
//! CLI command implementations.
//!
//! Each submodule implements one subcommand, returning a structured result
//! that `main` renders for the user:
//!
//! - [`init`] — scaffold the config directory with sample files.
//! - [`verify`] — validate config, templates, and palette references.
//! - [`doctor`] — check that enrolled apps include the generated partials.
//! - [`status`] — report the currently loaded profile.
pub mod doctor;
pub mod init;
pub mod status;
pub mod verify;
