use std::collections::HashMap;

use super::{TemplatingEngine, RenderError};

pub struct Mustache {}

impl Mustache {
    pub fn new() -> Self {
        Mustache {}
    }
}

impl TemplatingEngine for Mustache {
    fn render_template(&self, template: &str, values: &HashMap<&str, &str>) -> Result<String, RenderError> {
        let template = mustache::compile_str(template)?;

        Ok(template.render_to_string(&values)?)
    }
}

impl From<mustache::Error> for RenderError {
    fn from(e: mustache::Error) -> Self {
        RenderError {
            inner: Box::new(e),
        }
    }
}