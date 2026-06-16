// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
//! Themis — a theme orchestrator CLI for Linux.
//!
//! Themis manages switching system themes (light/dark and named palettes)
//! across multiple applications. It acts as a "general contractor" for desktop
//! theming: it does not generate colors, but manages the _who, what, and when_
//! of applying them through profiles and integrations.
//!
//! # Modules
//!
//! - [`core`] — domain logic: orchestration, configuration, profiles, state,
//!   integrations, and the I/O trait abstractions.
//! - [`adapters`] — concrete trait implementations (real I/O, dry-run, Tera).
//! - [`commands`] — CLI command implementations (`init`, `verify`, `doctor`,
//!   `status`).
pub mod adapters;
pub mod commands;
pub mod core;
