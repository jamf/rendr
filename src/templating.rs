use mustache::MapBuilder;

pub fn render_template(s: &str) -> String {
    let template = mustache::compile_str(s).unwrap();

    let data = MapBuilder::new()
        .insert_str("name", "Venus")
        .insert_str("version", "0.1.0")
        .insert_str("foobar", "42")
        .build();

    template.render_data_to_string(&data).unwrap()
}