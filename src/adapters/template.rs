// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Two Wells <contact@twowells.dev>
use crate::core::traits::TemplateRenderer;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use tera::{Context as TeraContext, Tera};

#[derive(Default)]
pub struct TeraAdapter;

impl TeraAdapter {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl TemplateRenderer for TeraAdapter {
    fn render(&self, template_content: &str, context: &HashMap<String, Value>) -> Result<String> {
        let mut tera_ctx = TeraContext::new();
        for (k, v) in context {
            tera_ctx.insert(k, v);
        }

        // Tera requires registering a template string to render it if it's not a file
        // Or we can use render_str (one-off)
        let rendered = Tera::one_off(template_content, &tera_ctx, false)
            .context("Failed to render template string")?;

        Ok(rendered)
    }
}
