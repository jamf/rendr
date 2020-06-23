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

// Implicit conversion of any error type to the RenderError wrapper. This is so
// that we can use the ? shorthand and so that life is easy.
impl<E: Error + 'static> From<E> for RenderError {
    fn from(err: E) -> Self {
        RenderError {
            inner: Box::new(err),
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
