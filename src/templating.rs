use std::fmt::Display;
use std::fmt::Formatter;
use std::error::Error;
use std::collections::HashMap;

pub fn render_template(template: &str, values: &HashMap<&str, &str>) -> Result<String, RenderError> {
    let template = mustache::compile_str(template)?;

    Ok(template.render_to_string(&values)?)
}

#[derive(Debug)]
pub struct RenderError {
    inner: Box<dyn Error>,
}

impl Display for RenderError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "template rendering failed: {}", self.inner)?;

        Ok(())
    }
}

impl Error for RenderError {}

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
        render_template(template, &values).unwrap(),
        "name: foo, value: bar",
    );
}
