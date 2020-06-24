use std::collections::HashMap;

use super::{TemplatingEngine, RenderError};

/// [{{ mustache }}](https://mustache.github.io/) is a simple, logic-less templating engine.
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

#[test]
fn render_valid_template() {
    let template = "name: {{ name }}, value: {{ value }}";

    let values: HashMap<_, _> = [("name", "foo"), ("value", "bar"), ("asd", "dsa")]
        .iter().cloned().collect();

    assert_eq!(
        Mustache::new()
            .render_template(template, &values)
            .unwrap(),
        "name: foo, value: bar",
    );
}