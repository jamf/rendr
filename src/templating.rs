use std::collections::HashMap;

pub fn render_template(template: &str, values: &HashMap<&str, &str>) -> String {
    let template = mustache::compile_str(template).unwrap();

    template.render_to_string(&values).unwrap()
}

#[test]
fn render_valid_template() {
    let template = "name: {{ name }}, value: {{ value }}";

    let values: HashMap<_, _> = [("name", "foo"), ("value", "bar"), ("asd", "dsa")]
        .iter().cloned().collect();

    assert_eq!(
        render_template(template, &values),
        "name: foo, value: bar",
    );
}