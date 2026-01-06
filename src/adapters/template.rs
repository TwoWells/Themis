use crate::core::traits::TemplateRenderer;
use anyhow::{Context, Result};
use std::collections::HashMap;
use serde_json::Value;
use tera::{Tera, Context as TeraContext};

pub struct TeraAdapter;

impl TeraAdapter {
    pub fn new() -> Self {
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
