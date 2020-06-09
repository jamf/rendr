use std::collections::HashMap;

pub fn render_template(template: &str, values: &HashMap<&str, &str>) -> String {
    let template = mustache::compile_str(template).unwrap();

    template.render_to_string(&values).unwrap()
}