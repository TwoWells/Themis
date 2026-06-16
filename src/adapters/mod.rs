// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
//! Concrete implementations of the core I/O traits.
//!
//! Each adapter implements one of the abstractions from
//! [`crate::core::traits`]:
//!
//! - [`filesystem`] / [`command`] / [`template`] — real I/O used in normal runs.
//! - [`dryrun`] — logging-only adapters for `--dry-run` previews.
pub mod command;
pub mod dryrun;
pub mod filesystem;
pub mod template;

#[cfg(test)]
pub(crate) mod mock;
