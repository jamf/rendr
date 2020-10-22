use super::{RenderError, TemplatingEngine};
use crate::blueprint::Values;

/// [{{ mustache }}](https://mustache.github.io/) is a simple, logic-less templating engine.
pub struct Mustache {}

impl Mustache {
    pub fn new() -> Self {
        Mustache {}
    }
}

impl TemplatingEngine for Mustache {
    fn render_template(&self, template: &str, values: Values) -> Result<String, RenderError> {
        let template = mustache::compile_str(template)?;

        Ok(template.render_to_string(&values)?)
    }
}

impl From<mustache::Error> for RenderError {
    fn from(e: mustache::Error) -> Self {
        RenderError { inner: Box::new(e) }
    }
}

#[test]
fn render_valid_template() {
    use std::collections::HashMap;

    let template = "name: {{ name }}, value: {{ value }}";

    let values: HashMap<_, _> = [("name", "foo"), ("value", "bar"), ("asd", "dsa")]
        .iter()
        .cloned()
        .collect();

    assert_eq!(
        Mustache::new()
            .render_template(template, values.into())
            .unwrap(),
        "name: foo, value: bar",
    );
}
